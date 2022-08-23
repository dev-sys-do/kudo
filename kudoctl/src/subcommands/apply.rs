use std::io::{self, Read};

use crate::{
    client::{self, request::Client},
    config,
    resource::Resource,
};
use anyhow::{bail, Context, Result};
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

pub async fn execute(args: Apply, conf: &config::Config) -> Result<String> {
    let client = Client::new(conf).context("Error creating client")?;

    // read the yaml file if -f is used
    let yaml = if let Some(file) = args.file {
        debug!("Reading file {}", file);

        std::fs::read_to_string(file.clone())
            .with_context(|| format!("Error reading file {}", file))?
    } else {
        // Read the yaml file from stdin otherwise

        let mut buffer = String::new();
        let mut stdin = io::stdin();

        stdin
            .read_to_string(&mut buffer)
            .context("Error reading stdin")?;

        if buffer.is_empty() {
            bail!("No yaml file provided, please use the -f option or pass the yaml file as stdin");
        }
        buffer
    };

    // parse the yaml file
    let resource_data: Resource =
        serde_yaml::from_str(&yaml).context("Error parsing file resource")?;

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
    Ok("".to_string())
}
