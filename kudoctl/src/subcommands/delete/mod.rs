use crate::config;
use anyhow::Result;
use clap::{Args, ValueEnum};
mod instance;
mod resource;

#[derive(Debug, Args)]
/// Delete a kudo subject, this is not reversible.
pub struct Subcommand {
    /// Define the type of element(s) to delete
    #[clap(arg_enum, value_parser)]
    subject: Subjects,

    /// Identifier of the element to delete
    #[clap(value_name = "ID")]
    id: String,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum Subjects {
    /// resources
    Resource,

    /// instances
    Instance,
}

/// match the subcommand to get the correct info
pub async fn execute(args: Subcommand, conf: &config::Config) -> Result<String> {
    match args.subject {
        Subjects::Resource => resource::execute(conf, args.id.as_str()).await,
        Subjects::Instance => instance::execute(conf, args.id.as_str()).await,
    }?;

    Ok(String::new())
}
