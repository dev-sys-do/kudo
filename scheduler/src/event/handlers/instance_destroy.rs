use std::sync::Arc;

use crate::{orchestrator::Orchestrator, InstanceIdentifier};
use anyhow::Result;
use tokio::sync::{oneshot, Mutex};
use tonic::Response;

pub struct InstanceDestroyHandler {}

impl InstanceDestroyHandler {
    pub async fn handle(
        orchestrator: Arc<Mutex<Orchestrator>>,
        id: InstanceIdentifier,
        tx: oneshot::Sender<Result<Response<()>, tonic::Status>>,
    ) {
        match orchestrator.lock().await.destroy_instance(id.clone()).await {
            Ok(_) => {
                log::info!("destroyed instance : {:?}", id);

                tx.send(Ok(Response::new(()))).unwrap();
            }
            Err(err) => {
                log::error!("error while destroying instance : {:?} ({:?})", id, err);

                tx.send(Err(tonic::Status::internal(format!(
                    "Error thrown by the orchestrator: {:?}",
                    err
                ))))
                .unwrap();
            }
        };
    }
}
