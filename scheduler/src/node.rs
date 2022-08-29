use std::net::IpAddr;

use log::debug;
use proto::{
    agent::{instance_service_client::InstanceServiceClient, Signal, SignalInstruction},
    scheduler::{
        node_service_server::NodeService, Instance, NodeRegisterRequest, NodeRegisterResponse,
        NodeStatus, NodeUnregisterRequest, NodeUnregisterResponse, Resource, Status,
    },
};
use tokio::sync::mpsc;
use tonic::{Request, Response, Streaming};

use crate::{instance::InstanceParser, manager::Manager, Event, InstanceIdentifier, ProxyError};

#[derive(Debug)]
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
        let mut stream = request.into_inner();
        let (tx, mut rx) = Manager::create_mpsc_channel();

        loop {
            let message = stream.message().await?;
            match message {
                Some(node_status) => {
                    debug!("Node status: {:?}", node_status);
                    self.sender
                        .send(Event::NodeStatus(node_status, tx.clone()))
                        .await
                        .unwrap();

                    if let Some(res) = rx.recv().await {
                        match res {
                            Ok(()) => {
                                debug!("Node status updated successfully");
                            }
                            Err(err) => {
                                debug!("Error updating node status: {:?}", err);
                                return Err(err);
                            }
                        }
                    }
                }
                None => {
                    return Ok(Response::new(()));
                }
            }
        }
    }

    async fn register(
        &self,
        request: Request<NodeRegisterRequest>,
    ) -> Result<Response<NodeRegisterResponse>, tonic::Status> {
        debug!("{:?}", request);
        let (tx, rx) = Manager::create_oneshot_channel();
        let remote_addr = request.remote_addr().unwrap().ip();

        debug!("Registering node from ip: {:?}", remote_addr);

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
        debug!("{:?}", request);
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
    pub tx: Option<mpsc::Sender<Result<NodeStatus, tonic::Status>>>,
    pub grpc_client: Option<InstanceServiceClient<tonic::transport::Channel>>,
}

impl NodeProxied {
    pub fn new(id: String, node: Node, address: IpAddr) -> Self {
        NodeProxied {
            id,
            node,
            address,
            tx: None,
            grpc_client: None,
        }
    }

    pub async fn connect_to_grpc(&mut self) -> Result<(), ProxyError> {
        let addr = format!("http://{}:{}", self.address.to_string(), "50053");

        let client = InstanceServiceClient::connect(addr)
            .await
            .map_err(ProxyError::TonicTransportError)?;

        self.grpc_client = Some(client);
        Ok(())
    }

    pub async fn update_status(
        &mut self,
        status: Status,
        _: Option<String>,
        resource: Option<Resource>,
    ) -> Result<(), ProxyError> {
        self.node.status = status;
        self.node.resource = resource;

        // todo;
        // self.tx
        //     .send(Ok(InstanceStatus {
        //         id: Uuid::new_v4().to_string(),
        //         status: status.into(),
        //         status_description: description.unwrap_or("".to_string()),
        //         resource: match Status::from_i32(self.instance.status) {
        //             Some(Status::Running) => self.instance.resource.clone(),
        //             _ => None,
        //         },
        //     }))
        //     .await
        //     .map_err(|_| ProxyError::ChannelSenderError)?;

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
