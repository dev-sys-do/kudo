use crate::workload_manager::workload_listener::workload_listener::WorkloadListener;

use futures::StreamExt;

use bollard::container::{CPUStats, StatsOptions};
use bollard::models::{ContainerStateStatusEnum, HealthStatusEnum};
use bollard::Docker;

use tokio::runtime::Runtime;
use tonic::Status;

use proto::agent::{Instance, InstanceStatus, Resource, ResourceSummary, Status as WorkloadStatus};

use std::sync::mpsc::Sender;
use sysinfo::{System, SystemExt};

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
    ///
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
        let container_resources_opt = docker_connection
            .stats(container_id, stats_options)
            .next()
            .await;

        if container_resources_opt.is_none() {
            return Err(Status::internal(format!(
                "Cannot poll stats from workload {} (container {}) ",
                instance.id, container_id
            )));
        };

        let container_resources_result = container_resources_opt.unwrap();

        let container_resources = match container_resources_result {
            Ok(res) => res,
            Err(e) => return Err(Status::internal(e.to_string())),
        };

        let container_data = match container_data_result {
            Ok(res) => res,
            Err(e) => return Err(Status::internal(e.to_string())),
        };

        let container_health_opt = container_data.clone().state;

        let mut workload_status = match container_data.state {
            Some(state) => match state.status {
                Some(container_status) => match container_status {
                    ContainerStateStatusEnum::RUNNING => WorkloadStatus::Running,
                    ContainerStateStatusEnum::REMOVING => WorkloadStatus::Destroying,
                    ContainerStateStatusEnum::EXITED => WorkloadStatus::Terminated,
                    ContainerStateStatusEnum::DEAD => WorkloadStatus::Crashed,
                    _ => WorkloadStatus::Running,
                },
                None => {
                    return Err(Status::internal(format!(
                        "Cannot get status of workload {} (container {}) ",
                        instance.id, container_id
                    )))
                }
            },
            None => {
                return Err(Status::internal(format!(
                    "Cannot get state of workload {} (container {}) ",
                    instance.id, container_id
                )))
            }
        };

        // In case we have a healthcheck in our container, we can override it with WorkloadStatus::Starting
        // https://docs.rs/bollard/latest/bollard/models/struct.Health.html
        workload_status = match container_health_opt {
            Some(s) => match s.health {
                Some(h) => match h.status {
                    Some(health_enum) => {
                        if health_enum == HealthStatusEnum::STARTING {
                            WorkloadStatus::Starting
                        } else {
                            workload_status
                        }
                    }
                    None => workload_status,
                },
                None => workload_status,
            },
            None => workload_status,
        };

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
                    memory: container_resources.memory_stats.usage.unwrap_or(0) as i32,
                    disk: container_resources
                        .storage_stats
                        .read_count_normalized
                        .unwrap_or(0) as i32,
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
    ///
    fn calculate_cpu_usage(cpu: CPUStats, pre_cpu: CPUStats) -> i32 {
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

        (total_mulli_cpu * cpu_percentage) as i32
    }
}

impl WorkloadListener for ContainerListener {
    fn new(id: String, instance: Instance, sender: Sender<InstanceStatus>) -> Self {
        std::thread::spawn(move || {
            #[cfg(unix)]
            let docker = Docker::connect_with_socket_defaults().unwrap();
            let rt = Runtime::new().unwrap();

            loop {
                let new_instance_status = rt.block_on(ContainerListener::fetch_instance_status(
                    id.as_str(),
                    &instance,
                    &docker,
                ));
                match new_instance_status {
                    Ok(instance_to_send) => {
                        let status: i32 = instance_to_send.status;

                        sender.send(instance_to_send).unwrap();

                        if status == WorkloadStatus::Crashed as i32
                            || status == WorkloadStatus::Terminated as i32
                        {
                            break;
                        }
                    }
                    Err(_e) => break,
                };
            }

            drop(sender);
        });

        Self {}
    }
}

#[cfg(test)]
mod tests {
    use super::WorkloadListener;
    use bollard::service::HealthConfig;
    use bollard::{container::Config, Docker};
    use proto::agent::Status as WorkloadStatus;
    use proto::agent::{Instance, Port, Resource, ResourceSummary, Status, Type as IType};
    use std::sync::mpsc::channel;

    #[tokio::test]
    async fn test_fetch() -> Result<(), Status> {
        //test setup
        #[cfg(unix)]
        let docker = Docker::connect_with_socket_defaults().unwrap();

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
                    cpu: i32::MAX,
                    memory: i32::MAX,
                    disk: i32::MAX,
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
        assert!(res.resource.unwrap().usage.unwrap().cpu >= 0);

        Ok(())
    }

    #[tokio::test]
    async fn it_works() -> Result<(), ()> {
        //test setup
        #[cfg(unix)]
        let docker = Docker::connect_with_socket_defaults().unwrap();
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
                    cpu: i32::MAX,
                    memory: i32::MAX,
                    disk: i32::MAX,
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

        let (tx, rx) = channel();

        //test
        super::ContainerListener::new(container.clone().id, instance, tx.clone());

        loop {
            let msg = rx.recv();

            if msg.is_err() {
                break;
            }

            let received = msg.unwrap();
            let status = received.status as i32;
            let received2 = received.clone();
            let received3 = received.clone();

            println!("{:?}", received);

            assert_eq!(received.resource.unwrap().limit.unwrap().cpu, i32::MAX);
            assert!(received2.resource.unwrap().usage.unwrap().cpu >= 0);
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
