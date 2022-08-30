use std::net::SocketAddr;

use crate::{etcd::EtcdClient, external_api::generic::filter::FilterService};

use super::model::{NodeError, NodeStatus};

pub struct NodeService {
    pub etcd_service: EtcdClient,
    pub filter_service: FilterService,
}

impl NodeService {
    pub async fn new(etcd_address: SocketAddr) -> Result<Self, NodeError> {
        let etcd_service = EtcdClient::new(etcd_address.to_string())
            .await
            .map_err(|err| NodeError::Etcd(err.to_string()))?;

        Ok(Self {
            etcd_service,
            filter_service: FilterService::new(),
        })
    }

    /// It gets a node from etcd, and if it exists return it to the caller.
    ///
    /// # Arguments:
    ///
    /// * `node_id`: The node id to get
    ///
    /// # Returns:
    ///
    /// A Result<NodeStatus, NodeError>

    pub async fn get_node(&mut self, node_id: &str) -> Result<NodeStatus, NodeError> {
        match self.etcd_service.get(node_id).await {
            Some(node) => {
                let node_status: NodeStatus = serde_json::from_str(&node)
                    .map_err(NodeError::SerdeError)
                    .unwrap();
                Ok(node_status)
            }
            None => Err(NodeError::NodeNotFound),
        }
    }

    /// This function get all nodes from etcd and slice the result by limit and offset
    /// If there is an error , the function always return an empty vector
    /// # Arguments:
    ///
    /// * `limit`: The number of nodes to return.
    /// * `offset`: The offset of the nodes to be returned
    ///
    /// # Returns:
    ///
    /// A vector of nodes

    pub async fn get_all_nodes(&mut self, limit: u32, offset: u32) -> Vec<NodeStatus> {
        let mut nodes = Vec::new();
        match self.etcd_service.get_all().await {
            Some(keys) => {
                for node in keys {
                    if let Ok(node) = serde_json::from_str(&node) {
                        nodes.push(node);
                    }
                }
                if offset > 0 {
                    match self.filter_service.offset(&nodes, offset) {
                        Ok(new_nodes) => nodes = new_nodes,
                        Err(_) => return Vec::new(),
                    }
                }
                if limit > 0 {
                    nodes = self.filter_service.limit(&nodes, limit);
                }
                nodes
            }
            None => nodes,
        }
    }
}
