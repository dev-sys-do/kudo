use std::sync::Arc;

use crate::orchestrator::Orchestrator;
use anyhow::Result;
use proto::scheduler::{Instance, InstanceStatus, Status};
use tokio::sync::{mpsc, Mutex};

pub struct InstanceCreateHandler {}

impl InstanceCreateHandler {
    pub async fn handle(
        orchestrator: Arc<Mutex<Orchestrator>>,
        instance: Instance,
        tx: mpsc::Sender<Result<InstanceStatus, tonic::Status>>,
    ) {
        log::info!("scheduling a new instance {:?} ...", instance.id);

        match orchestrator
            .lock()
            .await
            .create_instance(instance.clone(), tx.clone())
            .await
        {
            Ok(_) => {
                // todo: proxy the stream to the controller
            }
            Err(err) => {
                log::error!(
                    "error while scheduling instance : {:?} ({:?})",
                    instance.id,
                    err
                );

                let instance_status = InstanceStatus {
                    id: instance.id,
                    status: Status::Failed.into(),
                    status_description: format!("Error thrown by the orchestrator: {:?}", err),
                    resource: None,
                };

                let _ = tx.send(Ok(instance_status)).await;
            }
        }
    }
}
