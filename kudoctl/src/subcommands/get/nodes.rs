use super::output::{self, OutputFormat};
use crate::{
    client::{self, node::GetNodesResponse, request::Client},
    config,
};
use anyhow::{Context, Result};
use std::fmt::Display;

/// get nodes subcommand execution
/// Does the request, then formats the output.
pub async fn execute(
    conf: &config::Config,
    format: OutputFormat,
    show_header: bool,
) -> Result<String> {
    let client = Client::new(conf).context("Error creating client")?;
    let mut result = client::node::list(&client).await?;
    result.show_header = show_header;
    output::format_output(result, format)
}

impl Display for GetNodesResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.show_header {
            writeln!(f, "ID\tSTATUS\tINSTANCES")?;
        }

        for node in &self.nodes {
            writeln!(
                f,
                "{}\t{}\t{}\n",
                node.id,
                node.status_description,
                node.instances.len()
            )?;
        }
        Ok(())
    }
}
