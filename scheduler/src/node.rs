use std::{net::IpAddr, sync::Arc};

use log;
use proto::{
    agent::{instance_service_client::InstanceServiceClient, Signal, SignalInstruction},
    controller::node_service_client::NodeServiceClient,
    scheduler::{
        node_service_server::NodeService, Instance, NodeRegisterRequest, NodeRegisterResponse,
        NodeStatus, NodeUnregisterRequest, NodeUnregisterResponse, Resource, Status,
    },
};
use tokio::sync::{mpsc, Mutex};
use tonic::{Request, Response, Streaming};

use crate::{
    event::Event,
    manager::Manager,
    parser::{InstanceParser, ResourceParser},
    InstanceIdentifier, ProxyError,
};

#[derive(Debug, Clone)]
pub struct Node {
    pub id: String,
    pub status: Status,
    pub resource: Option<Resource>,
}

#[derive(Debug)]
pub struct NodeListener {
    sender: mpsc::Sender<Event>,
}

impl NodeListener {
    pub fn new(sender: mpsc::Sender<Event>) -> Self {
        NodeListener { sender }
    }
}

#[tonic::async_trait]
impl NodeService for NodeListener {
    async fn status(
        &self,
        request: Request<Streaming<NodeStatus>>,
    ) -> Result<Response<()>, tonic::Status> {
        log::debug!("received gRPC request: {:?}", request);

        let mut stream = request.into_inner();

        // send each status to the manager
        loop {
            let (tx, mut rx) = Manager::create_mpsc_channel();
            let message = stream.message().await?;

            match message {
                Some(node_status) => {
                    self.sender
                        .send(Event::NodeStatus(node_status, tx.clone()))
                        .await
                        .unwrap();

                    // wait for the manager to respond
                    if let Some(res) = rx.recv().await {
                        match res {
                            Ok(_) => {}
                            Err(err) => return Err(err),
                        }
                    }
                }
                None => {
                    log::error!("Node status stream closed");
                    // todo: emit node crash event (get the node id from the first status)
                    return Ok(Response::new(()));
                }
            }
        }
    }

    async fn register(
        &self,
        request: Request<NodeRegisterRequest>,
    ) -> Result<Response<NodeRegisterResponse>, tonic::Status> {
        log::debug!("received gRPC request: {:?}", request);

        let (tx, rx) = Manager::create_oneshot_channel();
        let remote_addr = request.remote_addr().unwrap().ip();
        log::debug!("Registering a new node from: {:?}", remote_addr);

        match self
            .sender
            .send(Event::NodeRegister(request.into_inner(), remote_addr, tx))
            .await
        {
            Ok(_) => {
                return rx.await.unwrap();
            }
            Err(_) => {
                return Err(tonic::Status::internal("could not send event to manager"));
            }
        }
    }

    async fn unregister(
        &self,
        request: Request<NodeUnregisterRequest>,
    ) -> Result<Response<NodeUnregisterResponse>, tonic::Status> {
        log::debug!("received gRPC request: {:?}", request);

        let (tx, rx) = Manager::create_oneshot_channel();

        match self
            .sender
            .send(Event::NodeUnregister(request.into_inner(), tx))
            .await
        {
            Ok(_) => {
                return rx.await.unwrap();
            }
            Err(_) => {
                return Err(tonic::Status::internal("could not send event to manager"));
            }
        }
    }
}

#[derive(Debug)]
pub struct NodeProxied {
    pub id: String,
    pub node: Node,
    pub address: IpAddr,
    pub tx: Option<mpsc::Sender<Result<proto::controller::NodeStatus, tonic::Status>>>,
    pub grpc_client: Option<InstanceServiceClient<tonic::transport::Channel>>,
    pub instances: Vec<Instance>,
}

impl NodeProxied {
    pub fn new(id: String, node: Node, address: IpAddr) -> Self {
        NodeProxied {
            id,
            node,
            address,
            tx: None,
            grpc_client: None,
            instances: Vec::new(),
        }
    }

    pub async fn connect_to_grpc(&mut self) -> Result<(), ProxyError> {
        let addr = format!("http://{}:{}", self.address, "50053");

        let client = InstanceServiceClient::connect(addr)
            .await
            .map_err(ProxyError::TonicTransportError)?;

        self.grpc_client = Some(client);
        Ok(())
    }

    pub async fn open_node_status_stream(
        &mut self,
        client: Arc<Mutex<Option<NodeServiceClient<tonic::transport::Channel>>>>,
    ) -> Result<(), ProxyError> {
        if self.tx.is_some() {
            return Ok(());
        }

        let (tx, mut rx) = Manager::create_mpsc_channel();
        self.tx = Some(tx);

        let node_status_stream = async_stream::stream! {
            loop {
                let event = rx.recv().await;
                match event {
                    Some(Ok(node_status)) => {
                        yield node_status;
                    }
                    Some(Err(_)) => {
                        break;
                    }
                    None => {
                        break;
                    }
                }
            }

            log::debug!("Node status stream closed");
            // todo: emit node crash event (get the node id from the first status)
        };

        let request = Self::wrap_request(node_status_stream);

        tokio::spawn(async move {
            client
                .lock()
                .await
                .as_mut()
                .unwrap()
                .update_node_status(request)
                .await
        });

        Ok(())
    }

    pub async fn update_status(
        &mut self,
        status: Status,
        description: Option<String>,
        resource: Option<Resource>,
    ) -> Result<(), ProxyError> {
        self.node.status = status;
        self.node.resource = resource;

        self.tx
            .as_mut()
            .ok_or(ProxyError::GrpcStreamNotFound)?
            .send(Ok(proto::controller::NodeStatus {
                id: self.id.clone(),
                state: self.node.status.into(),
                status_description: description.unwrap_or_else(|| "".to_string()),
                resource: match self.node.status {
                    Status::Running => Some(ResourceParser::to_controller_resource(
                        self.node.resource.clone().unwrap(),
                    )),
                    _ => None,
                },
                instances: self
                    .instances
                    .iter()
                    .map(|instance| InstanceParser::fake_controller_instance(instance.id.clone()))
                    .collect(),
            }))
            .await
            .map_err(|_| ProxyError::ChannelSenderError)?;

        Ok(())
    }

    pub async fn create_instance(
        &mut self,
        instance: Instance,
    ) -> Result<Streaming<proto::agent::InstanceStatus>, ProxyError> {
        let client = self
            .grpc_client
            .as_mut()
            .ok_or(ProxyError::GrpcClientNotFound)?;

        let request = Self::wrap_request(InstanceParser::to_agent_instance(instance));

        let response = client
            .create(request)
            .await
            .map_err(ProxyError::TonicStatusError)?;

        Ok(response.into_inner())
    }

    pub async fn stop_instance(&mut self, id: InstanceIdentifier) -> Result<(), ProxyError> {
        let client = self
            .grpc_client
            .as_mut()
            .ok_or(ProxyError::GrpcClientNotFound)?;

        let request = Self::wrap_request(SignalInstruction {
            signal: Signal::Stop.into(),
            instance: Some(InstanceParser::fake_agent_instance(id)),
        });

        client
            .signal(request)
            .await
            .map_err(ProxyError::TonicStatusError)?;

        Ok(())
    }

    pub async fn kill_instance(&mut self, id: InstanceIdentifier) -> Result<(), ProxyError> {
        let client = self
            .grpc_client
            .as_mut()
            .ok_or(ProxyError::GrpcClientNotFound)?;

        let request = Self::wrap_request(SignalInstruction {
            signal: Signal::Kill.into(),
            instance: Some(InstanceParser::fake_agent_instance(id)),
        });

        client
            .signal(request)
            .await
            .map_err(ProxyError::TonicStatusError)?;

        Ok(())
    }

    pub fn wrap_request<T>(request: T) -> Request<T> {
        Request::new(request)
    }
}
