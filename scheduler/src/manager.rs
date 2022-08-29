use anyhow::Result;
use std::sync::Arc;

use anyhow::Result;
use log::{debug, info};
use proto::scheduler::{
    instance_service_server::InstanceServiceServer, node_service_server::NodeServiceServer,
    InstanceStatus, NodeRegisterResponse, NodeUnregisterResponse, Status,
};
use tokio::sync::{mpsc, oneshot};
use tokio::{sync::Mutex, task::JoinHandle};
use tonic::{transport::Server, Response};
use uuid::Uuid;

use crate::SchedulerError;
use crate::{
    config::Config, instance::InstanceListener, node::NodeListener, orchestrator::Orchestrator,
    storage::Storage, Event,
};
use crate::{ManagerError, ProxyError};

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
        info!("creating grpc server ...");
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
                        log::info!("scheduling a new instance {:?} ...", instance.id);

                        match orchestrator
                            .lock()
                            .await
                            .create_instance(instance.clone(), tx.clone())
                            .await
                        {
                            Ok(_) => {
                                // todo: proxy the stream to the controller
                            }
                            Err(err) => {
                                log::error!(
                                    "error while scheduling instance : {:?} ({:?})",
                                    instance.id,
                                    err
                                );

                                let instance_status = InstanceStatus {
                                    id: Uuid::new_v4().to_string(),
                                    status: Status::Failed.into(),
                                    status_description: format!(
                                        "Error thrown by the orchestrator: {:?}",
                                        err
                                    ),
                                    resource: None,
                                };

                                let _ = tx.send(Ok(instance_status)).await;
                            }
                        }
                    }
                    Event::InstanceStop(id, tx) => {
                        log::trace!("received instance stop event : {:?}", id);

                        match orchestrator.lock().await.stop_instance(id.clone()).await {
                            Ok(_) => {
                                log::info!("stopped instance : {:?}", id);

                                tx.send(Ok(Response::new(()))).unwrap();
                            }
                            Err(err) => {
                                log::error!("error while stopping instance : {:?} ({:?})", id, err);

                                tx.send(Err(tonic::Status::internal(format!(
                                    "Error thrown by the orchestrator: {:?}",
                                    err
                                ))))
                                .unwrap();
                            }
                        };
                    }
                    Event::InstanceDestroy(id, tx) => {
                        log::trace!("received instance destroy event : {:?}", id);

                        match orchestrator.lock().await.destroy_instance(id.clone()).await {
                            Ok(_) => {
                                log::info!("destroyed instance : {:?}", id);

                                tx.send(Ok(Response::new(()))).unwrap();
                            }
                            Err(err) => {
                                log::error!(
                                    "error while destroying instance : {:?} ({:?})",
                                    id,
                                    err
                                );

                                tx.send(Err(tonic::Status::internal(format!(
                                    "Error thrown by the orchestrator: {:?}",
                                    err
                                ))))
                                .unwrap();
                            }
                        };
                    }
                    Event::NodeRegister(request, addr, tx) => {
                        log::trace!("received node register event : {:?}", request);

                        // todo: parse certificate and get the node information
                        let node = Node {
                            id: Uuid::new_v4().to_string(),
                            status: Status::Starting,
                            resource: None,
                        };

                        match orchestrator
                            .lock()
                            .await
                            .register_node(node.clone(), addr, controller_client.clone())
                            .await
                        {
                            Ok(_) => {
                                log::info!("successfully registered node: {:?}", node.id);

                                let response = NodeRegisterResponse {
                                    code: 0,
                                    description: "Welcome to the cluster".to_string(),
                                    subnet: "".to_string(),
                                };

                                tx.send(Ok(tonic::Response::new(response))).unwrap();
                            }
                            Err(err) => {
                                log::error!(
                                    "error while registering node : {:?} ({:?})",
                                    node.id,
                                    err
                                );

                                let response = NodeRegisterResponse {
                                    code: 1,
                                    description: format!(
                                        "Error thrown by the orchestrator: {:?}",
                                        err
                                    ),
                                    subnet: "".to_string(),
                                };

                                tx.send(Ok(tonic::Response::new(response))).unwrap();
                            }
                        };
                    }
                    Event::NodeUnregister(request, tx) => {
                        log::trace!("received node unregister event : {:?}", request);

                        match orchestrator
                            .lock()
                            .await
                            .unregister_node(request.id.clone())
                        {
                            Ok(_) => {
                                log::info!("successfully unregistered node {:?}", request.id);

                                let response = NodeUnregisterResponse {
                                    code: 0,
                                    description: "Bye from the cluster".to_string(),
                                };

                                tx.send(Ok(tonic::Response::new(response))).unwrap();
                            }
                            Err(err) => {
                                log::error!(
                                    "error while unregistering node : {:?} ({:?})",
                                    request.id,
                                    err
                                );

                                let response = NodeUnregisterResponse {
                                    code: 1,
                                    description: format!(
                                        "Error thrown by the orchestrator: {:?}",
                                        err
                                    ),
                                };

                                tx.send(Ok(tonic::Response::new(response))).unwrap();
                            }
                        };
                    }
                    Event::NodeStatus(status, tx) => {
                        log::trace!("received node status event : {:?}", status);

                        match orchestrator
                            .lock()
                            .await
                            .update_node_status(status.id.clone(), status.clone())
                            .await
                        {
                            Ok(_) => {
                                log::debug!("successfully updated node status : {:?}", status.id);

                                tx.send(Ok(())).await.unwrap();
                            }
                            Err(err) => {
                                log::info!(
                                    "error while updating node status : {:?} ({:?})",
                                    status.id,
                                    err
                                );

                                tx.send(Err(tonic::Status::internal(format!(
                                    "Error thrown by the orchestrator: {:?}",
                                    err
                                ))))
                                .await
                                .unwrap();
                            }
                        };
                    }
                }
            }
        })
    }

    async fn connect_to_controller(&mut self) -> Result<(), ProxyError> {
        let addr = format!(
            "http://{}:{}",
            self.config.controller.host, self.config.controller.port
        );

        log::info!("connecting to controller at {} ...", addr);

        let client = NodeServiceClient::connect(addr)
            .await
            .map_err(ProxyError::TonicTransportError)?;

        log::info!("successfully connected to controller");
        self.grpc_controller_client = Arc::new(Mutex::new(Some(client)));
        Ok(())
    }

    pub async fn run(&mut self) -> Result<(), ManagerError> {
        // connect to the controller
        self.connect_to_controller()
            .await
            .map_err(ManagerError::CannotConnectToController)?;

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
