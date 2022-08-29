use proto::scheduler::{
    Instance, InstanceStatus, NodeStatus, Port, Resource, ResourceSummary, Status,
};
use uuid::Uuid;

pub struct NodeParser {}

impl NodeParser {}

pub struct InstanceParser {}

impl InstanceParser {
    pub fn to_agent_instance(instance: Instance) -> proto::agent::Instance {
        proto::agent::Instance {
            id: instance.id,
            name: instance.name,
            r#type: instance.r#type,
            status: instance.status,
            uri: instance.uri,
            environment: instance.environnement,
            resource: instance.resource.map(ResourceParser::to_agent_resource),
            ports: PortParser::to_agent_ports(instance.ports),
            ip: instance.ip,
        }
    }

    pub fn from_agent_instance(instance: proto::agent::Instance) -> Instance {
        Instance {
            id: instance.id,
            name: instance.name,
            r#type: instance.r#type,
            status: instance.status,
            uri: instance.uri,
            environnement: instance.environment,
            resource: instance.resource.map(ResourceParser::from_agent_resource),
            ports: PortParser::from_agent_ports(instance.ports),
            ip: instance.ip,
        }
    }

    pub fn fake_agent_instance(id: String) -> proto::agent::Instance {
        proto::agent::Instance {
            id,
            name: "".to_string(),
            r#type: proto::agent::Type::Container.into(),
            status: Status::Stopping.into(),
            uri: "".to_string(),
            environment: vec![],
            resource: None,
            ports: vec![],
            ip: "".to_string(),
        }
    }

    pub fn fake_controller_instance(id: String) -> proto::controller::Instance {
        proto::controller::Instance {
            id,
            name: "".to_string(),
            r#type: proto::agent::Type::Container.into(),
            state: Status::Stopping.into(),
            uri: "".to_string(),
            environnement: vec![],
            resource: None,
            ports: vec![],
            ip: "".to_string(),
        }
    }
}

pub struct ResourceParser {}

impl ResourceParser {
    /// It converts a Resource struct to a proto::agent::Resource struct.
    ///
    /// Arguments:
    ///
    /// * `resource`: Resource
    ///
    /// Returns:
    ///
    /// A proto::agent::Resource struct
    pub fn to_agent_resource(resource: Resource) -> proto::agent::Resource {
        proto::agent::Resource {
            limit: resource.limit.map(Self::to_agent_resourcesummary),
            usage: resource.usage.map(Self::to_agent_resourcesummary),
        }
    }

    /// It converts a Resource struct to a proto::controller::Resource struct.
    ///
    /// Arguments:
    ///
    /// * `resource`: Resource
    ///
    /// Returns:
    ///
    /// A proto::controller::Resource struct
    pub fn to_controller_resource(resource: Resource) -> proto::controller::Resource {
        proto::controller::Resource {
            limit: resource.limit.map(Self::to_controller_resourcesummary),
            usage: resource.usage.map(Self::to_controller_resourcesummary),
        }
    }

    /// It converts a proto::agent::Resource struct to a Resource struct.
    ///
    /// Arguments:
    ///
    /// * `resource`: proto::agent::Resource
    ///
    /// Returns:
    ///
    /// A Resource struct
    pub fn from_agent_resource(resource: proto::agent::Resource) -> Resource {
        Resource {
            limit: resource.limit.map(Self::from_agent_resourcesummary),
            usage: resource.usage.map(Self::from_agent_resourcesummary),
        }
    }

    /// It converts a ResourceSummary to a proto::agent::ResourceSummary struct.
    ///
    /// Arguments:
    ///
    /// * `resource`: ResourceSummary
    ///
    /// Returns:
    ///
    /// A proto::agent::ResourceSummary struct
    pub fn to_agent_resourcesummary(resource: ResourceSummary) -> proto::agent::ResourceSummary {
        proto::agent::ResourceSummary {
            cpu: resource.cpu,
            memory: resource.memory,
            disk: resource.disk,
        }
    }

    /// It converts a ResourceSummary to a proto::agent::ResourceSummary struct.
    ///
    /// Arguments:
    ///
    /// * `resource`: ResourceSummary
    ///
    /// Returns:
    ///
    /// A proto::agent::ResourceSummary struct
    pub fn to_controller_resourcesummary(
        resource: ResourceSummary,
    ) -> proto::controller::ResourceSummary {
        proto::controller::ResourceSummary {
            cpu: resource.cpu,
            memory: resource.memory,
            disk: resource.disk,
        }
    }

    /// It converts a proto::agent::ResourceSummary to a ResourceSummary struct.
    ///
    /// Arguments:
    ///
    /// * `resource`: proto::agent::ResourceSummary
    ///
    /// Returns:
    ///
    /// A ResourceSummary struct
    pub fn from_agent_resourcesummary(resource: proto::agent::ResourceSummary) -> ResourceSummary {
        ResourceSummary {
            cpu: resource.cpu,
            memory: resource.memory,
            disk: resource.disk,
        }
    }
}

pub struct PortParser {}

impl PortParser {
    /// `ports` is a vector of `Port`s, and we want to convert it into a vector of `proto::agent::Port`s
    ///
    /// Arguments:
    ///
    /// * `ports`: Vec<Port>
    ///
    /// Returns:
    ///
    /// A vector of proto::agent::Port
    pub fn to_agent_ports(ports: Vec<Port>) -> Vec<proto::agent::Port> {
        ports
            .into_iter()
            .map(|port| proto::agent::Port {
                source: port.source,
                destination: port.destination,
            })
            .collect()
    }

    /// `from_agent_ports` takes a vector of `proto::agent::Port`s and returns a vector of `Port`s
    ///
    /// Arguments:
    ///
    /// * `ports`: Vec<proto::agent::Port>
    ///
    /// Returns:
    ///
    /// A vector of Port structs.
    pub fn from_agent_ports(ports: Vec<proto::agent::Port>) -> Vec<Port> {
        ports
            .into_iter()
            .map(|port| Port {
                source: port.source,
                destination: port.destination,
            })
            .collect()
    }
}

pub struct StatusParser {}

impl StatusParser {
    /// It takes a `proto::agent::InstanceStatus` and returns an `InstanceStatus`
    ///
    /// Arguments:
    ///
    /// * `status`: proto::agent::InstanceStatus
    ///
    /// Returns:
    ///
    /// A new InstanceStatus struct
    pub fn from_agent_instance_status(status: proto::agent::InstanceStatus) -> InstanceStatus {
        InstanceStatus {
            id: status.id,
            status: status.status,
            status_description: status.description,
            resource: status.resource.map(ResourceParser::from_agent_resource),
        }
    }

    /// It takes a `NodeStatus` struct and returns a `proto::controller::NodeStatus` struct
    ///
    /// Arguments:
    ///
    /// * `status`: NodeStatus - this is the status of the node.
    pub fn to_controller_node_status(status: NodeStatus) -> proto::controller::NodeStatus {
        proto::controller::NodeStatus {
            id: Uuid::new_v4().to_string(),
            state: status.status,
            status_description: status.status_description,
            resource: status.resource.map(ResourceParser::to_controller_resource),
            instances: vec![], // todo;
        }
    }
}
