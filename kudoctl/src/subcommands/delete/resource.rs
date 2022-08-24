use crate::{
    client::{self, request::Client},
    config,
};
use anyhow::{Context, Result};

/// Request deletion of a resource on the cluster.
pub async fn execute(conf: &config::Config, id: &str) -> Result<()> {
    let client = Client::new(conf).context("Error creating client")?;
    client::workload::delete(&client, id).await
}
