use proto::scheduler::{Resource, ResourceSummary};

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
