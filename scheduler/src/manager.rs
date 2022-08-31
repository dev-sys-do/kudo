use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use log::{debug, info};
use proto::controller::node_service_client::NodeServiceClient;
use proto::scheduler::{
    instance_service_server::InstanceServiceServer, node_service_server::NodeServiceServer,
};
use tokio::sync::{mpsc, Mutex};
use tokio::time;
use tokio::{sync::oneshot, task::JoinHandle};
use tonic::transport::Server;

use crate::event::handlers::instance_create::InstanceCreateHandler;
use crate::event::handlers::instance_destroy::InstanceDestroyHandler;
use crate::event::handlers::instance_stop::InstanceStopHandler;
use crate::event::handlers::instance_stream_crash::InstanceStreamCrashHandler;
use crate::event::handlers::instance_terminated::InstanceTerminatedHandler;
use crate::event::handlers::node_register::NodeRegisterHandler;
use crate::event::handlers::node_status::NodeStatusHandler;
use crate::event::handlers::node_stream_crash::NodeStreamCrashHandler;
use crate::event::handlers::node_unregister::NodeUnregisterHandler;
use crate::event::Event;
use crate::instance::listener::InstanceListener;
use crate::node::listener::NodeListener;
use crate::orchestrator::Orchestrator;
use crate::SchedulerError;
use crate::{config::Config, storage::Storage};

#[derive(Debug)]
pub struct Manager {
    config: Arc<Config>,
    grpc_controller_client: Arc<Mutex<Option<NodeServiceClient<tonic::transport::Channel>>>>,
    orchestrator: Arc<Mutex<Orchestrator>>,
}

impl Manager {
    /// `new` creates a new `Manager` struct with two empty `Storage` structs
    ///
    /// Returns:
    ///
    /// A new Manager struct
    pub fn new(config: Config) -> Self {
        let config = Arc::new(config);
        let nodes = Storage::new();

        let orchestrator = Orchestrator::new(nodes, config.clone());

        Manager {
            config,
            grpc_controller_client: Arc::new(Mutex::new(None)),
            orchestrator: Arc::new(Mutex::new(orchestrator)),
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
        info!("creating grpc server ...");
        let addr = format!("{}:{}", self.config.host, self.config.port)
            .parse()
            .map_err(|_| SchedulerError::InvalidGrpcAddress)?;

        let node_listener = NodeListener::new(tx.clone());
        debug!("create node listener with data : {:?}", node_listener);

        let instance_listener = InstanceListener::new(tx);
        debug!(
            "create instance listener with data : {:?}",
            instance_listener
        );

        Ok(tokio::spawn(async move {
            info!("started grpc server at {}", addr);

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
    /// * `tx`: mpsc::Sender<Event>
    ///
    /// Returns:
    ///
    /// A JoinHandle<()>
    fn listen_events(
        &self,
        mut rx: mpsc::Receiver<Event>,
        tx_events: mpsc::Sender<Event>,
    ) -> JoinHandle<()> {
        info!("listening for incoming events ...");
        let orchestrator = self.orchestrator.clone();
        let controller_client = self.grpc_controller_client.clone();

        tokio::spawn(async move {
            while let Some(event) = rx.recv().await {
                debug!("received event : {:?}", event);
                match event {
                    Event::InstanceCreate(instance, tx) => {
                        info!("received instance create event : {:?}", instance);
                        InstanceCreateHandler::handle(
                            orchestrator.clone(),
                            instance,
                            tx,
                            tx_events.clone(),
                        )
                        .await;
                    }
                    Event::InstanceStop(id, tx) => {
                        log::trace!("received instance stop event : {:?}", id);
                        InstanceStopHandler::handle(orchestrator.clone(), id, tx).await;
                    }
                    Event::InstanceDestroy(id, tx) => {
                        log::trace!("received instance destroy event : {:?}", id);
                        InstanceDestroyHandler::handle(orchestrator.clone(), id, tx).await;
                    }
                    Event::InstanceTerminated(id) => {
                        log::trace!("received instance terminated event : {:?}", id);
                        InstanceTerminatedHandler::handle(orchestrator.clone(), id).await;
                    }
                    Event::InstanceStreamCrash(id) => {
                        log::trace!("received instance stream crash event : {:?}", id);
                        InstanceStreamCrashHandler::handle(orchestrator.clone(), id).await;
                    }
                    Event::NodeRegister(request, addr, tx) => {
                        log::trace!("received node register event : {:?}", request);
                        NodeRegisterHandler::handle(
                            orchestrator.clone(),
                            request.certificate,
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
                    Event::NodeStreamCrash(id) => {
                        log::trace!("received node stream crash event : {:?}", id);
                        NodeStreamCrashHandler::handle(orchestrator.clone(), id).await;
                    }
                }
            }
        })
    }

    /// It tries to connect to the controller every 5 seconds until it succeeds
    async fn connect_to_controller(&mut self) {
        let addr = format!(
            "http://{}:{}",
            self.config.controller.host, self.config.controller.port
        );

        log::info!("connecting to controller at {} ...", addr);

        let mut interval = time::interval(Duration::from_secs(5));
        loop {
            interval.tick().await;
            match NodeServiceClient::connect(addr.clone()).await {
                Ok(client) => {
                    self.grpc_controller_client = Arc::new(Mutex::new(Some(client)));
                    break;
                }
                Err(err) => {
                    log::error!("error while connecting to controller : {:?}", err);
                }
            }
        }

        log::info!("successfully connected to controller");
    }

    /// The function creates a channel to communicate with the orchestrator, creates a gRPC server and a
    /// listener for incoming events, and then waits for the end of all the threads
    ///
    /// Returns:
    ///
    /// A Result<(), Box<dyn std::error::Error>>
    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut handlers = vec![];
        let (tx, rx) = Self::create_mpsc_channel();

        // create listeners and serve the grpc server
        handlers.push(self.create_grpc_server(tx.clone())?);

        // connect to the controller
        self.connect_to_controller().await;

        // listen for incoming events and pass them to the orchestrator
        handlers.push(self.listen_events(rx, tx));

        info!("scheduler running and ready to receive incoming requests ...");

        // wait the end of all the threads
        for handler in handlers {
            handler.await?;
        }

        Ok(())
    }
}
