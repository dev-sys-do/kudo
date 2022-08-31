use std::sync::Arc;

use tokio::sync::Mutex;

use crate::{orchestrator::Orchestrator, NodeIdentifier};

pub struct NodeStreamCrashHandler {}

impl NodeStreamCrashHandler {
    pub async fn handle(orchestrator: Arc<Mutex<Orchestrator>>, id: NodeIdentifier) {
        log::info!("unregistering node after the stream crashed: {:?}", id);

        orchestrator
            .lock()
            .await
            .unregister_node(id.clone(), Some("Node stream crashed".to_string()))
            .await
            .map_err(|err| {
                log::error!(
                    "error while unregistering node after the stream crashed : {:?} ({:?})",
                    id,
                    err
                );
            })
            .ok();

        log::info!("unregistered node: {:?}", id);
    }
}
