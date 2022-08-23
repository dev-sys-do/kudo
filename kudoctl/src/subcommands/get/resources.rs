use crate::{
    client::{self, request::Client, workload::GetWorkloadResponse},
    config, resource,
};
use anyhow::{Context, Result};
use clap::Args;
use std::fmt::Display;

use super::output::{self, OutputFormat}; // import without risk of name clashing

#[derive(Debug, Args)]
pub struct GetResources {}

/// get resources subcommand execution
pub async fn execute(
    conf: &config::Config,
    format: OutputFormat,
    show_header: bool,
) -> Result<String> {
    let client = Client::new(conf).context("Error creating client")?;
    let mut result = client::workload::list(&client).await?;
    result.show_header = show_header;
    output::format_output(result, format)
}

impl Display for GetWorkloadResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.show_header {
            writeln!(f, "NAME\tTYPE")?;
        }

        for r in &self.workloads {
            match r {
                resource::Resource::Workload(workload) => {
                    writeln!(f, "{}\tWorkload\n", workload.name)?;
                }
            }
        }
        Ok(())
    }
}
