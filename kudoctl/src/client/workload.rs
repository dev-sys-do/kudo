use anyhow::{Context, Result};
use log::debug;
use reqwest::Method;
use serde::{Deserialize, Serialize};

use crate::{
    client::types::IdResponse,
    resource::{self, workload},
};

use super::request::Client;

/// Creates a workload in the cluster.
///
/// Returns the id of the workload.
pub async fn create(client: &Client, workload: &workload::Workload) -> Result<String> {
    let response: IdResponse = (*client)
        .send_json_request("/workload", Method::PUT, Some(workload))
        .await
        .context("Error creating workload")?;
    debug!("Workload {} created", response.id);
    Ok(response.id)
}

/// Get info about a workload.
///
/// Returns the workload info.
pub async fn get(client: &Client, workload_id: &str) -> Result<workload::Workload> {
    let response: workload::Workload = (*client)
        .send_json_request::<workload::Workload, ()>(
            &format!("/workload/{}", workload_id),
            Method::GET,
            None,
        )
        .await
        .context("Error getting workload")?;
    Ok(response)
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GetWorkloadResponse {
    pub count: u64,
    pub workloads: Vec<resource::Resource>,
    #[serde(skip)]
    pub show_header: bool,
}

/// Get the workloads in the cluster.
///
/// Returns a vector of workloads.
pub async fn list(client: &Client) -> Result<GetWorkloadResponse> {
    let response: GetWorkloadResponse = (*client)
        .send_json_request::<GetWorkloadResponse, ()>("/workload", Method::GET, None)
        .await
        .context("Error getting workloads")?;
    debug!(
        "{} total workloads, {} workloads received ",
        response.count,
        response.workloads.len()
    );
    Ok(response)
}

/// Delete a workload.
pub async fn delete(client: &Client, id: &str) -> Result<()> {
    let response = (*client)
        .send_json_request::<IdResponse, ()>(
            format!("/workload/{}", id).as_str(),
            Method::DELETE,
            None,
        )
        .await
        .context("Error deleting workload")?;
    debug!("Workload {} deleted", response.id);
    Ok(())
}
