use log::{info, debug};
use scheduler::manager::Manager;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    info!("starting up");

    let manager = Manager::new();
    debug!("initialized manager struct with data : {:?}", manager);
    
    let handlers = manager.run()?;
    info!("started");

    // wait the end of all the threads
    for handler in handlers {
        handler.await?;
    }

    info!("shutting down");
    Ok(())
}
