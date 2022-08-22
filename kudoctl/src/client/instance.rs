use anyhow::Context;
use log::debug;
use reqwest::Method;

use crate::client::types::IdResponse;

use super::request::Client;

/// Starts an instance on the cluster.
///
/// Returns the id of the instance.
pub async fn create(client: &Client, workload_id: &String) -> anyhow::Result<String> {
    let response: IdResponse = (*client)
        .send_json_request::<IdResponse, ()>(
            &format!("/instance/?workloadId={}", workload_id),
            Method::PUT,
            None,
        )
        .await
        .context("Error creating instance")?;
    debug!("Instance {} created", response.id);
    Ok(response.id)
}
