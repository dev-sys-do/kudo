use cidr::Ipv4Inet;

use crate::node_ip::NodeIp;

// Setup
pub struct SetupNodeRequest {
    /// Unique identifier of the node. This identifier is used to create a bridge interface
    pub node_id: String,
    /// Composed of the node ip and a mask
    pub node_ip_cidr: Ipv4Inet,
    /// Nodes already in the cluster
    pub nodes_ips: Vec<NodeRequest>,
}

impl SetupNodeRequest {
    pub fn new(node_id: String, node_ip_cidr: Ipv4Inet, nodes_ips: Vec<NodeRequest>) -> Self {
        Self {
            node_id,
            node_ip_cidr,
            nodes_ips,
        }
    }
}

pub struct SetupIptablesRequest {
    pub node_id: String,
}

impl SetupIptablesRequest {
    pub fn new(node_id: String) -> Self {
        Self { node_id }
    }
}

pub struct NodeRequest {
    pub node_id: String,
    pub nodes_ips: NodeIp,
}

impl NodeRequest {
    pub fn new(node_id: String, nodes_ips: NodeIp) -> Self {
        Self { node_id, nodes_ips }
    }
}

// Clean up
pub struct CleanNodeRequest {
    pub node_id: String,
}

impl CleanNodeRequest {
    pub fn new(node_id: String) -> Self {
        Self { node_id }
    }
}
