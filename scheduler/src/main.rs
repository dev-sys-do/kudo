use anyhow::Result;
use std::env;

use log::{debug, info};
use scheduler::{config::Config, manager::Manager, SchedulerError};

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    log::info!("starting up");

    info!("loading config");
    let mut dir = env::current_dir().map_err(SchedulerError::ConfigPathReadError)?; // get executable path
    dir.push("scheduler.conf"); // add config file name
    log::debug!("lookup configuration file at: {}", dir.display());

    // load config from path
    let config: Config =
        confy::load_path(dir.as_path()).map_err(SchedulerError::ConfigReadError)?;
    debug!("config: {:?}", config);

    let mut manager = Manager::new(config);
    log::trace!("initialized manager struct with data : {:?}", manager);

    manager.run().await?;

    log::info!("shutting down");
    Ok(())
}
