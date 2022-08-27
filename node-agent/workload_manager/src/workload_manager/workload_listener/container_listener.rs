use crate::workload_manager::workload_listener::workload_listener::WorkloadListener;

use futures::StreamExt;

use bollard::container::{CPUStats, StatsOptions};
use bollard::errors::Error;
use bollard::models::ContainerStateStatusEnum;
use bollard::Docker;

use tokio::runtime::Runtime;
use tonic::Status;

use proto::agent::{Instance, InstanceStatus, Resource, ResourceSummary, Status as WorkloadStatus};

use sysinfo::{System, SystemExt};
use tokio::sync::mpsc::Sender;

use log::{debug, info};

fn convert_status(state: Option<ContainerStateStatusEnum>) -> WorkloadStatus {
    match state.unwrap_or(ContainerStateStatusEnum::RUNNING) {
        ContainerStateStatusEnum::RUNNING => WorkloadStatus::Running,
        ContainerStateStatusEnum::REMOVING => WorkloadStatus::Destroying,
        ContainerStateStatusEnum::EXITED => WorkloadStatus::Terminated,
        ContainerStateStatusEnum::DEAD => WorkloadStatus::Crashed,
        _ => WorkloadStatus::Running,
    }
}

#[derive(Clone)]
pub struct ContainerListener {}

impl ContainerListener {
    /// Fetches all the needed data and return it in an InstanceStatus
    ///
    /// # Arguments
    ///
    /// * `container_id` - container's id
    /// * `instance` - instance's struct
    /// * `docker_connection` - bollard's Docker Struct
    pub async fn fetch_instance_status(
        container_id: &str,
        instance: &Instance,
        docker_connection: &Docker,
    ) -> Result<InstanceStatus, Status> {
        let stats_options = Some(StatsOptions {
            stream: true,
            one_shot: false,
        });

        let lim = match instance.clone().resource {
            Some(resource) => match resource.limit {
                Some(limit) => limit,
                None => return Err(Status::internal("Could not get limit in Instance struct")),
            },
            None => return Err(Status::internal("No ressource in Instance struct")),
        };

        let container_data_result = docker_connection
            .inspect_container(container_id, None)
            .await;
        debug!("inspected container");

        let container_resources_opt = docker_connection
            .stats(container_id, stats_options)
            .next()
            .await;
        debug!("got stats stream");


        let container_resources = container_resources_opt
            .ok_or(|e: Error| e)
            .map_err(
                //At this point we have a Result<Result<Stats, Error>, Error> so we use map_er two times
                |_e| {
                    Status::internal(format!(
                        "Cannot get state of workload {} (container {}) ",
                        instance.id, container_id
                    ))
                },
            )?
            .map_err(|e| Status::internal(e.to_string()))?;

        let container_data = container_data_result.map_err(|e| Status::internal(e.to_string()))?;

        let workload_status = convert_status(
            container_data
                .state
                .ok_or_else(|| {
                    Status::internal(format!(
                        "Cannot get state of workload {} (container {}) ",
                        instance.id, container_id
                    ))
                })?
                .status,
        );

        let cpu_usage_in_milli_cpu = ContainerListener::calculate_cpu_usage(
            container_resources.cpu_stats,
            container_resources.precpu_stats,
        );

        Ok(InstanceStatus {
            id: instance.clone().id,
            status: workload_status as i32,
            description: "heres a description".to_string(),
            resource: Some(Resource {
                limit: Some(ResourceSummary {
                    cpu: lim.clone().cpu,
                    memory: lim.clone().memory,
                    disk: lim.clone().disk,
                }),
                usage: Some(ResourceSummary {
                    cpu: cpu_usage_in_milli_cpu,
                    memory: container_resources.memory_stats.usage.unwrap_or(0),
                    disk: container_resources
                        .storage_stats
                        .read_count_normalized
                        .unwrap_or(0),
                }),
            }),
        })
    }

    /// Calculates and returns the CPU usage of container in millicpu, https://docs.docker.com/engine/api/v1.41/#tag/Container/operation/ContainerStats
    ///
    /// # Arguments
    ///
    /// * `cpu` - cpu usage
    /// * `pre_cpu` - pre_cpu usage
    fn calculate_cpu_usage(cpu: CPUStats, pre_cpu: CPUStats) -> u64 {
        let mut sys = System::new_all();
        sys.refresh_all();
        let cpu_cores_number = match sys.physical_core_count() {
            None => 0,
            Some(n) => n as u64,
        };
        let total_mulli_cpu = cpu_cores_number * 1000;

        let cpu_delta = cpu.cpu_usage.total_usage - pre_cpu.cpu_usage.total_usage;

        let mut cpu_delta_system = cpu.system_cpu_usage.unwrap_or(0);

        //to avoid overflow: u64::abs does not exists and u64::abs_diff is deprecated
        cpu_delta_system = if cpu_delta_system >= pre_cpu.system_cpu_usage.unwrap_or(0) {
            cpu_delta_system - pre_cpu.system_cpu_usage.unwrap_or(0)
        } else {
            pre_cpu.system_cpu_usage.unwrap_or(0) - cpu_delta_system
        };

        let cpu_percentage = if cpu_delta_system > 0 {
            (cpu_delta / cpu_delta_system) * cpu_cores_number
        } else {
            0
        };

        total_mulli_cpu * cpu_percentage
    }
}

impl WorkloadListener for ContainerListener {
    fn run(id: String, instance: Instance, sender: Sender<Result<InstanceStatus, Status>>) {
        std::thread::spawn(move || -> Result<(), Status> {
            #[cfg(unix)]
            let docker = Docker::connect_with_socket_defaults().unwrap();
            let rt = Runtime::new().unwrap();

            let mut cached = InstanceStatus::default();
            debug!("{:?}", cached);

            loop {
                let new_instance_status = rt.block_on(ContainerListener::fetch_instance_status(
                    id.as_str(),
                    &instance,
                    &docker,
                ));
                debug!("finished fetching data");
                debug!("{:?}", new_instance_status);

                match new_instance_status {
                    Ok(instance_to_send) => {

                        if cached == instance_to_send {
                            continue;
                        }
                        cached = instance_to_send.clone();

                        let status: i32 = instance_to_send.status;

                        rt.block_on(sender.send(Ok(instance_to_send)))
                            .map_err(|e| Status::internal(e.to_string()))?;

                        if status == WorkloadStatus::Crashed as i32
                            || status == WorkloadStatus::Terminated as i32
                        {
                            debug!("Listener is stopping because contained has crashed or terminated");
                            break;
                        }
                    }
                    Err(e) => {
                        rt.block_on(sender.send(Err(e))).unwrap();
                        debug!("Error while fetching instance status, listener is stopping");
                        break;
                    }
                };
            }
            info!("Container listener stopped");

            Ok(())
        });

        info!("Container listener up and running!");
    }
}

#[cfg(test)]
mod tests {
    use crate::workload_manager::workload_listener::workload_listener::WorkloadListener;
    use bollard::service::HealthConfig;
    use bollard::{container::Config, image::CreateImageOptions, Docker};
    use futures::TryStreamExt;
    use proto::agent::Status as WorkloadStatus;
    use proto::agent::{Instance, Port, Resource, ResourceSummary, Status, Type as IType};
    use tokio::sync::mpsc::channel;

    #[tokio::test]
    async fn test_fetch() -> Result<(), Status> {
        //test setup
        #[cfg(unix)]
        let docker = Docker::connect_with_socket_defaults().unwrap();

        docker
            .create_image(
                Some(CreateImageOptions::<&str> {
                    from_image: "debian:latest",
                    ..Default::default()
                }),
                None,
                None,
            )
            .try_collect::<Vec<_>>()
            .await
            .unwrap();

        let health_check = Some(HealthConfig {
            test: Some(vec![
                "/bin/echo".to_string(),
                "a very well done test".to_string(),
            ]),
            interval: Some(1000000),
            timeout: Some(500000000),
            retries: Some(1),
            start_period: Some(1000000),
        });

        let cfg = Config {
            cmd: Some(vec!["tee"]),
            image: Some("debian"),
            tty: Some(true),
            attach_stdin: Some(false),
            attach_stdout: Some(false),
            attach_stderr: Some(false),
            open_stdin: Some(false),
            healthcheck: health_check,
            ..Default::default()
        };

        let container = docker
            .create_container::<&str, &str>(None, cfg)
            .await
            .unwrap();
        docker
            .start_container::<String>(&container.clone().id, None)
            .await
            .unwrap();

        let instance = Instance {
            id: "someuuid".to_string(),
            name: "somename".to_string(),
            r#type: IType::Container as i32,
            status: Status::Running as i32,
            uri: "http://localhost".to_string(),
            environment: vec!["A=0".to_string()],
            resource: Some(Resource {
                limit: Some(ResourceSummary {
                    cpu: u64::MAX,
                    memory: u64::MAX,
                    disk: u64::MAX,
                }),
                usage: Some(ResourceSummary {
                    cpu: 0,
                    memory: 0,
                    disk: 0,
                }),
            }),
            ports: vec![Port {
                source: 80,
                destination: 80,
            }],
            ip: "127.0.0.1".to_string(),
        };

        let res = super::ContainerListener::fetch_instance_status(
            container.id.as_str(),
            &instance,
            &docker,
        )
        .await
        .unwrap();

        docker
            .kill_container::<String>(container.id.as_str(), None)
            .await
            .unwrap();
        docker
            .remove_container(container.id.as_str(), None)
            .await
            .unwrap();

        assert_eq!(res.id, instance.id);
        assert!(res.resource.unwrap().usage.unwrap().cpu < u64::MAX);

        Ok(())
    }

    #[tokio::test]
    async fn it_works() -> Result<(), ()> {
        //test setup
        #[cfg(unix)]
        let docker = Docker::connect_with_socket_defaults().unwrap();

        docker
            .create_image(
                Some(CreateImageOptions::<&str> {
                    from_image: "debian:latest",
                    ..Default::default()
                }),
                None,
                None,
            )
            .try_collect::<Vec<_>>()
            .await
            .unwrap();

        let cfg = Config {
            cmd: Some(vec!["/bin/sleep", "5"]),
            image: Some("debian"),
            tty: Some(true),
            attach_stdin: Some(false),
            attach_stdout: Some(false),
            attach_stderr: Some(false),
            open_stdin: Some(false),
            ..Default::default()
        };

        let container = docker
            .create_container::<&str, &str>(None, cfg)
            .await
            .unwrap();
        docker
            .start_container::<String>(&container.clone().id, None)
            .await
            .unwrap();

        let instance = Instance {
            id: "someuuid".to_string(),
            name: "somename".to_string(),
            r#type: IType::Container as i32,
            status: Status::Running as i32,
            uri: "http://localhost".to_string(),
            environment: vec!["A=0".to_string()],
            resource: Some(Resource {
                limit: Some(ResourceSummary {
                    cpu: u64::MAX,
                    memory: u64::MAX,
                    disk: u64::MAX,
                }),
                usage: Some(ResourceSummary {
                    cpu: 0,
                    memory: 0,
                    disk: 0,
                }),
            }),
            ports: vec![Port {
                source: 80,
                destination: 80,
            }],
            ip: "127.0.0.1".to_string(),
        };

        let (tx, mut rx) = channel(1000);

        //test
        crate::workload_manager::workload_listener::ContainerListener::run(container.clone().id, instance, tx.clone());

        loop {
            let msg = rx.recv().await;

            let received = msg.unwrap().unwrap();
            let status = received.status as i32;
            let received2 = received.clone();
            let received3 = received.clone();

            println!("{:?}", received);

            assert_eq!(received.resource.unwrap().limit.unwrap().cpu, u64::MAX);
            assert!(received2.resource.unwrap().usage.unwrap().cpu < u64::MAX);
            assert!(received3.status as i32 <= 8);

            if status == WorkloadStatus::Crashed as i32
                || status == WorkloadStatus::Terminated as i32
            {
                break;
            }
        }

        docker
            .remove_container(container.id.as_str(), None)
            .await
            .unwrap();

        Ok(())
    }
}