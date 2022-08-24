mod instances;
mod output;
mod resources;
use self::output::OutputFormat;
use crate::config;
use anyhow::{bail, Result};
use clap::{Args, ValueEnum};

#[derive(Debug, Args)]
pub struct GetSubcommand {
    /// Change the output format
    #[clap(short = 'F', long, arg_enum, value_parser)]
    format: Option<OutputFormat>,

    /// Don’t show header (human readable only)
    #[clap(long)]
    no_header: bool,

    /// Define the type of element(s) to get
    ///
    /// Add an id after the type to get the element.
    /// Use plural to get all the elements of the type,
    /// use singular to get only one element of the type (ID parameter is required).
    #[clap(arg_enum, value_parser)]
    subject: GetSubjects,

    /// Search for a specific element
    #[clap(value_name = "ID")]
    id: Option<String>,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum GetSubjects {
    /// resources
    Resources,
    Resource,

    /// instances
    Instances,
    Instance,
}

/// match the subcommand to get the correct info
pub async fn execute(args: GetSubcommand, conf: &config::Config) -> Result<String> {
    let format = args.format.unwrap_or(OutputFormat::HumanReadable);
    let show_header = !args.no_header;

    match args.subject {
        GetSubjects::Resources => resources::execute(conf, format, show_header).await,
        GetSubjects::Instances => instances::execute(conf, format, show_header).await,
        GetSubjects::Resource | GetSubjects::Instance => {
            bail!(format!("{:?} not implemented yet", args.subject))
        }
    }
}
