use anyhow::{Context, Result};
use log::debug;
use reqwest::Method;
use serde::{Deserialize, Serialize};

use crate::client::request::check_count_exists_for_list;

use super::request::Client;

#[derive(Debug, Deserialize, Serialize)]
pub struct Namespace {
    pub name: String,
    pub instances: Vec<String>, // instance ids
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GetNamespacesResponse {
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
    let count = check_count_exists_for_list(&response)?;
    debug!(
        "{} total namespaces, {} namespaces received ",
        count,
        response.data.namespaces.len()
    );
    Ok(response.data)
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
