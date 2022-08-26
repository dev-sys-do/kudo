use super::output::{self, OutputFormat};
use crate::{
    client::{self, namespace::GetNamespacesResponse, request::Client},
    config,
};
use anyhow::{Context, Result};
use std::fmt::Display;

/// get namespaces subcommand execution
/// Does the request, then formats the output.
pub async fn execute(
    conf: &config::Config,
    format: OutputFormat,
    show_header: bool,
) -> Result<String> {
    let client = Client::new(conf).context("Error creating client")?;
    let mut result = client::namespace::list(&client).await?;
    result.show_header = show_header;
    output::format_output(result, format)
}

impl Display for GetNamespacesResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.show_header {
            writeln!(f, "NAMEI\tINSTANCES")?;
        }

        for node in &self.namespaces {
            writeln!(f, "{}\t{}", node.name, node.instances.len())?;
        }
        Ok(())
    }
}
