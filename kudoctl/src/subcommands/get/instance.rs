use super::output::{self, OutputFormat};
use crate::{
    client::{self, instance::Instance, request::Client},
    config,
};
use anyhow::{bail, Context, Result};
use std::fmt::Display;

/// get instance <id> subcommand execution
/// Does the request, then formats the output.
pub async fn execute(
    conf: &config::Config,
    format: OutputFormat,
    search: Option<String>,
) -> Result<String> {
    if search.is_none() {
        bail!("You must provide an instance id");
    }
    let search = search.unwrap();

    let client = Client::new(conf).context("Error creating client")?;
    let result = client::instance::get(&client, &conf.namespace, search.as_str()).await?;

    output::format_output(result, format)
}

impl Display for Instance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "id : {}\n", self.id)?;
        writeln!(f, "workload name : {}\n", self.name)?;
        writeln!(f, "status : {}\n", self.status)?;
        writeln!(f, "type : {}\n", self.r#type)?;
        writeln!(f, "uri : {}\n", self.uri)?;

        // display ports
        let ports_str = self
            .ports
            .iter()
            .fold(String::new(), |acc, port| acc + &format!("{} ", port))
            .trim()
            .replace(' ', ",")
            .replace(':', "->");
        writeln!(f, "ports : {} ", ports_str)?;

        // display environment variables
        let env_vars_str = self
            .env
            .iter()
            .fold(String::new(), |acc, env_var| acc + &format!("{} ", env_var))
            .trim()
            .replace(' ', ",");
        writeln!(f, "env variables : {} ", env_vars_str)?;

        // display resources

        writeln!(
            f,
            "resources : {}milliCPU, {}mB memory, {}GB disk ",
            self.resources.cpu, self.resources.memory, self.resources.disk
        )?;
        Ok(())
    }
}
