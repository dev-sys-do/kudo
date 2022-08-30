use serde::{Deserialize, Serialize};

use super::super::instance::model::Instance;

#[derive(Debug)]
pub enum NodeError {
    NodeNotFound,
    Etcd(String),
    SerdeError(serde_json::Error),
}

impl ToString for NodeError {
    fn to_string(&self) -> String {
        match self {
            NodeError::Etcd(err) => {
                format!("ETCD Error : {}", err)
            }
            NodeError::SerdeError(err) => {
                format!("Serde Error : {}", err)
            }
            &NodeError::NodeNotFound => "Node not found".to_string(),
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct NodeDTO {
    pub id: String,
}

#[derive(Deserialize, Serialize, Clone)]

pub struct ResourceSummary {
    pub cpu: i64,
    pub memory: i64,
    pub disk: i64,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Resource {
    pub usage: ResourceSummary,
    pub limit: ResourceSummary,
}

#[derive(Deserialize, Serialize, Clone)]
pub enum NodeState {
    REGISTERING = 0,
    REGISTERED = 1,
    UNREGISTERING = 2,
    UNREGISTERED = 3,
    FAILING = 4,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct NodeStatus {
    pub id: String,
    pub state: NodeState,
    pub status_description: String,
    pub resource: Resource,
    pub instances: Vec<Instance>,
}
