use log::{info, debug};
use scheduler::manager::Manager;

fn main() {
    env_logger::init();

    info!("starting up");

    let scheduler = Manager::new();
    debug!("initialized manager struct with {:?}", scheduler);
    scheduler.run();

    info!("shutting down");
}
