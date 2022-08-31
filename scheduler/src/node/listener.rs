use proto::scheduler::{
    node_service_server::NodeService, NodeRegisterRequest, NodeRegisterResponse, NodeStatus,
    NodeUnregisterRequest, NodeUnregisterResponse,
};
use tokio::sync::mpsc;
use tonic::{Request, Response, Streaming};

use crate::{event::Event, manager::Manager};

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
        let mut node_id = None;

        // send each status to the manager
        loop {
            let (tx, mut rx) = Manager::create_mpsc_channel();
            let message = match stream.message().await {
                Ok(message) => message,
                Err(err) => {
                    if let Some(node_id) = node_id {
                        log::error!("node status stream crashed: {:?}", node_id);

                        // send the node stream node event to the manager
                        self.sender.send(Event::NodeStreamCrash(node_id)).await.ok();
                    }

                    return Err(err);
                }
            };

            match message {
                Some(node_status) => {
                    if node_id.is_none() {
                        node_id = Some(node_status.id.clone());
                    }

                    self.sender
                        .send(Event::NodeStatus(node_status, tx.clone()))
                        .await
                        .ok();

                    // wait for the manager to respond
                    if let Some(res) = rx.recv().await {
                        match res {
                            Ok(_) => {}
                            Err(err) => return Err(err),
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
