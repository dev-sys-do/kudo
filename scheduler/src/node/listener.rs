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

        // send each status to the manager
        loop {
            let (tx, mut rx) = Manager::create_mpsc_channel();
            let message = stream.message().await?;

            match message {
                Some(node_status) => {
                    self.sender
                        .send(Event::NodeStatus(node_status, tx.clone()))
                        .await
                        .unwrap();

                    // wait for the manager to respond
                    if let Some(res) = rx.recv().await {
                        match res {
                            Ok(_) => {}
                            Err(err) => return Err(err),
                        }
                    }
                }
                None => {
                    log::error!("Node status stream closed");
                    // todo: emit node crash event (get the node id from the first status)
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
