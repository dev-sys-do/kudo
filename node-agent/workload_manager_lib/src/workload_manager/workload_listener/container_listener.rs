use crate::workload_manager::workload_listener::workload_listener::{
    WorkloadListener
};
use crate::workload_manager::workload_listener::error::{
    WorkloadListenerError
};
use futures::stream::Stream;
use bollard::Docker;
use bollard::container::{CreateContainerOptions, Config};
use std::default::Default;

pub struct ContainerListener {
    id: String,
    container_id: String
}

impl ContainerListener {
    pub async fn new(id: &str, container_id: &str, docker: Docker) -> Result<Self, WorkloadListenerError>{
        let response = docker.inspect_container(id, None).await;
        match response {
            Ok(_) => Ok(ContainerListener { id: id.to_string(), container_id: container_id.to_string() }),
            Err(_e) => Err(WorkloadListenerError::new("Failed to create listener: container does not exist"))
        }
    }

    pub fn getContainerId(&self) -> &str {
        self.container_id.as_str()
    }

    pub fn getId(&self) -> &str {
        self.id.as_str()
    }
}

impl WorkloadListener for ContainerListener {
    fn getStream(&self) -> Box<dyn Stream<Item = String>>{
        panic!()
    }
}

#[tokio::test]
async fn it_works() -> Result<(), WorkloadListenerError>{
    #[cfg(unix)]
    let docker = Docker::connect_with_socket_defaults().unwrap();

    let opt = Some(CreateContainerOptions {
        name: "debian",
    });

    let cfg = Config {
        image: Some("debian"),
        cmd: Some(vec!["tee"]),
        tty: Some(true),
        attach_stdin: Some(false),
        attach_stdout: Some(false),
        attach_stderr: Some(false),
        open_stdin: Some(false),
        ..Default::default()
    };

    let container = docker.create_container::<&str, &str>(opt, cfg).await.unwrap();
    docker.start_container::<String>("debian", None).await.unwrap();

    let listener_stream = ContainerListener::new("debian", "debian", docker).await?;

    assert_eq!(listener_stream.getId(), "debian");
    //TODO: check if stream is open

    Ok(())
}