use crate::workload_manager::workload_listener;
use futures::executor::{ block_on, block_on_stream };
use bollard::Docker;
use bollard::container::StatsOptions;
use bollard::errors::Error;
use bollard::models::{ ContainerStateStatusEnum, HealthStatusEnum };
use tonic::Status;
use proto::agent::{
    Instance,
    ResourceSummary,
    Resource,
    InstanceStatus,
    Status as WorkloadStatus
};
use std::sync::mpsc::Sender;
use std::thread;

#[derive(Clone)]
pub struct ContainerListener {
 //   container_id: String, 
    //sender: Sender<InstanceStatus>,
}


impl ContainerListener {

    pub fn new(container_id: String, instance: Instance, sender: Sender<InstanceStatus>) -> Result<Self, ()> { 
            
        thread::spawn(move || {
            #[cfg(unix)]
            let docker = Docker::connect_with_socket_defaults().unwrap();

            loop {
                let new_instance_status = Self::fetch_instance_status(container_id.as_str(), &instance, &docker);

                match new_instance_status {
                    Ok(instance_to_send) => sender.send(instance_to_send).unwrap(),
                    Err(_e) => break,
                };
            }
            
        });

        Ok(Self{})

    }


    /**
     * Get instance data via inspect_container() and stats() then returns an InstanceStatus
     */
    pub fn fetch_instance_status(container_id: &str, instance: &Instance, docker_connection: &Docker) -> Result<InstanceStatus, Error> {
        let stats_options = Some(StatsOptions{
            stream: false,
            one_shot: true,
        });

        let container_data = block_on(docker_connection.inspect_container(container_id, None))?;
        let container_resources = block_on_stream(docker_connection.stats(container_id, stats_options)).next().unwrap()?;
        let container_health = container_data.clone().state.unwrap().health.unwrap().status.unwrap();
        let container_status = container_data.state.unwrap().status.unwrap();

        // WorkloadStatus::Failed occurs when the container hasn't even started
        let mut workload_status = match container_status {
            //ContainerStateStatusEnum::CREATED => ,
            ContainerStateStatusEnum::RUNNING => WorkloadStatus::Running,
            //ContainerStateStatusEnum::PAUSED => ,
            //ContainerStateStatusEnum::RESTARTING => ,
            //ContainerStateStatusEnum::EMPTY => ,
            ContainerStateStatusEnum::REMOVING => WorkloadStatus::Destroying,
            ContainerStateStatusEnum::EXITED => WorkloadStatus::Terminated,
            ContainerStateStatusEnum::DEAD => WorkloadStatus::Crashed,
            _ => WorkloadStatus::Scheduled
        };

        workload_status = match container_health {
            HealthStatusEnum::STARTING => WorkloadStatus::Starting,
            _ => workload_status
        };

        Ok(InstanceStatus {
            id: instance.clone().id,
            status: workload_status as i32,
            description: "heres a description".to_string() ,
            resource: Some(Resource {
                limit: Some(ResourceSummary { 
                    cpu: instance.clone().resource.unwrap().limit.unwrap().cpu,
                    memory: instance.clone().resource.unwrap().limit.unwrap().memory,
                    disk: instance.clone().resource.unwrap().limit.unwrap().disk
                }),
                usage: Some(ResourceSummary { //check if these are good units
                    cpu: container_resources.cpu_stats.cpu_usage.usage_in_usermode as i32, 
                    memory: container_resources.memory_stats.usage.unwrap() as i32, 
                    disk: container_resources.storage_stats.read_count_normalized.unwrap() as i32
                }),
            }), 
        })
    }

}

impl workload_listener::workload_listener::WorkloadListener for ContainerListener {}