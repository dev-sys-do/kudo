use anyhow::{Context, Result};
use log::debug;
use reqwest::Method;
use serde::{Deserialize, Serialize};

use super::request::Client;

#[derive(Debug, Deserialize, Serialize)]
pub struct Namespace {
    pub name: String,
    pub instances: Vec<String>, // instance ids
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GetNamespacesResponse {
    pub count: u64,
    pub namespaces: Vec<Namespace>,
    #[serde(skip)]
    pub show_header: bool,
}

/// Get the list of namespaces
pub async fn list(client: &Client) -> Result<GetNamespacesResponse> {
    let response = (*client)
        .send_json_request::<GetNamespacesResponse, ()>("/namespace", Method::GET, None)
        .await
        .context("Error getting nodes")?;
    debug!(
        "{} total namespaces, {} namespaces received ",
        response.count,
        response.namespaces.len()
    );
    Ok(response)
}

/// Delete an namespace with the given id.
pub async fn delete(client: &Client, name: &str) -> anyhow::Result<()> {
    (*client)
        .send_json_request::<(), ()>(&format!("/namespace/{}", name), Method::DELETE, None)
        .await
        .context("Error deleting namespace")?;
    debug!("Namespace {} deleted", name);
    Ok(())
}
