use std::collections::HashMap;
use tokio::sync::mpsc::Sender;

mod workload;
use workload::container::Container;
use workload::workload_runner::WorkloadRunner;

use workload::workload_trait::Workload;

mod workload_listener;
use workload_listener::create as create_workload_listener;

use proto::agent::{Instance, InstanceStatus, SignalInstruction, Status as WorkloadStatus};

use log::{debug, info};

use tonic::Status;

pub type StreamSender = Sender<Result<InstanceStatus, Status>>;

#[derive(Default)]
pub struct WorkloadManager {
    workloads: HashMap<String, Container>,
    senders: HashMap<String, StreamSender>,
    node_id: String,
}

impl WorkloadManager {
    /// Creates an empty WorkloadManager
    pub fn new(node_id: String) -> Self {
        info!("Creating WorkloadManager");
        WorkloadManager{
            node_id,
            workloads: HashMap::new(),
            ..WorkloadManager::default()
        }
    }

    /// Creates a Workload, run it and starts its listener, a receiver is returned to read all Workloads' status
    ///
    /// # Arguments
    /// * `instance` - Respresentation of instance to create
    pub async fn create(&mut self, instance: Instance, sender: StreamSender) -> Result<(), Status> {
        let workload_id = instance.clone().id;

        //Create a workload and it's listener
        let runner = WorkloadRunner::new(self.node_id.clone());

        let workload = match runner.run(instance.clone()).await {
            Ok(wrkld) => wrkld,
            Err(e) => return Err(Status::internal(e.to_string())),
        };
        info!("Workload {} created", workload_id);

        //create listener from the workloadId;
        create_workload_listener(workload.id(), instance, sender.clone());
        info!("Workload {} listener created", workload_id);

        self.workloads.insert(workload_id.clone(), workload);
        
        sender.send(Ok(InstanceStatus{ 
            id: workload_id.clone(),
            status: WorkloadStatus::Starting as i32,
            ..InstanceStatus::default() }
        )).await.unwrap_or(());

        self.senders.insert(workload_id, sender);

        Ok(())
    }

    /// Send a signal to a Workload
    ///
    /// # Arguments
    /// * `signal_instruction` - Respresentation of signal to send
    pub async fn signal(&mut self, signal_instruction: SignalInstruction) -> Result<(), Status> {
        let workload_id = match signal_instruction.instance {
            Some(inst) => inst.id,
            None => return Err(Status::invalid_argument("Please provide an 'Instance'")),
        };

        let workload = match self.workloads.get(&workload_id.clone()) {
            None => return Err(Status::not_found("This workload does not exist")),
            Some(wrkld) => wrkld,
        };

        let status_stopping = InstanceStatus {
            id: workload_id.clone(),
            status: WorkloadStatus::Stopping as i32,
            ..Default::default()
        };

        // TODO: remove unwrap
        info!("Sending signal to workload");
        self.senders
            .get(&workload_id.clone())
            .unwrap()
            .send(Ok(status_stopping))
            .await.unwrap_or(());

        let status_destroying = InstanceStatus {
            id: workload_id.clone(),
            status: WorkloadStatus::Destroying as i32,
            ..Default::default()
        };

        let status_terminated = InstanceStatus {
            id: workload_id.clone(),
            status: WorkloadStatus::Terminated as i32,
            ..Default::default()
        };

        let sender = self.senders
        .get(&workload_id.clone())
        .unwrap();

        match signal_instruction.signal {
            // Status::Stop
            0 => {
                debug!("stopping workload");
                let promised = workload.stop();
                sender.send(Ok(status_destroying)).await.unwrap_or(());
                promised.await.unwrap_or(());
                info!("workload stopped and destroyed");
            }
            // Status::Kill
            1 => {
                debug!("killing workload");
                let promised = workload.kill();
                sender.send(Ok(status_destroying)).await.unwrap_or(());
                promised.await.unwrap_or(());
                info!("workload killed and destroyed");
            }
            _ => return Err(Status::not_found("This signal does not exist")),
        };

        sender.send(Ok(status_terminated)).await.unwrap_or(());

        debug!("removing workload from manager");
        self.senders.remove(&workload_id.clone());
        self.workloads.remove(&workload_id);

        Ok(())
    }
}

// #[cfg(test)]
// mod tests {
//     use std::collections::HashMap;
//     use std::sync::{ Arc, Mutex };

//     #[test]
//     fn constructor() {
//         let wm = super::WorkloadManager::new();

//         assert_eq!(wm.workloads.capacity(), 0);
//     }

//     #[tokio::test]
//     async fn create() {
//         let mut wm = super::WorkloadManager::new();

//         let instance = proto::agent::Instance {
//             id: "someuuid".to_string(),
//             name: "somename".to_string(),
//             r#type: proto::agent::Type::Container as i32,
//             status: proto::agent::Status::Running as i32,
//             uri: "debian:latest".to_string(),
//             environment: vec!["A=0".to_string()],
//             resource: Some(proto::agent::Resource {
//                 limit: Some(proto::agent::ResourceSummary {
//                     cpu: u64::MAX,
//                     memory: u64::MAX,
//                     disk: u64::MAX,
//                 }),
//                 usage: Some(proto::agent::ResourceSummary {
//                     cpu: 0,
//                     memory: 0,
//                     disk: 0,
//                 }),
//             }),
//             ports: vec![],
//             ip: "127.0.0.1".to_string(),
//         };

//         let rx = wm.create(instance).await.unwrap();

//         // uncomment this when workloads will be merged
//         // let received = rx.recv().unwrap();
//         // assert!(received.resource.unwrap().usage.unwrap().cpu >= 0);

//         //println!("{:?}", wm.workloads.keys());
//         assert_eq!(wm.workloads.len(), 1);
//     }

// #[tokio::test]
// async fn signal() {
//     let mut wm = super::WorkloadManager::new();

//     let instance = proto::agent::Instance {
//         id: "someuuid".to_string(),
//         name: "somename".to_string(),
//         r#type: proto::agent::Type::Container as i32,
//         status: proto::agent::Status::Running as i32,
//         uri: "debian:latest".to_string(),
//         environment: vec!["A=0".to_string()],
//         resource: Some(proto::agent::Resource {
//             limit: Some(proto::agent::ResourceSummary {
//                 cpu: i32::MAX,
//                 memory: i32::MAX,
//                 disk: i32::MAX,
//             }),
//             usage: Some(proto::agent::ResourceSummary {
//                 cpu: 0,
//                 memory: 0,
//                 disk: 0,
//             }),
//         }),
//         ports: vec![],
//         ip: "127.0.0.1".to_string(),
//     };

//     let signal = proto::agent::SignalInstruction {
//         instance: Some(proto::agent::Instance {
//             id: "someuuid".to_string(),
//             name: "somename".to_string(),
//             r#type: proto::agent::Type::Container as i32,
//             ..Default::default()
//         }),
//         signal: proto::agent::Signal::Kill as i32,
//     };

//     let tx = wm.tx.clone();
//     let rx = wm.create(instance).await.unwrap().lock().unwrap();

//     let (_to_replace, not_used_rx) = std::sync::mpsc::channel();

//     let hshmpwrkld: HashMap<String, Box<dyn crate::workload_manager::Workload>> =
//         std::collections::HashMap::new();

//     // cannot borrow `wm` as mutable more than once at a timesecond mutable borrow occurs here
//     let mut wm2 = crate::workload_manager::WorkloadManager {
//         workloads: hshmpwrkld,
//         tx,
//         rx: Arc::new(Mutex::new(not_used_rx)),
//     };

//     wm2.signal(signal).await.unwrap();

//     for _ in 0..2 {
//         let recv = rx.recv().unwrap().unwrap();
//         assert!(
//             recv.status == proto::agent::Status::Stopping as i32
//                 || recv.status == proto::agent::Status::Destroying as i32
//         );
//     }

// }
// }
