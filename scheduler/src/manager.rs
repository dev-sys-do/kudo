use anyhow::Result;
use proto::controller::node_service_client::NodeServiceClient;
use std::sync::Arc;
use std::time::Duration;
use tokio::time;

use log;
use proto::scheduler::{
    instance_service_server::InstanceServiceServer, node_service_server::NodeServiceServer,
};
use tokio::sync::{mpsc, oneshot};
use tokio::{sync::Mutex, task::JoinHandle};
use tonic::transport::Server;

use crate::event::handlers::instance_create::InstanceCreateHandler;
use crate::event::handlers::instance_destroy::InstanceDestroyHandler;
use crate::event::handlers::instance_stop::InstanceStopHandler;
use crate::event::handlers::node_register::NodeRegisterHandler;
use crate::event::handlers::node_status::NodeStatusHandler;
use crate::event::handlers::node_unregister::NodeUnregisterHandler;
use crate::event::Event;
use crate::ManagerError;
use crate::SchedulerError;
use crate::{
    config::Config, instance::InstanceListener, node::NodeListener, orchestrator::Orchestrator,
    storage::Storage,
};

#[derive(Debug)]
pub struct Manager {
    config: Arc<Config>,
    orchestrator: Arc<Mutex<Orchestrator>>,
    grpc_controller_client: Arc<Mutex<Option<NodeServiceClient<tonic::transport::Channel>>>>,
}

impl Manager {
    /// The `new` function creates a new `Manager` struct with two empty `Storage` structs and a new `Orchestrator` struct.
    ///
    /// Returns:
    ///
    /// A new Manager struct
    pub fn new(config: Config) -> Self {
        let instances = Storage::new();
        let nodes = Storage::new();
        let config = Arc::new(config);

        let orchestrator = Orchestrator::new(instances, nodes, config.clone());

        Manager {
            config,
            orchestrator: Arc::new(Mutex::new(orchestrator)),
            grpc_controller_client: Arc::new(Mutex::new(None)),
        }
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
    fn create_grpc_server(&self, tx: mpsc::Sender<Event>) -> Result<JoinHandle<()>> {
        log::info!("creating grpc server ...");

        let addr = format!("{}:{}", self.config.host, self.config.port)
            .parse()
            .map_err(|_| SchedulerError::InvalidGrpcAddress)?;

        let node_listener = NodeListener::new(tx.clone());
        log::trace!("create node listener with data : {:?}", node_listener);

        let instance_listener = InstanceListener::new(tx);
        log::trace!(
            "create instance listener with data : {:?}",
            instance_listener
        );

        Ok(tokio::spawn(async move {
            log::info!("started grpc server at {}", addr);

            Server::builder()
                .add_service(NodeServiceServer::new(node_listener))
                .add_service(InstanceServiceServer::new(instance_listener))
                .serve(addr)
                .await
                .unwrap();
        }))
    }

    /// Create a multi-producer, single-consumer channel with a buffer size of 32
    pub fn create_mpsc_channel<T>() -> (mpsc::Sender<T>, mpsc::Receiver<T>) {
        log::trace!("creating mpsc channel ...");
        let (tx, rx): (mpsc::Sender<T>, mpsc::Receiver<T>) = mpsc::channel(32);
        log::trace!("created mpsc channel");
        (tx, rx)
    }

    /// It creates a channel that can be used to send a single message from one thread to another
    pub fn create_oneshot_channel<T>() -> (oneshot::Sender<T>, oneshot::Receiver<T>) {
        log::trace!("creating oneshot channel ...");
        let (tx, rx): (oneshot::Sender<T>, oneshot::Receiver<T>) = oneshot::channel();
        log::trace!("created oneshot channel");
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
        log::debug!("listening for incoming events ...");
        let orchestrator = Arc::clone(&self.orchestrator);
        let controller_client = self.grpc_controller_client.clone();

        tokio::spawn(async move {
            while let Some(event) = rx.recv().await {
                log::debug!("received event : {:?}", event);

                match event {
                    Event::InstanceCreate(instance, tx) => {
                        log::trace!("received instance create event : {:?}", instance);
                        InstanceCreateHandler::handle(orchestrator.clone(), instance, tx).await;
                    }
                    Event::InstanceStop(id, tx) => {
                        log::trace!("received instance stop event : {:?}", id);
                        InstanceStopHandler::handle(orchestrator.clone(), id, tx).await;
                    }
                    Event::InstanceDestroy(id, tx) => {
                        log::trace!("received instance destroy event : {:?}", id);
                        InstanceDestroyHandler::handle(orchestrator.clone(), id, tx).await;
                    }
                    Event::NodeRegister(request, addr, tx) => {
                        log::trace!("received node register event : {:?}", request);
                        NodeRegisterHandler::handle(
                            orchestrator.clone(),
                            addr,
                            controller_client.clone(),
                            tx,
                        )
                        .await;
                    }
                    Event::NodeUnregister(request, tx) => {
                        log::trace!("received node unregister event : {:?}", request);
                        NodeUnregisterHandler::handle(orchestrator.clone(), request.id, tx).await;
                    }
                    Event::NodeStatus(status, tx) => {
                        log::trace!("received node status event : {:?}", status);
                        NodeStatusHandler::handle(orchestrator.clone(), status, tx).await;
                    }
                }
            }
        })
    }

    async fn connect_to_controller(&mut self) {
        let addr = format!(
            "http://{}:{}",
            self.config.controller.host, self.config.controller.port
        );

        log::info!("connecting to controller at {} ...", addr);

        let mut interval = time::interval(Duration::from_secs(5));
        loop {
            match NodeServiceClient::connect(addr.clone()).await {
                Ok(client) => {
                    self.grpc_controller_client = Arc::new(Mutex::new(Some(client)));
                    break;
                }
                Err(err) => {
                    log::error!("error while connecting to controller : {:?}", err);
                }
            }
            interval.tick().await;
        }

        log::info!("successfully connected to controller");
    }

    pub async fn run(&mut self) -> Result<()> {
        // connect to the controller
        self.connect_to_controller().await;

        // create the threads for the gRPC server & the events
        let mut handlers = vec![];
        let (tx, rx) = Self::create_mpsc_channel();

        // create listeners and serve the grpc server
        handlers.push(self.create_grpc_server(tx)?);

        // listen for incoming events and pass them to the orchestrator
        handlers.push(self.listen_events(rx));

        log::info!("scheduler running and ready to receive incoming requests ...");

        // wait the end of all the threads
        for handler in handlers {
            handler.await.map_err(ManagerError::FromTaskError)?;
        }

        Ok(())
    }
}
