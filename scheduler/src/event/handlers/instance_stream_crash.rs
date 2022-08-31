use std::sync::Arc;

use tokio::sync::Mutex;

use crate::{orchestrator::Orchestrator, InstanceIdentifier};

pub struct InstanceStreamCrashHandler {}

impl InstanceStreamCrashHandler {
    pub async fn handle(orchestrator: Arc<Mutex<Orchestrator>>, id: InstanceIdentifier) {
        log::info!("deleting instance after the stream crashed: {:?}", id);

        orchestrator
            .lock()
            .await
            .delete_instance(
                id.clone(),
                Some("instance status stream crashed".to_string()),
                true,
            )
            .await
            .map_err(|err| {
                log::error!(
                    "error while deleting instance after the stream crashed : {:?} ({:?})",
                    id,
                    err
                );
            })
            .ok();

        log::info!("deleted instance: {:?}", id);
    }
}
