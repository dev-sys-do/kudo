use std::{sync::{Arc}};

use log::{info, debug};
use proto::scheduler::Instance;

use crate::{storage::{Storage, IStorage}, Node};

#[derive(Debug)]
pub enum OrchestratorError {
    NoAvailableNodes,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct Orchestrator {
    instances: Arc<Storage<Instance>>,
    nodes: Arc<Storage<Node>>,
}

impl Orchestrator {
    pub fn new(instances: Arc<Storage<Instance>>, nodes: Arc<Storage<Node>>) -> Self {
        Orchestrator {
            instances,
            nodes
        }
    }

    pub fn find_best_node(&self, instance: &Instance) -> Result<(), OrchestratorError> {
        debug!("Finding best node for instance: {:?}", instance);
        
        let nodes_storage = self.nodes.clone();
        let nodes = nodes_storage.get_all();

        if nodes.len() == 0 {
            debug!("No nodes available");
            return Err(OrchestratorError::NoAvailableNodes);
        }

        for (_, node) in self.nodes.clone().as_ref().get_all() {
            info!("{:?}", node);
        }

        Ok(())
    }

}
