use anyhow::{Context, Result};
use log::debug;
use reqwest::Method;
use serde::{Deserialize, Serialize};

use crate::{
    client::types::IdResponse,
    resource::{self, workload},
};

use super::request::{Client, RequestError};

/// Creates a workload in the cluster.
///
/// Returns the id of the workload.
pub async fn create(
    client: &Client,
    namespace: &str,
    workload: &workload::Workload,
) -> std::result::Result<String, RequestError> {
    let response: IdResponse = (*client)
        .send_json_request(
            format!("/workload/{}", namespace).as_str(),
            Method::PUT,
            Some(workload),
        )
        .await?;
    debug!("Workload {} created", response.id);
    Ok(response.id)
}

/// Updates a workload in the cluster.
///
/// Returns the id of the workload.
pub async fn update(
    client: &Client,
    namespace: &str,
    workload: &workload::Workload,
) -> std::result::Result<String, RequestError> {
    let response: IdResponse = (*client)
        .send_json_request(
            format!("/workload/{}/{}", namespace, workload.name).as_str(),
            Method::PATCH,
            Some(workload),
        )
        .await?;
    debug!("Workload {} updated", response.id);
    Ok(response.id)
}

/// Get info about a workload.
///
/// Returns the workload info.
pub async fn get(
    client: &Client,
    namespace: &str,
    workload_id: &str,
) -> Result<workload::Workload> {
    let response: workload::Workload = (*client)
        .send_json_request::<workload::Workload, ()>(
            &format!("/workload/{}/{}", namespace, workload_id),
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
pub async fn list(client: &Client, namespace: &str) -> Result<GetWorkloadResponse> {
    let response: GetWorkloadResponse = (*client)
        .send_json_request::<GetWorkloadResponse, ()>(
            format!("/workload/{}", namespace).as_str(),
            Method::GET,
            None,
        )
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
