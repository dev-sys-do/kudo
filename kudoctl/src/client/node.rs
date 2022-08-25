use super::request::Client;
use crate::resource::workload;
use anyhow::{Context, Result};
use log::debug;
use reqwest::Method;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Node {
    pub id: String,
    pub node_state: u32,
    pub status_description: String,
    pub resource: workload::Resources, // May need a refactor to move this type elsewhere
    pub instances: Vec<String>,        // instance ids
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GetNodesResponse {
    pub count: u64,
    pub nodes: Vec<Node>,
    #[serde(skip)]
    pub show_header: bool,
}

pub async fn list(client: &Client) -> Result<GetNodesResponse> {
    let response = (*client)
        .send_json_request::<GetNodesResponse, ()>("/node", Method::GET, None)
        .await
        .context("Error getting nodes")?;
    debug!(
        "{} total nodes, {} nodes received ",
        response.count,
        response.nodes.len()
    );
    Ok(response)
}
