use log::info;
use proto::scheduler::Instance;

use crate::{storage::Storage, Node};

#[derive(Debug)]
pub struct Manager {
    instances: Storage<Instance>,
    nodes: Storage<Node>
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
            nodes: Storage::new()
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

    pub fn run(&self) {
        info!("scheduler running and ready to receive incoming requests ...");
    }
}
