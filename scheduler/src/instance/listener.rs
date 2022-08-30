use proto::scheduler::{
    instance_service_server::InstanceService, Instance, InstanceIdentifier, InstanceStatus,
};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response};

use crate::{manager::Manager, Event};

#[derive(Debug)]
pub struct InstanceListener {
    sender: mpsc::Sender<Event>,
}

impl InstanceListener {
    pub fn new(sender: mpsc::Sender<Event>) -> Self {
        InstanceListener { sender }
    }
}

#[tonic::async_trait]
impl InstanceService for InstanceListener {
    async fn create(
        &self,
        request: Request<Instance>,
    ) -> Result<Response<Self::CreateStream>, tonic::Status> {
        log::debug!("received gRPC request: {:?}", request);
        let (tx, rx) = Manager::create_mpsc_channel();

        match self
            .sender
            .send(Event::InstanceCreate(request.into_inner(), tx))
            .await
        {
            Ok(_) => {
                return Ok(Response::new(ReceiverStream::new(rx)));
            }
            Err(_) => {
                return Err(tonic::Status::internal("could not send event to manager"));
            }
        }
    }

    type CreateStream = ReceiverStream<Result<InstanceStatus, tonic::Status>>;

    async fn start(&self, _: Request<InstanceIdentifier>) -> Result<Response<()>, tonic::Status> {
        Err(tonic::Status::unimplemented("not implemented"))
    }

    async fn stop(
        &self,
        request: Request<InstanceIdentifier>,
    ) -> Result<Response<()>, tonic::Status> {
        log::debug!("received gRPC request: {:?}", request);
        let (tx, rx) = Manager::create_oneshot_channel();

        match self
            .sender
            .send(Event::InstanceStop(request.into_inner().id, tx))
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

    async fn destroy(
        &self,
        request: Request<InstanceIdentifier>,
    ) -> Result<Response<()>, tonic::Status> {
        log::debug!("received gRPC request: {:?}", request);
        let (tx, rx) = Manager::create_oneshot_channel();

        match self
            .sender
            .send(Event::InstanceDestroy(request.into_inner().id, tx))
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
