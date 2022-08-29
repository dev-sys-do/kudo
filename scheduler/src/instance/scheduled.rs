use proto::scheduler::{Instance, InstanceStatus, Status};
use tokio::sync::mpsc;

use crate::{NodeIdentifier, ProxyError};

/// InstanceScheduled represents an instance that is scheduled to a node. It is used to send
/// messages to the node and contains the node identifier where it's scheduled.
///
/// Properties:
///
/// * `id`: The id of the instance.
/// * `instance`: The instance that is being registered.
/// * `node_id`: The node identifier of the node that the instance is running on.
/// * `tx`: This is the channel that the instance will use to send status updates to the controller.
#[derive(Debug, Clone)]
pub struct InstanceScheduled {
    pub id: String,
    pub instance: Instance,
    pub node_id: Option<NodeIdentifier>,
    pub tx: mpsc::Sender<Result<InstanceStatus, tonic::Status>>,
}

impl InstanceScheduled {
    /// `new` creates a new `InstanceStatus` struct
    ///
    /// Arguments:
    ///
    /// * `id`: The id of the instance.
    /// * `instance`: The instance that we want to run.
    /// * `node_id`: The node identifier of the node that the instance is running on.
    /// * `tx`: This is the channel that the instance will use to send status updates to the controller.
    ///
    /// Returns:
    ///
    /// A new instance of the struct `InstanceStatus`
    pub fn new(
        id: String,
        instance: Instance,
        node_id: Option<NodeIdentifier>,
        tx: mpsc::Sender<Result<InstanceStatus, tonic::Status>>,
    ) -> Self {
        Self {
            id,
            instance,
            node_id,
            tx,
        }
    }

    /// This function updates the status of the instance and sends the updated status to the controller
    ///
    /// Arguments:
    ///
    /// * `status`: The status of the node.
    /// * `description`: A string that describes the status of the node.
    ///
    /// Returns:
    ///
    /// A Result<(), ProxyError>
    pub async fn change_status(
        &mut self,
        status: Status,
        description: Option<String>,
    ) -> Result<(), ProxyError> {
        self.instance.status = status.into();

        self.tx
            .send(Ok(InstanceStatus {
                id: self.id.clone(),
                status: status.into(),
                status_description: description.unwrap_or_else(|| "".to_string()),
                resource: match self.instance.status() {
                    Status::Running => self.instance.resource.clone(),
                    _ => None,
                },
            }))
            .await
            .map_err(|_| ProxyError::ChannelSenderError)?;

        Ok(())
    }
}
