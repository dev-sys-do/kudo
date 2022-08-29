use super::output::{self, OutputFormat};
use crate::{
    client::{self, instance::GetInstancesResponse, request::Client},
    config,
};
use anyhow::{Context, Result};
use std::fmt::Display;

/// get instances subcommand execution
/// Does the request, then formats the output.
pub async fn execute(
    conf: &config::Config,
    format: OutputFormat,
    show_header: bool,
) -> Result<String> {
    let client = Client::new(conf).context("Error creating client")?;
    let mut result = client::instance::list(&client, &conf.namespace).await?;
    result.show_header = show_header;
    output::format_output(result, format)
}

impl Display for GetInstancesResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.show_header {
            writeln!(f, "ID\tSTATUS")?;
        }

        for inst in &self.instances {
            writeln!(f, "{}\t{}\n", inst.id, inst.status)?;
        }
        Ok(())
    }
}
