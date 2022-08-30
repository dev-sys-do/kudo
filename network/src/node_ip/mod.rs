use cidr::Ipv4Inet;
use std::net::Ipv4Addr;

pub struct NodeIp {
    pub cluster_ip_cidr: Ipv4Inet,
    pub external_ip_addr: Ipv4Addr,
}

impl NodeIp {
    pub fn new(cluster_ip_cidr: Ipv4Inet, external_ip_addr: Ipv4Addr) -> Self {
        Self {
            cluster_ip_cidr,
            external_ip_addr,
        }
    }
}
