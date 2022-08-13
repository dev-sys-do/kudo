use log::{debug, info};
use proto::scheduler::{
    instance_service_server::InstanceServiceServer, node_service_server::NodeServiceServer,
    Instance,
};
use tokio::task::JoinHandle;
use tonic::transport::Server;

use crate::{
    instance_listener::InstanceListener, node_listener::NodeListener, storage::Storage, Node,
};

#[derive(Debug)]
pub struct Manager {
    instances: Storage<Instance>,
    nodes: Storage<Node>,
}

impl Manager {
    /// `new` creates a new `Manager` struct with two empty `Storage` structs
    ///
    /// Returns:
    ///
    /// A new Manager struct
    pub fn new() -> Self {
        Manager {
            instances: Storage::new(),
            nodes: Storage::new(),
        }
    }

    /// This function returns a reference to the instances storage.
    ///
    /// Returns:
    ///
    /// A reference to the instances storage.
    pub fn get_instances_storage(&self) -> &Storage<Instance> {
        &self.instances
    }

    ///This function returns a reference to the nodes storage.
    ///
    /// Returns:
    ///
    /// A reference to the nodes storage.
    pub fn get_nodes_storage(&self) -> &Storage<Node> {
        &self.nodes
    }

    /// This function creates a gRPC server that listens on port 50051 and registers the
    /// `NodeServiceServer` with the server
    ///
    /// Returns:
    ///
    /// A JoinHandle<()>
    fn create_grpc_server(&self) -> JoinHandle<()> {
        info!("creating grpc server ...");

        let addr = "127.0.0.1:50051".parse().unwrap();
        let node_listener = NodeListener::default();
        debug!("create node listener with data : {:?}", node_listener);

        let instance_listener = InstanceListener::default();
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

    /// This function runs the scheduler.
    ///
    /// Returns:
    ///
    /// A vector of JoinHandles.
    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut handlers = vec![];

        // create listeners and serve the grpc server
        handlers.push(self.create_grpc_server());

        info!("scheduler running and ready to receive incoming requests ...");

        // wait the end of all the threads
        for handler in handlers {
            handler.await?;
        }

        Ok(())
    }
}
