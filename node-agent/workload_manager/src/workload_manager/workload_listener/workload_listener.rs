use proto::agent::{Instance, InstanceStatus};
use tokio::sync::mpsc::Sender;
use tonic::Status;

pub trait WorkloadListener {
    /// Launch a thread that will listen to a workload and send continously an InstanceStatus
    ///
    /// # Arguments
    ///
    /// * `id` - A String that is used by the workload's engine to identify it (docker containers' id for instance)
    /// * `instance` - An Instance struct, given by the scheduler,
    /// * `sender` - A sender given by the WorkloadManager, whose receiver is given to the scheduler
    fn run(id: String, instance: Instance, sender: Sender<Result<InstanceStatus, Status>>);
}
