use log::{debug, info};
use scheduler::manager::Manager;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    info!("starting up");

    let manager = Manager::new();
    debug!("initialized manager struct with data : {:?}", manager);

    manager.run().await?;

    info!("shutting down");
    Ok(())
}
