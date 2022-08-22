use log::debug;
use proto::scheduler::{node_service_server::NodeService, NodeRegisterRequest, NodeStatus, NodeUnregisterRequest, NodeUnregisterResponse, NodeRegisterResponse};
use tonic::{Request, Status, Response, Streaming};

#[derive(Default, Debug)]
pub struct NodeListener {}

#[tonic::async_trait]
impl NodeService for NodeListener {
    async fn status(
        &self,
        request: Request<Streaming<NodeStatus>>,
    ) -> Result<Response<()>, Status> {
        debug!("{:?}", request);
        Ok(Response::new(()))
    }

    async fn register(
        &self,
        request: Request<NodeRegisterRequest>,
    ) -> Result<Response<NodeRegisterResponse>, Status> {
        debug!("{:?}", request);
        Ok(Response::new(NodeRegisterResponse::default()))
    }

    async fn unregister(
        &self,
        request: Request<NodeUnregisterRequest>,
    ) -> Result<Response<NodeUnregisterResponse>, Status> {
        debug!("{:?}", request);
        Ok(Response::new(NodeUnregisterResponse::default()))
    }
}
