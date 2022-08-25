use cidr::Ipv4Inet;
use std::net::Ipv4Addr;

use crate::port::Port;

// Setup
pub struct SetupInstanceRequest {
    /// Unique identifier of the node. This identifier is used to find
    /// the node CNI
    pub node_id: String,
    /// Any ip to give to this node. Must be unique inside the cluster
    pub node_ip_addr: Ipv4Addr,
    /// Unique identifier of the instance. This identifier is used to create
    /// instance network interfaces
    pub instance_id: String,
    /// Composed of the instance ip and a mask
    pub instance_ip_cidr: Ipv4Inet,
    pub ports: Vec<Port>,
}

impl SetupInstanceRequest {
    pub fn new(
        node_id: String,
        node_ip_addr: Ipv4Addr,
        instance_id: String,
        instance_ip_cidr: Ipv4Inet,
        ports: Vec<Port>,
    ) -> Self {
        Self {
            node_id,
            node_ip_addr,
            instance_id,
            instance_ip_cidr,
            ports,
        }
    }
}

// Clean up
pub struct CleanInstanceRequest {
    /// Unique identifier of the instance. This identifier is used to find
    /// instance network interfaces and namespace
    pub instance_id: String,
    /// Exposed ports to be closed
    pub ports: Vec<Port>,
    /// Composed of the instance ip and a mask
    pub instance_ip_cidr: Ipv4Inet,
}

impl CleanInstanceRequest {
    pub fn new(instance_id: String, ports: Vec<Port>, instance_ip_cidr: Ipv4Inet) -> Self {
        Self {
            instance_id,
            ports,
            instance_ip_cidr,
        }
    }
}
