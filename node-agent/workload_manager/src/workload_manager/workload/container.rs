use std::collections::HashMap;

use bollard::container::{
    Config, KillContainerOptions, RemoveContainerOptions, RenameContainerOptions,
    StopContainerOptions,
};
use bollard::Docker;

use anyhow::{Context, Error, Result};

use bollard::image::CreateImageOptions;
use futures_util::TryStreamExt;

use super::workload_trait::Workload;
use proto::agent::Instance;

pub struct Container {
    id: String,
}

impl Container {
    /// Create a new workload (container) and start it
    pub async fn new(instance: Instance) -> Result<Self, Error> {
        let docker =
            Docker::connect_with_socket_defaults().context("Can't connect to docker socket. ")?;

        docker
            .create_image(
                Some(CreateImageOptions {
                    from_image: instance.uri.clone(),
                    ..Default::default()
                }),
                None,
                None,
            )
            .try_collect::<Vec<_>>()
            .await
            .context("Can't create image. ")?;

        let container_id = create_container(&docker, instance).await?;

        docker
            .start_container::<String>(container_id.as_str(), None)
            .await
            .context("Can't start container. ")?;

        Ok(Container { id: container_id })
    }

    /// Removes a container
    async fn remove(&self) -> Result<(), Error> {
        let docker =
            Docker::connect_with_socket_defaults().context("Can't connect to docker socket. ")?;
        docker
            .remove_container(
                self.id().as_str(),
                Some(RemoveContainerOptions {
                    force: true,
                    ..Default::default()
                }),
            )
            .await
            .context("Can't remove container. ")?;

        Ok(())
    }
}

#[tonic::async_trait]
impl Workload for Container {
    fn id(&self) -> String {
        self.id.to_string()
    }

    // Gracefully stop a workload
    async fn stop(&self) -> Result<(), Error> {
        let docker =
            Docker::connect_with_socket_defaults().context("Can't connect to docker socket. ")?;

        docker
            .stop_container(
                self.id().as_str(),
                Some(StopContainerOptions {
                    ..Default::default()
                }),
            )
            .await
            .context("Can't stop docker container.")?;

        self.remove().await?;

        Ok(())
    }

    // Force a workload to stop
    // (equivalent to a `kill -9` on linux)
    async fn kill(&self) -> Result<(), Error> {
        let docker =
            Docker::connect_with_socket_defaults().context("Can't connect to docker socket. ")?;

        docker
            .kill_container(
                self.id().as_str(),
                Some(KillContainerOptions { signal: "SIGKILL" }),
            )
            .await
            .context("Can't kill docker container. ")?;

        self.remove().await?;

        Ok(())
    }
}

/// It creates a container with the given instance's configuration
///
/// Arguments:
///
/// * `docker`: &Docker - This is the docker client that we created earlier.
/// * `instance`: Instance
///
/// Returns:
///
/// A string that is the container id.
async fn create_container(docker: &Docker, instance: Instance) -> Result<String> {
    let mut ports = HashMap::new();
    let port_list = &instance.ports;
    for port in port_list {
        ports.insert(
            port.destination.to_string(),
            Some(vec![bollard::service::PortBinding {
                host_port: Some(port.destination.to_string()),
                ..Default::default()
            }]),
        );
    }

    let container_config: Config<&str> = Config {
        image: Some(instance.uri.as_str()),
        tty: Some(true),
        host_config: Some(bollard::service::HostConfig {
            port_bindings: Some(ports),
            nano_cpus: instance
                .resource
                .clone()
                .and_then(|resource| resource.limit.map(|limit| limit.cpu.try_into().unwrap())),
            memory: instance
                .resource
                .clone()
                .and_then(|resource| resource.limit.map(|limit| limit.memory.try_into().unwrap())),
            ..Default::default()
        }),
        ..Default::default()
    };

    let container_id = docker
        .create_container::<&str, &str>(None, container_config)
        .await
        .context("Can't create container. ")?
        .id;

    docker
        .rename_container(
            container_id.as_str(),
            RenameContainerOptions {
                name: instance.name,
            },
        )
        .await
        .ok();

    Ok(container_id)
}

#[cfg(test)]
mod tests {
    use crate::workload_manager::workload::workload_trait::Workload;

    use super::Container;
    use anyhow::{Error, Result};
    use bollard::{
        container::{ListContainersOptions, RemoveContainerOptions},
        Docker,
    };
    use proto::agent::{Instance, Resource, ResourceSummary, Type};

    const IMAGE: &str = "alpine:3";

    async fn create_default_container() -> Result<Container, Error> {
        let resource: Resource = Resource {
            limit: Some(ResourceSummary {
                cpu: 0,
                memory: 0,
                disk: 0,
            }),
            usage: Some(ResourceSummary {
                cpu: 0,
                memory: 0,
                disk: 0,
            }),
        };

        let instance = Instance {
            id: "0".to_string(),
            name: "test_container".to_string(),
            ip: "".to_string(),
            uri: IMAGE.to_string(),
            environment: Vec::new(),
            ports: Vec::new(),
            resource: Some(resource),
            status: 1,
            r#type: Type::Container.into(),
        };

        Ok(Container::new(instance).await?)
    }

    async fn create_container_test() -> Result<(), Error> {
        let container = create_default_container().await?;

        let docker = Docker::connect_with_socket_defaults()?;

        let result = &docker
            .list_containers(Some(ListContainersOptions::<String> {
                all: true,
                ..Default::default()
            }))
            .await?;

        assert_ne!(0, result.len());
        assert!(result
            .iter()
            .any(|cont| cont.id.as_ref().unwrap() == &container.id),);

        let _ = &docker
            .remove_container(
                container.id().as_str(),
                Some(RemoveContainerOptions {
                    force: true,
                    ..Default::default()
                }),
            )
            .await?;

        Ok(())
    }

    async fn stop_container_test() -> Result<(), Error> {
        let docker = Docker::connect_with_socket_defaults()?;

        let container = create_default_container().await?;

        container.stop().await?;

        let result = &docker
            .list_containers(Some(ListContainersOptions::<String> {
                all: true,
                ..Default::default()
            }))
            .await?;

        assert_eq!(
            result
                .iter()
                .any(|cont| cont.id.as_ref().unwrap() == &container.id),
            false
        );

        Ok(())
    }

    #[test]
    fn test_create_container() {
        tokio_test::block_on(create_container_test()).unwrap();
    }

    #[test]
    fn test_stop_container() {
        tokio_test::block_on(stop_container_test()).unwrap();
    }
}
