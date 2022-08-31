use std::sync::Arc;

use tokio::sync::Mutex;

use crate::{orchestrator::Orchestrator, InstanceIdentifier};

pub struct InstanceTerminatedHandler {}

impl InstanceTerminatedHandler {
    pub async fn handle(orchestrator: Arc<Mutex<Orchestrator>>, id: InstanceIdentifier) {
        log::info!("deleting instance after terminated/failed status: {:?}", id);

        orchestrator
            .lock()
            .await
            .delete_instance(id.clone(), None, false)
            .await
            .map_err(|err| {
                log::error!(
                    "error while deleting instance after terminated/failed status : {:?} ({:?})",
                    id,
                    err
                );
            })
            .ok();

        log::info!("deleted instance: {:?}", id);
    }
}
