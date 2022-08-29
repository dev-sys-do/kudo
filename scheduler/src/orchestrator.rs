use std::{net::IpAddr, sync::Arc};

use log::{debug, info};
use proto::scheduler::{Instance, InstanceStatus, NodeStatus, Resource, ResourceSummary, Status};
use tokio::sync::mpsc;
use tonic::Streaming;

use crate::{
    config::Config,
    instance::InstanceProxied,
    node::{Node, NodeProxied},
    storage::{IStorage, Storage},
    InstanceIdentifier, NodeIdentifier, ProxyError,
};

#[derive(Debug)]
pub enum OrchestratorError {
    NoAvailableNodes,
    NodeNotFound,
    InstanceNotFound,
    NetworkError(String),
    NotEnoughResources,
    FromProxyError(ProxyError),
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct Orchestrator {
    instances: Storage<InstanceProxied>,
    nodes: Storage<NodeProxied>,
    config: Arc<Config>,
}

impl Orchestrator {
    pub fn new(
        instances: Storage<InstanceProxied>,
        nodes: Storage<NodeProxied>,
        config: Arc<Config>,
    ) -> Self {
        Orchestrator {
            instances,
            nodes,
            config: config.clone(),
        }
    }

    /// It creates an instance
    ///
    /// Arguments:
    ///
    /// * `instance`: Instance - the instance to create
    /// * `tx`: mpsc::Sender<Result<InstanceStatus, tonic::Status>>
    ///
    /// Returns:
    ///
    /// A stream of InstanceStatus
    pub async fn create_instance(
        &mut self,
        instance: Instance,
        tx: mpsc::Sender<Result<InstanceStatus, tonic::Status>>,
    ) -> Result<Streaming<proto::agent::InstanceStatus>, OrchestratorError> {
        let mut instance_proxied = InstanceProxied::new(instance.id.clone(), instance, None, tx);

        // set instance status to Scheduling
        instance_proxied
            .change_status(Status::Scheduling, None)
            .await
            .map_err(OrchestratorError::FromProxyError)?;

        // find best node for the instance
        let target_node_id = self.find_best_node(&instance_proxied.instance)?;
        let target_node = self
            .nodes
            .get_mut(&target_node_id)
            .ok_or(OrchestratorError::NodeNotFound)?; // should never be None

        // create the instance on the node
        let stream = target_node
            .create_instance(instance_proxied.instance.clone())
            .await
            .map_err(OrchestratorError::FromProxyError)?;

        // set instance status to Scheduled
        instance_proxied
            .change_status(
                Status::Scheduled,
                Some(format!("Instance is scheduled on node: {}", target_node_id)),
            )
            .await
            .map_err(OrchestratorError::FromProxyError)?;

        // save the instance in the orchestrator
        self.instances
            .update(instance_proxied.id.clone().as_str(), instance_proxied);

        Ok(stream)
    }

    /// It stop an instance
    ///
    /// Arguments:
    ///
    /// * `id`: InstanceIdentifier
    ///
    /// Returns:
    ///
    /// A Result<(), OrchestratorError>
    pub async fn stop_instance(&mut self, id: InstanceIdentifier) -> Result<(), OrchestratorError> {
        let instance_proxied = self
            .instances
            .get_mut(&id)
            .ok_or(OrchestratorError::InstanceNotFound)?;

        // check if instance is already stopped
        if instance_proxied.instance.status() == Status::Stopped
            || instance_proxied.instance.status() == Status::Stopping
        {
            return Ok(());
        }

        // get the node where the instance is running
        let node_proxied = self
            .nodes
            .get_mut(&instance_proxied.node_id.clone().unwrap())
            .ok_or(OrchestratorError::NodeNotFound)?; // should never be None

        // send stop signal to the node
        node_proxied
            .stop_instance(instance_proxied.id.clone())
            .await
            .map_err(OrchestratorError::FromProxyError)
    }

    /// It destroys an instance
    ///
    /// Arguments:
    ///
    /// * `id`: InstanceIdentifier
    ///
    /// Returns:
    ///
    /// A Result<(), OrchestratorError>
    pub async fn destroy_instance(
        &mut self,
        id: InstanceIdentifier,
    ) -> Result<(), OrchestratorError> {
        let instance_proxied = self
            .instances
            .get_mut(&id)
            .ok_or(OrchestratorError::InstanceNotFound)?;

        // check if instance is already stopped
        if instance_proxied.instance.status() == Status::Destroying
            || instance_proxied.instance.status() == Status::Terminated
        {
            return Ok(());
        }

        // get the node where the instance is running
        let node_proxied = self
            .nodes
            .get_mut(&instance_proxied.node_id.clone().unwrap())
            .ok_or(OrchestratorError::NodeNotFound)?; // should never be None

        // send kill signal to the node
        node_proxied
            .kill_instance(instance_proxied.id.clone())
            .await
            .map_err(OrchestratorError::FromProxyError)
    }

    /// It registers a node with the orchestrator
    ///
    /// Arguments:
    ///
    /// * `node`: Node - the node to register
    /// * `addr`: IpAddr - the IP address of the node
    ///
    /// Returns:
    ///
    /// Result<(), OrchestratorError>
    pub async fn register_node(
        &mut self,
        node: Node,
        addr: IpAddr,
    ) -> Result<(), OrchestratorError> {
        let mut node_proxied = NodeProxied::new(node.id.clone(), node, addr);
        debug!("registering node: {:?}", node_proxied);

        // connect to node agent grpc service
        node_proxied
            .connect_to_grpc()
            .await
            .map_err(OrchestratorError::FromProxyError)?;

        // update node status as Starting
        node_proxied
            .update_status(Status::Starting, None, None)
            .await
            .map_err(OrchestratorError::FromProxyError)?;

        // save node in the orchestrator
        self.nodes.update(&node_proxied.id.clone(), node_proxied);
        Ok(())
    }

    pub fn unregister_node(&mut self, id: NodeIdentifier) -> Result<(), OrchestratorError> {
        // Return an error if the node is not found.
        self.nodes
            .get(&id.clone())
            .ok_or(OrchestratorError::NodeNotFound)?;

        // todo: get instance from node and change status to Destroyed

        self.nodes.delete(&id.clone());
        Ok(())
    }

    /// Update the status of a node in the orchestrator.
    ///
    /// The first thing we do is get the node from the `nodes` map. If the node is not found, we return
    /// an error else we update the node status.
    ///
    /// Arguments:
    ///
    /// * `id`: The identifier of the node to update.
    /// * `status`: The status of the node.
    ///
    /// Returns:
    ///
    /// A Result<(), OrchestratorError>
    pub async fn update_node_status(
        &mut self,
        id: NodeIdentifier,
        status: NodeStatus,
    ) -> Result<(), OrchestratorError> {
        // Return an error if the node is not found.
        let node_proxied = self
            .nodes
            .get_mut(&id.clone())
            .ok_or(OrchestratorError::NodeNotFound)?;

        node_proxied
            .update_status(
                status.status(),
                Some(status.status_description),
                status.resource,
            )
            .await
            .map_err(OrchestratorError::FromProxyError)
    }

    /// Find the best node for the given instance
    ///
    /// Arguments:
    ///
    /// * `instance`: The instance that we want to find a node for.
    ///
    /// Returns:
    ///
    /// A Result<NodeIdentifier, OrchestratorError>
    pub fn find_best_node(&self, instance: &Instance) -> Result<NodeIdentifier, OrchestratorError> {
        debug!("Finding best node for instance: {:?}", instance);

        let nodes = self.nodes.get_all();

        if nodes.len() == 0 {
            debug!("No nodes available");
            return Err(OrchestratorError::NoAvailableNodes);
        }

        for (_, node_proxied) in nodes {
            let has_enough_resources = match Self::has_enough_resources(
                node_proxied.node.resource.clone().unwrap(),
                instance.resource.clone().unwrap(),
            ) {
                Ok(_) => true,
                Err(_) => continue,
            };

            if has_enough_resources {
                info!(
                    "scheduling instance {:?} on node: {:?}",
                    instance.id.clone(),
                    node_proxied.node.id.clone()
                );

                return Ok(node_proxied.node.id.clone());
            }
        }

        Err(OrchestratorError::NotEnoughResources)
    }

    /// It takes a `Resource` struct, and returns a `ResourceSummary` struct
    ///
    /// Arguments:
    ///
    /// * `resource`: Resource - The resource object that we're going to compute the available resources
    /// for.
    ///
    /// Returns:
    ///
    /// A Result<ResourceSummary, OrchestratorError>
    fn compute_available_resources(
        resource: Resource,
    ) -> Result<ResourceSummary, OrchestratorError> {
        let available_limit = resource
            .limit
            .ok_or(OrchestratorError::NotEnoughResources)?;

        let available_usage = resource
            .usage
            .ok_or(OrchestratorError::NotEnoughResources)?;

        Ok(ResourceSummary {
            cpu: available_limit.cpu - available_usage.cpu,
            memory: available_limit.memory - available_usage.memory,
            disk: available_limit.disk - available_usage.disk,
        })
    }

    /// "If the needed resources are not defined, return an error. Otherwise, return true if the
    /// available resources are greater than or equal to the needed resources."
    ///
    /// The function is a bit more complicated than that, but that's the gist of it
    ///
    /// Arguments:
    ///
    /// * `available`: Resource - The available resources on the node
    /// * `needed`: The resources that the user wants to use
    ///
    /// Returns:
    ///
    /// A boolean value.
    fn has_enough_resources(
        available: Resource,
        needed: Resource,
    ) -> Result<bool, OrchestratorError> {
        let available_resources = Self::compute_available_resources(available)?;
        let needed_resources = needed.limit.ok_or(OrchestratorError::NotEnoughResources)?;

        Ok(available_resources.cpu >= needed_resources.cpu
            && available_resources.memory >= needed_resources.memory
            && available_resources.disk >= needed_resources.disk)
    }
}
