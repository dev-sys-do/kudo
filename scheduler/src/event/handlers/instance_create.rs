use std::sync::Arc;

use crate::{orchestrator::Orchestrator, parser::port::StatusParser};
use anyhow::Result;
use proto::scheduler::{Instance, InstanceStatus, Status};
use tokio::sync::{mpsc, Mutex};
use tonic::Streaming;

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
            Ok(stream) => {
                tokio::spawn(async move {
                    log::debug!(
                        "starting instance status stream to controller for {:?}",
                        instance.id
                    );
                    Self::proxy_status_to_controller(stream, tx).await;
                });
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

    /// It takes a stream of messages from the agent, and sends them to the controller
    ///
    /// Arguments:
    ///
    /// * `stream`: Streaming<proto::agent::InstanceStatus>
    /// * `tx`: mpsc::Sender<Result<InstanceStatus, tonic::Status>>
    async fn proxy_status_to_controller(
        stream: Streaming<proto::agent::InstanceStatus>,
        tx: mpsc::Sender<Result<InstanceStatus, tonic::Status>>,
    ) {
        let mut stream = stream;
        loop {
            let message = stream.message().await.unwrap();
            match message {
                Some(status) => {
                    let _ = tx
                        .send(Ok(StatusParser::from_agent_instance_status(status)))
                        .await;
                }
                None => {
                    return;
                }
            }
        }
    }
}
