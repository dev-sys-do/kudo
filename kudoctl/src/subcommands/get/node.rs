use super::output::{self, OutputFormat};
use crate::{
    client::{self, node::Node, request::Client},
    config,
};
use anyhow::{bail, Context, Result};
use std::fmt::Display;

/// get node <id> subcommand execution
/// Does the request, then formats the output.
pub async fn execute(
    conf: &config::Config,
    format: OutputFormat,
    search: Option<String>,
) -> Result<String> {
    if let Some(search) = search {
        let client = Client::new(conf).context("Error creating client")?;
        let result = client::node::get(&client, search.as_str()).await?;

        output::format_output(result, format)
    } else {
        bail!("You must provide an instance id");
    }
}

impl Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "id : {}\n", self.id)?;
        writeln!(f, "state : {}\n", self.status_description)?;

        let instances_str = self
            .instances
            .iter()
            .fold(String::new(), |acc, port| acc + &format!("{} ", port))
            .trim()
            .replace(' ', ",");
        writeln!(f, "ports : {} ", instances_str)?;

        // display resources

        writeln!(
            f,
            "resources : {}milliCPU, {}mB memory, {}GB disk ",
            self.resource.cpu, self.resource.memory, self.resource.disk
        )?;
        Ok(())
    }
}
