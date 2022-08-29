use crate::{
    client::{self, request::Client, workload::GetWorkloadResponse},
    config,
};
use anyhow::{Context, Result};
use std::fmt::Display;

use super::output::{self, OutputFormat}; // import without risk of name clashing

/// get resources subcommand execution
pub async fn execute(
    conf: &config::Config,
    format: OutputFormat,
    show_header: bool,
) -> Result<String> {
    let client = Client::new(conf).context("Error creating client")?;
    let mut result = client::workload::list(&client, &conf.namespace).await?;
    result.show_header = show_header;
    output::format_output(result, format)
}

impl Display for GetWorkloadResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.show_header {
            writeln!(f, "NAME\tTYPE")?;
        }

        for r in &self.workloads {
            writeln!(f, "{}\tWorkload\n", r.name)?;
        }
        Ok(())
    }
}
