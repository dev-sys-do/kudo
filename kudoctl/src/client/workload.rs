use anyhow::{Context, Result};
use log::{debug, warn};
use reqwest::Method;
use serde::{Deserialize, Serialize};

use crate::{
    client::types::IdResponse,
    resource::workload::{self, Resources},
};

use super::request::{Client, RequestError};

/// Ports binding for the workload
#[derive(Debug, Deserialize, Serialize)]
pub struct PortBinding {
    pub source: i32,
    pub destination: i32,
}

/// Workload, as stored in the controller
#[derive(Debug, Deserialize, Serialize)]
pub struct WorkloadBody {
    pub name: String,
    pub uri: String,
    pub environment: Vec<String>,
    pub resources: Resources,
    pub ports: Vec<PortBinding>,
}

/// Creates a workload in the cluster.
///
/// Returns the id of the workload.
pub async fn create(
    client: &Client,
    namespace: &str,
    workload: &workload::Workload,
) -> std::result::Result<String, RequestError> {
    let workload_body = WorkloadBody {
        name: workload.name.clone(),
        uri: workload.uri.clone(),
        environment: workload.env.as_deref().unwrap_or_default().to_vec(),
        resources: workload.resources.to_owned(),
        ports: workload
            .ports
            .to_owned()
            .unwrap_or_default()
            .into_iter()
            .map(|p| {
                // parse the port binding from the string

                let mut splitted = p.split(':');
                let source = splitted.next().unwrap_or(&p).parse::<i32>().unwrap_or(0);

                if source == 0 {
                    warn!("Invalid port binding: {}", p);
                }

                let destination = if let Some(s) = splitted.nth(1) {
                    debug!("No destination port specified, using source port");
                    s.parse::<i32>().unwrap_or(source)
                } else {
                    source
                };

                PortBinding {
                    source,
                    destination,
                }
            })
            .collect(),
    };

    let response: IdResponse = (*client)
        .send_json_request(
            format!("/workload/{}", namespace).as_str(),
            Method::PUT,
            Some(&workload_body),
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
pub async fn get(client: &Client, namespace: &str, workload_id: &str) -> Result<WorkloadBody> {
    let response: WorkloadBody = (*client)
        .send_json_request::<WorkloadBody, ()>(
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
    pub workloads: Vec<WorkloadBody>,
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
