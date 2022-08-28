use log::{error, info};
use std::sync::Arc;
use std::{fmt::Display, net::SocketAddr};
use tokio::sync::Mutex;
use tonic::{Code, Request, Response, Status, Streaming};

use super::service::NodeService;

#[derive(Debug)]
pub enum NodeControllerError {
    NodeServiceError(super::service::NodeServiceError),
}

impl Display for NodeControllerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeControllerError::NodeServiceError(err) => {
                write!(f, "NodeServiceError: {}", err)
            }
        }
    }
}

pub struct NodeController {
    node_service: Arc<Mutex<NodeService>>,
}

impl NodeController {
    pub async fn new(
        etcd_address: &SocketAddr,
        grpc_address: &str,
    ) -> Result<Self, NodeControllerError> {
        Ok(NodeController {
            node_service: Arc::new(Mutex::new(
                NodeService::new(etcd_address, grpc_address)
                    .await
                    .map_err(NodeControllerError::NodeServiceError)?,
            )),
        })
    }
}

#[tonic::async_trait]
impl proto::controller::node_service_server::NodeService for NodeController {
    async fn update_node_status(
        &self,
        request: Request<Streaming<proto::controller::NodeStatus>>,
    ) -> Result<Response<()>, Status> {
        let remote_address = if let Some(remote_address) = request.remote_addr() {
            remote_address.to_string()
        } else {
            error!("\"update_node_status\" Failed to get remote address");
            "Error getting remote address".to_string()
        };

        info!(
            "{} \"update_node_status\" streaming initiated",
            remote_address
        );

        let stream = request.into_inner();

        self.node_service
            .clone()
            .lock()
            .await
            .update_node_status(stream, remote_address)
            .await
            .map_err(|err| {
                Status::new(
                    Code::Internal,
                    NodeControllerError::NodeServiceError(err).to_string(),
                )
            })?;

        Ok(Response::new(()))
    }
}
