use proto::scheduler::{InstanceStatus, NodeStatus, Status};

use super::resource::ResourceParser;

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
            id: status.id.clone(),
            state: Self::to_node_state_controller(status.status()).into(),
            status_description: status.status_description,
            resource: status.resource.map(ResourceParser::to_controller_resource),
            instances: vec![], // todo;
        }
    }

    /// It converts a `Status` enum to a `NodeState` enum
    ///
    /// Arguments:
    ///
    /// * `status`: The status of the node.
    pub fn to_node_state_controller(status: Status) -> proto::controller::NodeState {
        match status {
            Status::Starting => proto::controller::NodeState::Registering,
            Status::Running => proto::controller::NodeState::Registered,
            Status::Stopping => proto::controller::NodeState::Unregistering,
            Status::Stopped => proto::controller::NodeState::Unregistered,
            Status::Terminated => proto::controller::NodeState::Unregistered,
            _ => proto::controller::NodeState::Failing,
        }
    }
}
