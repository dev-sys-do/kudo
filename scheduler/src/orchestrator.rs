use std::sync::Arc;

use log::{debug, info};
use proto::{
    agent::{self, instance_service_client::InstanceServiceClient},
    scheduler::{Instance, NodeStatus, Port, Resource, ResourceSummary, Status},
};

use crate::{
    config::Config,
    storage::{IStorage, Storage},
    InstanceIdentifier, Node, NodeIdentifier,
};

#[derive(Debug)]
pub enum OrchestratorError {
    NoAvailableNodes,
    NodeNotFound,
    InstanceNotFound,
    NetworkError(String),
    NotEnoughResources,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct Orchestrator {
    instances: Storage<Instance>,
    nodes: Storage<Node>,
    config: Arc<Config>,
}

impl Orchestrator {
    pub fn new(instances: Storage<Instance>, nodes: Storage<Node>, config: Arc<Config>) -> Self {
        Orchestrator {
            instances,
            nodes,
            config,
        }
    }

    pub async fn start_instance(&self, id: InstanceIdentifier) -> Result<(), OrchestratorError> {
        let instance = self
            .instances
            .get(&id)
            .ok_or(OrchestratorError::InstanceNotFound)?;

        // check if instance is already running
        if instance.status() == Status::Running || instance.status() == Status::Starting {
            return Ok(());
        }

        let config = self.config.clone();

        let mut client = match InstanceServiceClient::connect(format!(
            "{}:{}",
            config.agent.host, config.agent.port
        ))
        .await
        {
            Ok(client) => client,
            Err(_) => {
                return Err(OrchestratorError::NetworkError(
                    "Could not connect to node agent".to_string(),
                ));
            }
        };

        let request = tonic::Request::new(Transformer::scheduler_instance_to_agent_instance(
            instance.clone(),
        ));

        let response = match client.create(request).await {
            Ok(response) => response,
            Err(_) => {
                return Err(OrchestratorError::NetworkError(
                    "Could not connect to node agent".to_string(),
                ));
            }
        };

        println!("RESPONSE={:?}", response);

        Ok(())
    }

    pub fn stop_instance(&self, id: InstanceIdentifier) -> Result<(), OrchestratorError> {
        let instance = self
            .instances
            .get(&id)
            .ok_or(OrchestratorError::InstanceNotFound)?;

        // check if instance is already stopped
        if instance.status() == Status::Stopped || instance.status() == Status::Stopping {
            return Ok(());
        }

        // todo: stop instance from node agent's api

        Ok(())
    }

    pub fn destroy_instance(&self, id: InstanceIdentifier) -> Result<(), OrchestratorError> {
        let instance = self
            .instances
            .get(&id)
            .ok_or(OrchestratorError::InstanceNotFound)?;

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
        self.nodes
            .get(&id.clone())
            .ok_or(OrchestratorError::NodeNotFound)?;

        self.nodes.delete(&id.clone());
        Ok(())
    }

    pub fn update_node_status(
        &mut self,
        id: NodeIdentifier,
        status: NodeStatus,
    ) -> Result<(), OrchestratorError> {
        // Return an error if the node is not found.
        self.nodes
            .get(&id.clone())
            .ok_or(OrchestratorError::NodeNotFound)?;

        self.nodes.update(
            &id.clone(),
            Node {
                id: status.id,
                status: Status::Running,
                resource: status.resource,
            },
        );
        Ok(())
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

        for (_, node) in nodes {
            let has_enough_resources = match Self::has_enough_resources(
                node.resource.clone().unwrap(),
                instance.resource.clone().unwrap(),
            ) {
                Ok(_) => true,
                Err(_) => continue,
            };

            if has_enough_resources {
                info!("Instance will be scheduled on node: {:?}", node.id.clone());
                return Ok(node.id.clone());
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

struct Transformer {}

#[allow(dead_code)]
impl Transformer {
    pub fn scheduler_instance_to_agent_instance(instance: Instance) -> agent::Instance {
        agent::Instance {
            id: instance.id,
            name: instance.name,
            r#type: instance.r#type,
            status: instance.status.into(),
            uri: instance.uri,
            environment: instance.environnement,
            resource: match instance.resource {
                Some(r) => Some(Self::scheduler_resource_to_agent_resource(r)),
                None => None,
            },
            ports: Self::scheduler_ports_to_agent_ports(instance.ports),
            ip: instance.ip,
        }
    }

    pub fn agent_instance_to_scheduler_instance(instance: agent::Instance) -> Instance {
        Instance {
            id: instance.id,
            name: instance.name,
            r#type: instance.r#type,
            status: instance.status.into(),
            uri: instance.uri,
            environnement: instance.environment,
            resource: match instance.resource {
                Some(r) => Some(Self::agent_resource_to_scheduler_resource(r)),
                None => None,
            },
            ports: Self::agent_ports_to_scheduler_ports(instance.ports),
            ip: instance.ip,
        }
    }

    pub fn scheduler_resource_to_agent_resource(resource: Resource) -> agent::Resource {
        agent::Resource {
            limit: match resource.limit {
                Some(r) => Some(Self::scheduler_resourcesummary_to_agent_resourcesummary(r)),
                None => None,
            },
            usage: match resource.usage {
                Some(r) => Some(Self::scheduler_resourcesummary_to_agent_resourcesummary(r)),
                None => None,
            },
        }
    }

    pub fn agent_resource_to_scheduler_resource(resource: agent::Resource) -> Resource {
        Resource {
            limit: match resource.limit {
                Some(r) => Some(Self::agent_resourcesummary_to_scheduler_resourcesummary(r)),
                None => None,
            },
            usage: match resource.usage {
                Some(r) => Some(Self::agent_resourcesummary_to_scheduler_resourcesummary(r)),
                None => None,
            },
        }
    }

    pub fn scheduler_resourcesummary_to_agent_resourcesummary(
        resource: ResourceSummary,
    ) -> agent::ResourceSummary {
        agent::ResourceSummary {
            cpu: resource.cpu,
            memory: resource.memory,
            disk: resource.disk,
        }
    }

    pub fn agent_resourcesummary_to_scheduler_resourcesummary(
        resource: agent::ResourceSummary,
    ) -> ResourceSummary {
        ResourceSummary {
            cpu: resource.cpu,
            memory: resource.memory,
            disk: resource.disk,
        }
    }

    pub fn scheduler_ports_to_agent_ports(ports: Vec<Port>) -> Vec<agent::Port> {
        ports
            .into_iter()
            .map(|port| agent::Port {
                source: port.source,
                destination: port.destination,
            })
            .collect()
    }

    pub fn agent_ports_to_scheduler_ports(ports: Vec<agent::Port>) -> Vec<Port> {
        ports
            .into_iter()
            .map(|port| Port {
                source: port.source,
                destination: port.destination,
            })
            .collect()
    }
}
