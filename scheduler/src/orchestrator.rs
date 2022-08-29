use log::{info, debug};
use proto::scheduler::{Instance, NodeStatus, Status};

use crate::{storage::{Storage, IStorage}, Node, NodeIdentifier, InstanceIdentifier};

#[derive(Debug)]
pub enum OrchestratorError {
    NoAvailableNodes,
    NodeNotFound,
    InstanceNotFound,
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

    pub fn start_instance(&self, id: InstanceIdentifier) -> Result<(), OrchestratorError> {
        let instance = self.instances.get(&id).ok_or(OrchestratorError::InstanceNotFound)?;

        // check if instance is already running
        if instance.status() == Status::Running || instance.status() == Status::Starting {
            return Ok(());
        }

        // todo: start instance from node agent's api

        Ok(())
    }

    pub fn stop_instance(&self, id: InstanceIdentifier) -> Result<(), OrchestratorError> {
        let instance = self.instances.get(&id).ok_or(OrchestratorError::InstanceNotFound)?;

        // check if instance is already stopped
        if instance.status() == Status::Stopped || instance.status() == Status::Stopping {
            return Ok(());
        }

        // todo: stop instance from node agent's api

        Ok(())
    }

    pub fn destroy_instance(&self, id: InstanceIdentifier) -> Result<(), OrchestratorError> {
        let instance = self.instances.get(&id).ok_or(OrchestratorError::InstanceNotFound)?;

        // check if instance is already stopped
        if instance.status() == Status::Destroying || instance.status() == Status::Terminated {
            return Ok(());
        }

        // todo: destroy instance from node agent's api

        Ok(())
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

    pub fn update_node_status(&mut self, id: NodeIdentifier, status: NodeStatus) -> Result<(), OrchestratorError> {
        // Return an error if the node is not found.
        self.nodes.get(&id.clone()).ok_or(OrchestratorError::NodeNotFound)?;

        self.nodes.update(&id.clone(), Node {
            id: status.id
        });
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
