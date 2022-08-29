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

        self.nodes.update(&id.clone(), Node { id: status.id });
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

struct Transformer {}

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
