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
    config::Config, instance_listener::InstanceListener, node_listener::NodeListener,
    orchestrator::Orchestrator, storage::Storage, Event, Node,
};

#[derive(Debug)]
pub struct Manager {
    config: Arc<Config>,
    orchestrator: Arc<Mutex<Orchestrator>>,
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
    ///
    /// Returns:
    ///
    /// A JoinHandle<()>
    fn listen_events(&self, mut rx: mpsc::Receiver<Event>) -> JoinHandle<()> {
        info!("listening for incoming events ...");
        let orchestrator = Arc::clone(&self.orchestrator);

        tokio::spawn(async move {
            while let Some(event) = rx.recv().await {
                debug!("received event : {:?}", event);
                match event {
                    Event::InstanceCreate(instance, tx) => {
                        info!("received instance create event : {:?}", instance);

                        // should be move in the orchestrator but it's here for testing
                        match orchestrator.lock().await.find_best_node(&instance) {
                            Ok(_) => {
                                info!("found best node for instance : {:?}", instance);
                            }
                            Err(err) => {
                                info!("error finding best node for instance : {:?}", err);

                                let instance_status = InstanceStatus {
                                    id: instance.id,
                                    status: Status::Failed.into(),
                                    status_description: format!(
                                        "Error thrown by the orchestrator: {:?}",
                                        err
                                    ),
                                    resource: None,
                                };

                                let _ = tx.send(Ok(instance_status)).await;
                            }
                        };
                    }
                    Event::InstanceStart(id, tx) => {
                        info!("received instance start event : {:?}", id);

                        match orchestrator.lock().await.start_instance(id.clone()).await {
                            Ok(_) => {
                                info!("started instance : {:?}", id);
                                tx.send(Ok(Response::new(()))).unwrap();
                            }
                            Err(err) => {
                                info!("error while starting instance : {:?}", err);
                                tx.send(Err(tonic::Status::internal(format!(
                                    "Error thrown by the orchestrator: {:?}",
                                    err
                                ))))
                                .unwrap();
                            }
                        };
                    }
                    Event::InstanceStop(id, tx) => {
                        info!("received instance stop event : {:?}", id);

                        match orchestrator.lock().await.stop_instance(id.clone()) {
                            Ok(_) => {
                                info!("stopped instance : {:?}", id);
                                tx.send(Ok(Response::new(()))).unwrap();
                            }
                            Err(err) => {
                                info!("error while stopping instance : {:?}", err);
                                tx.send(Err(tonic::Status::internal(format!(
                                    "Error thrown by the orchestrator: {:?}",
                                    err
                                ))))
                                .unwrap();
                            }
                        };
                    }
                    Event::InstanceDestroy(id, tx) => {
                        info!("received instance destroy event : {:?}", id);

                        match orchestrator.lock().await.destroy_instance(id.clone()) {
                            Ok(_) => {
                                info!("destroyed instance : {:?}", id);
                                tx.send(Ok(Response::new(()))).unwrap();
                            }
                            Err(err) => {
                                info!("error while destroying instance : {:?}", err);
                                tx.send(Err(tonic::Status::internal(format!(
                                    "Error thrown by the orchestrator: {:?}",
                                    err
                                ))))
                                .unwrap();
                            }
                        };
                    }
                    Event::NodeRegister(request, tx) => {
                        info!("received node register event : {:?}", request);

                        // todo: parse certificate and get the node information
                        let node = Node {
                            id: Uuid::new_v4().to_string(),
                            status: Status::Starting,
                            resource: None,
                        };

                        debug!("registered node : {:?}", node);

                        match orchestrator.lock().await.register_node(node) {
                            Ok(_) => {
                                info!("successfully registered node");

                                let response = NodeRegisterResponse {
                                    code: 0,
                                    description: "Welcome to the cluster".to_string(),
                                    subnet: "".to_string(),
                                };

                                tx.send(Ok(tonic::Response::new(response))).unwrap();
                            }
                            Err(err) => {
                                info!("error while registering node : {:?}", err);

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
                        info!("received node unregister event : {:?}", request);

                        match orchestrator.lock().await.unregister_node(request.id) {
                            Ok(_) => {
                                info!("successfully unregistered node");

                                let response = NodeUnregisterResponse {
                                    code: 0,
                                    description: "Bye from the cluster".to_string(),
                                };

                                tx.send(Ok(tonic::Response::new(response))).unwrap();
                            }
                            Err(err) => {
                                info!("error while unregistering node : {:?}", err);

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
                        info!("received node status event : {:?}", status);

                        match orchestrator
                            .lock()
                            .await
                            .update_node_status(status.id.clone(), status)
                        {
                            Ok(_) => {
                                info!("successfully updated node status");
                                tx.send(Ok(())).await.unwrap();
                            }
                            Err(err) => {
                                info!("error while updating node status : {:?}", err);
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
        handlers.push(self.create_grpc_server(tx)?);

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
