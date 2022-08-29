use std::sync::Arc;

use crate::orchestrator::Orchestrator;
use anyhow::Result;
use proto::scheduler::NodeStatus;
use tokio::sync::{mpsc, Mutex};

pub struct NodeStatusHandler {}

impl NodeStatusHandler {
    pub async fn handle(
        orchestrator: Arc<Mutex<Orchestrator>>,
        status: NodeStatus,
        tx: mpsc::Sender<Result<(), tonic::Status>>,
    ) {
        match orchestrator
            .lock()
            .await
            .update_node_status(status.id.clone(), status.clone())
            .await
        {
            Ok(_) => {
                log::debug!("successfully updated node status : {:?}", status.id);

                tx.send(Ok(())).await.unwrap();
            }
            Err(err) => {
                log::info!(
                    "error while updating node status : {:?} ({:?})",
                    status.id,
                    err
                );

                tx.send(Err(tonic::Status::internal(format!(
                    "Error thrown by the orchestrator: {:?}",
                    err
                ))))
                .await
                .unwrap();
            }
        };
    }
}
