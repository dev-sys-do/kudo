use crate::{
    client::{self, request::Client},
    config,
    resource::Resource,
};
use anyhow::{Context, Result};
use clap::Args;
use log::{debug, info};

#[derive(Debug, Args)]
pub struct Apply {
    /// Read the informations from a yaml file
    #[clap(short, long)]
    file: Option<String>,

    /// If the resource already exists, don’t update it   
    #[clap(long)]
    no_update: bool,
}

pub async fn execute(args: Apply, conf: &config::Config) -> Result<()> {
    let client = Client::new(conf).context("Error creating client")?;

    // read the yaml file
    if args.file.is_none() {
        // Next : read the yaml file from stdin
        return Err(anyhow::Error::msg("No file specified"));
    }

    let file = args.file.unwrap();
    debug!("Reading file {}", file);

    let yaml = std::fs::read_to_string(file.clone())
        .with_context(|| format!("Error reading file {}", file))?;
    // parse the yaml file
    let resource_data: Resource =
        serde_yaml::from_str(&yaml).with_context(|| format!("Error parsing file {}", file))?;

    if args.no_update {
        // TODO: check if the workload already exists
    }

    match resource_data {
        Resource::Workload(workload) => {
            debug!("Creating workload {}", workload.name);

            let workload_id = client::workload::create(&client, &workload).await?;

            let instance_id = client::instance::create(&client, &workload_id).await?;

            info!(
                "Workload {} created with id {} and started with instance {}",
                workload.name, workload_id, instance_id
            );
        }
    }
    Ok(())
}
