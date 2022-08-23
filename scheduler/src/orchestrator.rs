use log::{info, debug};
use proto::scheduler::Instance;

use crate::{storage::{Storage, IStorage}, Node, NodeIdentifier};

#[derive(Debug)]
pub enum OrchestratorError {
    NoAvailableNodes,
    NodeNotFound,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct Orchestrator {
    instances: Storage<Instance>,
    nodes: Storage<Node>,
}

impl Orchestrator {
    pub fn new(instances: Storage<Instance>, nodes: Storage<Node>) -> Self {
        Orchestrator {
            instances,
            nodes
        }
    }

    pub fn register_node(&mut self, node: Node) -> Result<(), OrchestratorError> {
        self.nodes.update(&node.id.clone(), node);
        Ok(())
    }

    pub fn unregister_node(&mut self, id: NodeIdentifier) -> Result<(), OrchestratorError> {
        // Return an error if the node is not found.
        self.nodes.get(&id.clone()).ok_or(OrchestratorError::NodeNotFound)?;

        self.nodes.delete(&id.clone());
        Ok(())
    }

    pub fn find_best_node(&self, instance: &Instance) -> Result<(), OrchestratorError> {
        debug!("Finding best node for instance: {:?}", instance);
        
        let nodes = self.nodes.get_all();

        if nodes.len() == 0 {
            debug!("No nodes available");
            return Err(OrchestratorError::NoAvailableNodes);
        }

        for (_, node) in nodes {
            info!("{:?}", node);
        }

        Ok(())
    }

}
