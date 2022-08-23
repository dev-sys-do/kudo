use std::sync::Arc;

use log::{debug, info};
use proto::scheduler::{
    instance_service_server::InstanceServiceServer, node_service_server::NodeServiceServer,
    Instance, InstanceStatus, NodeRegisterResponse, NodeUnregisterResponse,
};
use tokio::sync::mpsc;
use tokio::{sync::oneshot, task::JoinHandle};
use tonic::{transport::Server, Response};

use crate::{
    instance_listener::InstanceListener, node_listener::NodeListener, storage::Storage, Event, Node,
};

#[derive(Debug)]
pub struct Manager {
    instances: Arc<Storage<Instance>>,
    nodes: Arc<Storage<Node>>,
}

impl Manager {
    /// `new` creates a new `Manager` struct with two empty `Storage` structs
    ///
    /// Returns:
    ///
    /// A new Manager struct
    pub fn new() -> Self {
        let instances = Arc::new(Storage::new());
        let nodes = Arc::new(Storage::new());

        Manager {
            instances: instances,
            nodes: nodes,
        }
    }

    /// This function returns a reference to the instances storage.
    ///
    /// Returns:
    ///
    /// A reference to the instances storage.
    pub fn instances(&self) -> Arc<Storage<Instance>> {
        self.instances.clone()
    }

    /// This function returns a reference to the nodes storage.
    ///
    /// Returns:
    ///
    /// A reference to the nodes storage.
    pub fn nodes(&self) -> Arc<Storage<Node>> {
        self.nodes.clone()
    }

    /// It creates a gRPC server that listens on port 50051 and spawns a new thread to handle incoming
    /// requests
    ///
    /// Arguments:
    ///
    /// * `tx`: mpsc::Sender<Event>
    ///
    /// Returns:
    ///
    /// A JoinHandle<()>
    fn create_grpc_server(&self, tx: mpsc::Sender<Event>) -> JoinHandle<()> {
        info!("creating grpc server ...");
        let addr = "127.0.0.1:50051".parse().unwrap();

        let node_listener = NodeListener::new(tx.clone());
        debug!("create node listener with data : {:?}", node_listener);

        let instance_listener = InstanceListener::new(tx);
        debug!(
            "create instance listener with data : {:?}",
            instance_listener
        );

        tokio::spawn(async move {
            info!("started grpc server at {}", addr);

            Server::builder()
                .add_service(NodeServiceServer::new(node_listener))
                .add_service(InstanceServiceServer::new(instance_listener))
                .serve(addr)
                .await
                .unwrap();
        })
    }

    /// Create a multi-producer, single-consumer channel with a buffer size of 32
    pub fn create_mpsc_channel<T>() -> (mpsc::Sender<T>, mpsc::Receiver<T>) {
        debug!("creating mpsc channel ...");
        let (tx, rx): (mpsc::Sender<T>, mpsc::Receiver<T>) = mpsc::channel(32);
        debug!("created mpsc channel");
        (tx, rx)
    }

    /// It creates a channel that can be used to send a single message from one thread to another
    pub fn create_oneshot_channel<T>() -> (oneshot::Sender<T>, oneshot::Receiver<T>) {
        debug!("creating oneshot channel ...");
        let (tx, rx): (oneshot::Sender<T>, oneshot::Receiver<T>) = oneshot::channel();
        debug!("created oneshot channel");
        (tx, rx)
    }

    /// This function listens for incoming events from the event bus and dispatches them to the
    /// orchestrator
    ///
    /// Arguments:
    ///
    /// * `rx`: mpsc::Receiver<Event>
    ///
    /// Returns:
    ///
    /// A JoinHandle<()>
    fn listen_events(&self, mut rx: mpsc::Receiver<Event>) -> JoinHandle<()> {
        info!("listening for incoming events ...");

        tokio::spawn(async move {
            while let Some(event) = rx.recv().await {
                debug!("received event : {:?}", event);
                match event {
                    Event::InstanceCreate(instance, tx) => {
                        info!("received instance create event : {:?}", instance);
                        tx.send(Ok(InstanceStatus::default())).await.unwrap();
                    }
                    Event::InstanceStart(id, tx) => {
                        info!("received instance start event : {:?}", id);
                        tx.send(Ok(Response::new(()))).unwrap();
                    }
                    Event::InstanceStop(id, tx) => {
                        info!("received instance stop event : {:?}", id);
                        tx.send(Ok(Response::new(()))).unwrap();
                    }
                    Event::InstanceDestroy(id, tx) => {
                        info!("received instance destroy event : {:?}", id);
                        tx.send(Ok(Response::new(()))).unwrap();
                    }
                    Event::NodeRegister(request, tx) => {
                        info!("received node register event : {:?}", request);
                        tx.send(Ok(Response::new(NodeRegisterResponse::default())))
                            .unwrap();
                    }
                    Event::NodeUnregister(request, tx) => {
                        info!("received node unregister event : {:?}", request);
                        tx.send(Ok(Response::new(NodeUnregisterResponse::default())))
                            .unwrap();
                    }
                    Event::NodeStatus(status, tx) => {
                        info!("received node status event : {:?}", status);
                        tx.send(Ok(())).await.unwrap();
                    }
                }
            }
        })
    }

    /// The function creates a channel to communicate with the orchestrator, creates a gRPC server and a
    /// listener for incoming events, and then waits for the end of all the threads
    ///
    /// Returns:
    ///
    /// A Result<(), Box<dyn std::error::Error>>
    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut handlers = vec![];
        let (tx, rx) = Self::create_mpsc_channel();

        // create listeners and serve the grpc server
        handlers.push(self.create_grpc_server(tx));

        // listen for incoming events and pass them to the orchestrator
        handlers.push(self.listen_events(rx));

        info!("scheduler running and ready to receive incoming requests ...");

        // wait the end of all the threads
        for handler in handlers {
            handler.await?;
        }

        Ok(())
    }
}
