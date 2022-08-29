use proto::scheduler::{Instance, Status};

use super::{port::PortParser, resource::ResourceParser};

pub struct InstanceParser {}

impl InstanceParser {
    /// It converts a `Instance` struct to a `proto::agent::Instance` struct
    ///
    /// Arguments:
    ///
    /// * `instance`: Instance - The instance to convert
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

    /// It converts a `proto::agent::Instance` struct to a `Instance` struct
    ///
    /// Arguments:
    ///
    /// * `instance`: proto::agent::Instance
    ///
    /// Returns:
    ///
    /// An Instance struct
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

    /// It creates a fake agent instance
    ///
    /// Arguments:
    ///
    /// * `id`: The id of the agent instance.
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

    /// It creates a fake controller instance
    ///
    /// Arguments:
    ///
    /// * `id`: The id of the instance
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
