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
pub async fn create(
    client: &Client,
    namespace: &str,
    workload: &workload::Workload,
) -> Result<String> {
    let response: IdResponse = (*client)
        .send_json_request(
            format!("/workload/{}", namespace).as_str(),
            Method::PUT,
            Some(workload),
        )
        .await
        .context("Error creating workload")?;
    debug!("Workload {} created", response.id);
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
