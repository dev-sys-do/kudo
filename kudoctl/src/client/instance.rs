use anyhow::Context;
use log::debug;
use reqwest::Method;
use serde::{Deserialize, Serialize};

use crate::{client::types::IdResponse, resource::workload};

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

#[derive(Debug, Deserialize, Serialize)]
pub struct Instance {
    pub id: String,
    pub name: String,
    pub r#type: String,
    pub uri: String,
    pub ports: Vec<String>,
    pub env: Vec<String>,
    pub resources: workload::Resources,
    pub status: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GetInstancesResponse {
    pub count: u64,
    pub instances: Vec<Instance>,

    /// used for formatting in the Display impl
    #[serde(skip)]
    pub show_header: bool,
}

/// List the instances in the cluster.
pub async fn list(client: &Client) -> anyhow::Result<GetInstancesResponse> {
    let response: GetInstancesResponse = (*client)
        .send_json_request::<GetInstancesResponse, ()>("/instance", Method::GET, None)
        .await
        .context("Error getting instances")?;
    debug!(
        "{} total instances, {} instances received ",
        response.count,
        response.instances.len()
    );
    Ok(response)
}
