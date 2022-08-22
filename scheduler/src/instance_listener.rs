use log::debug;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};

use proto::scheduler::{
    instance_service_server::InstanceService, Instance, InstanceIdentifier, InstanceStatus,
};

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
    ) -> Result<Response<Self::CreateStream>, Status> {
        debug!("received request: {:?}", request);
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
                return Err(Status::internal("could not send event to manager"));
            }
        }
    }

    type CreateStream = ReceiverStream<Result<InstanceStatus, Status>>;

    async fn start(&self, request: Request<InstanceIdentifier>) -> Result<Response<()>, Status> {
        debug!("received request: {:?}", request);
        let (tx, rx) = Manager::create_oneshot_channel();

        match self
            .sender
            .send(Event::InstanceStart(request.into_inner().id, tx))
            .await
        {
            Ok(_) => {
                return rx.await.unwrap();
            }
            Err(_) => {
                return Err(Status::internal("could not send event to manager"));
            }
        }
    }

    async fn stop(&self, request: Request<InstanceIdentifier>) -> Result<Response<()>, Status> {
        debug!("received request: {:?}", request);
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
                return Err(Status::internal("could not send event to manager"));
            }
        }
    }

    async fn destroy(&self, request: Request<InstanceIdentifier>) -> Result<Response<()>, Status> {
        debug!("received request: {:?}", request);
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
                return Err(Status::internal("could not send event to manager"));
            }
        }
    }
}
