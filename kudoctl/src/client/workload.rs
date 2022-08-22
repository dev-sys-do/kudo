use anyhow::{Context,Result};
use log::debug;
use reqwest::Method;

use crate::{client::types::IdResponse, resource::workload};

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
