use log::debug;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};

use proto::scheduler::{
    instance_service_server::InstanceService, Instance, InstanceIdentifier, InstanceStatus,
};

#[derive(Debug, Default)]
pub struct InstanceListener {}

#[tonic::async_trait]
impl InstanceService for InstanceListener {
    async fn create(
        &self,
        request: Request<Instance>,
    ) -> Result<Response<Self::CreateStream>, Status> {
        debug!("{:?}", request);

        let (tx, rx) = mpsc::channel(4);
        Ok(Response::new(ReceiverStream::new(rx)))
    }

    type CreateStream = ReceiverStream<Result<InstanceStatus, Status>>;

    async fn start(&self, request: Request<InstanceIdentifier>) -> Result<Response<()>, Status> {
        debug!("{:?}", request);

        Ok(Response::new(()))
    }

    async fn stop(&self, request: Request<InstanceIdentifier>) -> Result<Response<()>, Status> {
        debug!("{:?}", request);

        Ok(Response::new(()))
    }

    async fn destroy(&self, request: Request<InstanceIdentifier>) -> Result<Response<()>, Status> {
        debug!("{:?}", request);

        Ok(Response::new(()))
    }
}
