use std::{net::IpAddr, sync::Arc};

use proto::{
    agent::{instance_service_client::InstanceServiceClient, Signal, SignalInstruction},
    controller::node_service_client::NodeServiceClient,
    scheduler::{Instance, Resource, Status},
};
use tokio::sync::{
    mpsc::{self},
    Mutex,
};
use tonic::{Request, Streaming};

use crate::{
    manager::Manager,
    parser::{instance::InstanceParser, resource::ResourceParser},
    storage::{IStorage, Storage},
    InstanceIdentifier, ProxyError,
};

use super::Node;

/// NodeRegistered represents a node that is registered to the cluster. It is responsible for
/// keeping track of the node's status to the controller, create/stop/destroy instances on the node.
///
/// Properties:
///
/// * `id`: The id of the node.
/// * `node`: The node that is being registered.
/// * `address`: The address of the node.
/// * `tx`: This is the channel that the node will use to send status updates to the controller.
/// * `grpc_client`: The grpc client for the node.
/// * `instances`: The instances that are running on the node.
#[derive(Debug)]
pub struct NodeRegistered {
    pub id: String,
    pub node: Node,
    pub address: IpAddr,
    pub tx: Option<mpsc::Sender<Result<proto::controller::NodeStatus, tonic::Status>>>,
    pub grpc_client: Option<InstanceServiceClient<tonic::transport::Channel>>,
    pub instances: Storage<Instance>,
}

impl NodeRegistered {
    /// `new` creates a new `NodeProxied` struct
    ///
    /// Arguments:
    ///
    /// * `id`: The id of the node.
    /// * `node`: The node that this proxied node is connected to.
    /// * `address`: The IP address of the node.
    ///
    /// Returns:
    ///
    /// A new instance of the NodeProxied struct.
    pub fn new(id: String, node: Node, address: IpAddr) -> Self {
        NodeRegistered {
            id,
            node,
            address,
            tx: None,
            grpc_client: None,
            instances: Storage::new(),
        }
    }

    /// This function connects to the gRPC server and stores the client in the `grpc_client` field of
    /// the `Proxy` struct
    ///
    /// Returns:
    ///
    /// A Result<(), ProxyError>
    pub async fn connect(&mut self) -> Result<(), ProxyError> {
        let addr = format!("http://{}:{}", self.address, "50053");

        let client = InstanceServiceClient::connect(addr)
            .await
            .map_err(ProxyError::TonicTransportError)?;

        self.grpc_client = Some(client);
        Ok(())
    }

    /// It creates the node status stream between the controller and the scheduler.
    /// It forwards the node status to the controller.
    ///
    /// Arguments:
    ///
    /// * `client`: The client that will be used to send the request to the node.
    ///
    /// Returns:
    ///
    /// A Result<(), ProxyError>
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

    /// This function updates the status of the node and sends the updated status to the controller
    ///
    /// Arguments:
    ///
    /// * `status`: The status of the node.
    /// * `description`: A string that describes the status of the node.
    /// * `resource`: The resource that the node is currently running.
    ///
    /// Returns:
    ///
    /// A Result<(), ProxyError>
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
                    .get_all()
                    .values()
                    .map(|instance| InstanceParser::fake_controller_instance(instance.id.clone()))
                    .collect(),
            }))
            .await
            .map_err(|_| ProxyError::ChannelSenderError)?;

        Ok(())
    }

    /// Create a new instance to the node and return the InstanceStatus streaming.
    ///
    /// Arguments:
    ///
    /// * `instance`: Instance - The instance to create.
    ///
    /// Returns:
    ///
    /// Streaming of InstanceStatus - The streaming of the instance status.
    pub async fn create_instance(
        &mut self,
        instance: Instance,
    ) -> Result<Streaming<proto::agent::InstanceStatus>, ProxyError> {
        let client = self
            .grpc_client
            .as_mut()
            .ok_or(ProxyError::GrpcClientNotFound)?;

        let request = Self::wrap_request(InstanceParser::to_agent_instance(instance.clone()));

        let response = client
            .create(request)
            .await
            .map_err(ProxyError::TonicStatusError)?;

        Ok(response.into_inner())
    }

    /// Send a stop signal to the node for the given instance.
    ///
    /// Arguments:
    ///
    /// * `id`: InstanceIdentifier - The instance identifier of the instance to stop.
    ///
    /// Returns:
    ///
    /// A future that resolves to a result of either a unit or a ProxyError.
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

    /// Send a kill signal to the node for the given instance.
    ///
    /// Arguments:
    ///
    /// * `id`: InstanceIdentifier - The instance identifier of the instance to be killed.
    ///
    /// Returns:
    ///
    /// A future that resolves to a result of either a unit or a ProxyError.
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

    /// This function takes a request and returns a request wrapped with tonic.
    ///
    /// Arguments:
    ///
    /// * `request`: The request object that you want to wrap.
    ///
    /// Returns:
    ///
    /// A Request object with the request as the inner value.
    pub fn wrap_request<T>(request: T) -> Request<T> {
        Request::new(request)
    }
}
