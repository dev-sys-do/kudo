use proto::scheduler::{InstanceStatus, NodeStatus, Port};

use super::resource::ResourceParser;

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
            id: status.id,
            state: status.status,
            status_description: status.status_description,
            resource: status.resource.map(ResourceParser::to_controller_resource),
            instances: vec![], // todo;
        }
    }
}
