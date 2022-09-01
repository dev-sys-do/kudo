use super::request::Client;
use crate::{client::request::check_count_exists_for_list, resource::workload};
use anyhow::{Context, Result};
use log::debug;
use reqwest::Method;
use serde::{Deserialize, Serialize};

/// Representation of a node of the cluster.
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
    pub nodes: Vec<Node>,
    #[serde(skip)]
    pub show_header: bool,
}

/// Get the nodes connected to the cluster controller.
pub async fn list(client: &Client) -> Result<GetNodesResponse> {
    let response = (*client)
        .send_json_request::<GetNodesResponse, ()>("/node", Method::GET, None)
        .await
        .context("Error getting nodes")?;

    let count = check_count_exists_for_list(&response)?;

    debug!(
        "{} total nodes, {} nodes received ",
        count,
        response.data.nodes.len()
    );
    Ok(response.data)
}

/// Get the node with the given id.
pub async fn get(client: &Client, node_id: &str) -> Result<Node> {
    let response = (*client)
        .send_json_request::<Node, ()>(&format!("/node/{}", node_id), Method::GET, None)
        .await
        .context("Error getting node")?
        .data;
    debug!("Node {} received", response.id);
    Ok(response)
}
