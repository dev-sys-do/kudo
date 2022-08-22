use log::debug;
use proto::scheduler::{
    node_service_server::NodeService, NodeRegisterRequest, NodeRegisterResponse, NodeStatus,
    NodeUnregisterRequest, NodeUnregisterResponse,
};
use tokio::sync::mpsc;
use tonic::{Request, Response, Status, Streaming};

use crate::{manager::Manager, Event};

#[derive(Debug)]
#[allow(dead_code)]
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
    ) -> Result<Response<()>, Status> {
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
    ) -> Result<Response<NodeRegisterResponse>, Status> {
        debug!("{:?}", request);
        let (tx, rx) = Manager::create_oneshot_channel();

        match self
            .sender
            .send(Event::NodeRegister(request.into_inner(), tx))
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

    async fn unregister(
        &self,
        request: Request<NodeUnregisterRequest>,
    ) -> Result<Response<NodeUnregisterResponse>, Status> {
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
                return Err(Status::internal("could not send event to manager"));
            }
        }
    }
}
