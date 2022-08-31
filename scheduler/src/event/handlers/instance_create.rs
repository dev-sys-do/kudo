use std::sync::Arc;

use crate::{
    event::Event, orchestrator::Orchestrator, parser::port::StatusParser, InstanceIdentifier,
};
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
        tx_events: mpsc::Sender<Event>,
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
                    Self::proxy_status_to_controller(instance.id, stream, tx, tx_events).await;
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
        id: InstanceIdentifier,
        stream: Streaming<proto::agent::InstanceStatus>,
        tx: mpsc::Sender<Result<InstanceStatus, tonic::Status>>,
        tx_events: mpsc::Sender<Event>,
    ) {
        let mut stream = stream;
        let mut last_status = None;

        while let Ok(message) = stream.message().await {
            match message {
                Some(status) => {
                    last_status = Some(status.status());
                    let _ = tx
                        .send(Ok(StatusParser::from_agent_instance_status(status)))
                        .await;
                }
                None => {
                    break;
                }
            }
        }

        // streaming to controller has finished
        log::debug!(
            "instance status stream to controller for {:?} has finished",
            id
        );

        // check if the stream has finished with an error
        if last_status != Some(proto::agent::Status::Terminated)
            && last_status != Some(proto::agent::Status::Failed)
        {
            // send the instance stream crash event to the manager
            tx_events.send(Event::InstanceStreamCrash(id)).await.ok();
            return;
        }

        // send the instance terminated event to the manager
        tx_events.send(Event::InstanceTerminated(id)).await.ok();
    }
}
