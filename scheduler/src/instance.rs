use anyhow::Result;
use log;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response};

use proto::scheduler::{
    instance_service_server::InstanceService, Instance, InstanceIdentifier, InstanceStatus, Status,
};
use uuid::Uuid;

use crate::{manager::Manager, Event, NodeIdentifier, ProxyError};

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

#[derive(Debug, Clone)]
pub struct InstanceProxied {
    pub id: String,
    pub instance: Instance,
    pub node_id: Option<NodeIdentifier>,
    pub tx: mpsc::Sender<Result<InstanceStatus, tonic::Status>>,
}

impl InstanceProxied {
    pub fn new(
        id: String,
        instance: Instance,
        node_id: Option<NodeIdentifier>,
        tx: mpsc::Sender<Result<InstanceStatus, tonic::Status>>,
    ) -> Self {
        Self {
            id,
            instance,
            node_id,
            tx,
        }
    }

    pub async fn change_status(
        &mut self,
        status: Status,
        description: Option<String>,
    ) -> Result<(), ProxyError> {
        self.instance.status = status.into();

        self.tx
            .send(Ok(InstanceStatus {
                id: Uuid::new_v4().to_string(),
                status: status.into(),
                status_description: description.unwrap_or_else(|| "".to_string()),
                resource: match self.instance.status() {
                    Status::Running => self.instance.resource.clone(),
                    _ => None,
                },
            }))
            .await
            .map_err(|_| ProxyError::ChannelSenderError)?;

        Ok(())
    }
}
