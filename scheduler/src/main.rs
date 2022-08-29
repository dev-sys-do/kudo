use std::env;

use log::{debug, info};
use scheduler::{config::Config, manager::Manager, SchedulerError};

#[tokio::main]
async fn main() -> Result<(), ManagerError> {
    env_logger::init();
    info!("starting up");

    info!("loading config");
    let mut dir = env::current_dir().map_err(SchedulerError::ConfigPathReadError)?; // get executable path
    dir.push("scheduler.conf"); // add config file name

    // load config from path
    let config: Config =
        confy::load_path(dir.as_path()).map_err(SchedulerError::ConfigReadError)?;
    debug!("config: {:?}", config);

    let mut manager = Manager::new(config);
    debug!("initialized manager struct with data : {:?}", manager);

    manager.run().await?;

    info!("shutting down");
    Ok(())
}
