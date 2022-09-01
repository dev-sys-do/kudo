use anyhow::Context;
use log::debug;
use reqwest::Method;
use serde::{Deserialize, Serialize};

use crate::{
    client::{request::check_count_exists_for_list, types::IdResponse},
    resource::workload,
};

use super::request::Client;

#[derive(Debug, Serialize)]
struct CreateRequestBody {
    pub workload_name: String,
}

/// Starts an instance on the cluster.
///
/// Returns the id of the instance.
pub async fn create(client: &Client, namespace: &str, workload_id: &str) -> anyhow::Result<String> {
    let response: IdResponse = (*client)
        .send_json_request(
            &format!("/instance/{}", namespace),
            Method::PUT,
            Some(&CreateRequestBody {
                workload_name: workload_id.to_owned(),
            }),
        )
        .await
        .context("Error creating instance")?
        .data;
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
    pub instances: Vec<Instance>,

    /// used for formatting in the Display impl
    #[serde(skip)]
    pub show_header: bool,
}

/// List the instances in the cluster.
pub async fn list(client: &Client, namespace: &str) -> anyhow::Result<GetInstancesResponse> {
    let response = (*client)
        .send_json_request::<GetInstancesResponse, ()>(
            format!("/instance/{}", namespace).as_str(),
            Method::GET,
            None,
        )
        .await
        .context("Error getting instances")?;

    let count = check_count_exists_for_list(&response)?;

    debug!(
        "{} total instances, {} instances received ",
        count,
        response.data.instances.len()
    );
    Ok(response.data)
}

/// Get info about one instance.
pub async fn get(client: &Client, namespace: &str, instance_id: &str) -> anyhow::Result<Instance> {
    let response: Instance = (*client)
        .send_json_request::<Instance, ()>(
            &format!("/instance/{}/{}", namespace, instance_id),
            Method::GET,
            None,
        )
        .await
        .context("Error getting instance")?
        .data;
    debug!("Instance {} received", response.id);
    Ok(response)
}

/// Delete an instance with the given id.
pub async fn delete(client: &Client, namespace: &str, instance_id: &str) -> anyhow::Result<()> {
    (*client)
        .send_json_request::<(), ()>(
            &format!("/instance/{}/{}", namespace, instance_id),
            Method::DELETE,
            None,
        )
        .await
        .context("Error deleting instance")?;
    debug!("Instance {} deleted", instance_id);
    Ok(())
}
