use super::container::Container;
use anyhow::Result;
use proto::agent::{Instance, Type};

pub struct NetworkSettings {
    pub node_id: String,
    pub bridge_name: String,
}

pub struct WorkloadRunner {
    network_settings: NetworkSettings,
}

impl WorkloadRunner {
    /// Create a new WorkloadRunner and returns it
    ///
    /// Arguments:
    ///
    /// * `node_id`: The ID of the node that the instance is running on.
    /// * `node_ip`: The IP address of the node that the instance is running on.
    ///
    /// Returns:
    ///
    /// A new instance of the WorkloadRunner struct.
    pub fn new(node_id: String) -> Self {
        WorkloadRunner {
            network_settings: NetworkSettings {
                node_id: node_id.clone(),
                bridge_name: network::utils::bridge_name(node_id),
            },
        }
    }

    /// run a workload (container) based on the given instance definition
    ///
    /// The `run` function is the entry point for the `Workload` trait. It is the function that is
    /// called when a user wants to run a workload
    ///
    /// Arguments:
    ///
    /// * `instance`: Instance - The instance of the workload to run.
    ///
    /// Returns:
    ///
    /// A future that resolves to a Container.
    pub async fn run(&self, instance: Instance) -> Result<Container> {
        match instance.r#type() {
            Type::Container => Container::new(instance, &self.network_settings).await,
        }
    }
}
