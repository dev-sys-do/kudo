use std::{net::IpAddr, sync::Arc};

use log::{debug, info};
use proto::{
    controller::node_service_client::NodeServiceClient,
    scheduler::{Instance, InstanceStatus, NodeStatus, Resource, ResourceSummary, Status},
};
use tokio::sync::{mpsc, Mutex};
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
    InvalidWorkload,
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
        controller_client: Arc<Mutex<Option<NodeServiceClient<tonic::transport::Channel>>>>,
    ) -> Result<(), OrchestratorError> {
        let mut node_proxied = NodeProxied::new(node.id.clone(), node, addr);
        debug!("registering node: {:?}", node_proxied);

        // connect to node agent grpc service
        node_proxied
            .connect_to_grpc()
            .await
            .map_err(OrchestratorError::FromProxyError)?;

        node_proxied
            .open_node_status_stream(controller_client)
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

        let started_nodes: Vec<(_, &NodeProxied)> = nodes
            .iter()
            .filter(|node| node.1.node.status == Status::Running)
            .collect();

        if started_nodes.len() == 0 {
            debug!("No nodes available");
            return Err(OrchestratorError::NoAvailableNodes);
        }

        let mut best_node_id: Option<String> = None;
        let mut best_node_score: f64 = 0.0;

        for (_, node_proxied) in started_nodes {
            let node_score = match self
                .score_node_for_an_new_instance(node_proxied.node.clone(), instance.clone())
            {
                Ok(score) => score,
                Err(_) => continue,
            };

            if node_score > best_node_score {
                best_node_score = node_score;
                best_node_id = Some(node_proxied.id.clone());
            }
        }

        match best_node_id {
            Some(node_id) => {
                info!(
                    "scheduling instance {:?} on node: {:?}",
                    instance.id.clone(),
                    node_id
                );

                return Ok(node_id);
            }
            None => Err(OrchestratorError::NotEnoughResources),
        }
    }

    /// It takes a node and an instance and returns a score for the node based on the resources of the
    /// instance and the resources of the node. The score is a float between 0 and 1, with 1 being
    /// the best score.
    ///
    /// Arguments:
    ///
    /// * `node`: Node,
    /// * `instance`: Instance
    ///
    /// Returns:
    ///
    /// The score of the node for the new instance.
    fn score_node_for_an_new_instance(
        &self,
        node: Node,
        instance: Instance,
    ) -> Result<f64, OrchestratorError> {
        let instances_proxied = self.instances.get_all();

        let node_instances_proxied =
            instances_proxied
                .iter()
                .filter(|instance| match instance.1.node_id.clone() {
                    Some(id) => id == node.id,
                    None => false,
                });

        let mut sum_instances_resource_limit: ResourceSummary = match instance.resource {
            Some(resource) => match resource.limit {
                Some(limit) => ResourceSummary {
                    cpu: limit.cpu,
                    memory: limit.memory,
                    disk: limit.disk,
                },
                None => {
                    return Err(OrchestratorError::InvalidWorkload);
                }
            },
            None => return Err(OrchestratorError::InvalidWorkload),
        };

        for (_, instance_proxied) in node_instances_proxied {
            match instance_proxied.instance.resource.clone() {
                Some(resource) => match resource.limit {
                    Some(limit) => {
                        sum_instances_resource_limit.cpu += limit.cpu;
                        sum_instances_resource_limit.memory += limit.memory;
                        sum_instances_resource_limit.disk += limit.disk;
                    }
                    None => {}
                },
                None => {}
            }
        }

        let node_resource_summary_limit = match node.resource {
            Some(resource) => match resource.limit {
                Some(limit) => ResourceSummary {
                    cpu: limit.cpu,
                    memory: limit.memory,
                    disk: limit.disk,
                },
                None => {
                    return Err(OrchestratorError::InvalidWorkload);
                }
            },
            None => return Err(OrchestratorError::InvalidWorkload),
        };

        let mut node_score = 0.0;

        if Self::has_enough_resources(
            node_resource_summary_limit.clone(),
            sum_instances_resource_limit.clone(),
        ) {
            let cpu_score =
                sum_instances_resource_limit.cpu as f64 / node_resource_summary_limit.cpu as f64;

            let memory_score = sum_instances_resource_limit.memory as f64
                / node_resource_summary_limit.memory as f64;

            let disk_score =
                sum_instances_resource_limit.disk as f64 / node_resource_summary_limit.disk as f64;

            node_score = 1.0 - (cpu_score + memory_score + disk_score) / 3.0;
        }

        Ok(node_score)
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
    #[allow(dead_code)]
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

    /// "Returns true if the total resources limits of the instances are less than 95%
    /// of the node's resources limit"
    ///
    /// Arguments:
    ///
    /// * `available_resources`: ResourceSummary - The available resources on the node
    /// * `needed_resources`: ResourceSummary - The resources that the user wants to use
    ///
    /// Returns:
    ///
    /// A boolean value.
    fn has_enough_resources(
        available_resources: ResourceSummary,
        needed_resources: ResourceSummary,
    ) -> bool {
        available_resources.cpu as f64 * 0.95 >= needed_resources.cpu as f64
            && available_resources.memory as f64 * 0.95 >= needed_resources.memory as f64
            && available_resources.disk as f64 * 0.95 >= needed_resources.disk as f64
    }
}
