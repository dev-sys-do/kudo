pub mod container_listener;
#[allow(clippy::module_inception)]
pub mod workload_listener;

use proto::agent::{Instance, InstanceStatus, Type};
use tokio::sync::mpsc::Sender;
use tonic::Status;
use workload_listener::WorkloadListener;

use self::container_listener::ContainerListener;

pub fn create(id: String, instance: Instance, sender: Sender<Result<InstanceStatus, Status>>) {
    match instance.r#type() {
        Type::Container => ContainerListener::run(id, instance, sender),
    }
}
