use log::debug;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response};

use proto::scheduler::{
    instance_service_server::InstanceService, Instance, InstanceIdentifier, InstanceStatus, Port,
    Resource, ResourceSummary, Status,
};
use uuid::Uuid;

use crate::{manager::Manager, Event, NodeIdentifier, ProxyError};

#[derive(Debug)]
pub struct InstanceListener {
    sender: mpsc::Sender<Event>,
}

impl InstanceListener {
    pub fn new(sender: mpsc::Sender<Event>) -> Self {
        InstanceListener { sender }
    }
}

#[tonic::async_trait]
impl InstanceService for InstanceListener {
    async fn create(
        &self,
        request: Request<Instance>,
    ) -> Result<Response<Self::CreateStream>, tonic::Status> {
        debug!("received request: {:?}", request);
        let (tx, rx) = Manager::create_mpsc_channel();

        match self
            .sender
            .send(Event::InstanceCreate(request.into_inner(), tx))
            .await
        {
            Ok(_) => {
                return Ok(Response::new(ReceiverStream::new(rx)));
            }
            Err(_) => {
                return Err(tonic::Status::internal("could not send event to manager"));
            }
        }
    }

    type CreateStream = ReceiverStream<Result<InstanceStatus, tonic::Status>>;

    async fn start(&self, _: Request<InstanceIdentifier>) -> Result<Response<()>, tonic::Status> {
        Err(tonic::Status::unimplemented("not implemented"))
    }

    async fn stop(
        &self,
        request: Request<InstanceIdentifier>,
    ) -> Result<Response<()>, tonic::Status> {
        debug!("received request: {:?}", request);
        let (tx, rx) = Manager::create_oneshot_channel();

        match self
            .sender
            .send(Event::InstanceStop(request.into_inner().id, tx))
            .await
        {
            Ok(_) => {
                return rx.await.unwrap();
            }
            Err(_) => {
                return Err(tonic::Status::internal("could not send event to manager"));
            }
        }
    }

    async fn destroy(
        &self,
        request: Request<InstanceIdentifier>,
    ) -> Result<Response<()>, tonic::Status> {
        debug!("received request: {:?}", request);
        let (tx, rx) = Manager::create_oneshot_channel();

        match self
            .sender
            .send(Event::InstanceDestroy(request.into_inner().id, tx))
            .await
        {
            Ok(_) => {
                return rx.await.unwrap();
            }
            Err(_) => {
                return Err(tonic::Status::internal("could not send event to manager"));
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct InstanceProxied {
    pub id: String,
    pub instance: Instance,
    pub node_id: Option<NodeIdentifier>,
    pub tx: mpsc::Sender<Result<InstanceStatus, tonic::Status>>,
}

impl InstanceProxied {
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

    pub async fn change_status(
        &mut self,
        status: Status,
        description: Option<String>,
    ) -> Result<(), ProxyError> {
        self.instance.status = status.into();

        self.tx
            .send(Ok(InstanceStatus {
                id: Uuid::new_v4().to_string(),
                status: status.into(),
                status_description: description.unwrap_or("".to_string()),
                resource: match Status::from_i32(self.instance.status) {
                    Some(Status::Running) => self.instance.resource.clone(),
                    _ => None,
                },
            }))
            .await
            .map_err(|_| ProxyError::ChannelSenderError)?;

        Ok(())
    }
}

pub struct InstanceParser {}

impl InstanceParser {
    pub fn to_agent_instance(instance: Instance) -> proto::agent::Instance {
        proto::agent::Instance {
            id: instance.id,
            name: instance.name,
            r#type: instance.r#type,
            status: instance.status.into(),
            uri: instance.uri,
            environment: instance.environnement,
            resource: match instance.resource {
                Some(r) => Some(Self::to_agent_resource(r)),
                None => None,
            },
            ports: Self::to_agent_ports(instance.ports),
            ip: instance.ip,
        }
    }

    pub fn from_agent_instance(instance: proto::agent::Instance) -> Instance {
        Instance {
            id: instance.id,
            name: instance.name,
            r#type: instance.r#type,
            status: instance.status.into(),
            uri: instance.uri,
            environnement: instance.environment,
            resource: match instance.resource {
                Some(r) => Some(Self::from_agent_resource(r)),
                None => None,
            },
            ports: Self::from_agent_ports(instance.ports),
            ip: instance.ip,
        }
    }

    pub fn to_agent_resource(resource: Resource) -> proto::agent::Resource {
        proto::agent::Resource {
            limit: match resource.limit {
                Some(r) => Some(Self::to_agent_resourcesummary(r)),
                None => None,
            },
            usage: match resource.usage {
                Some(r) => Some(Self::to_agent_resourcesummary(r)),
                None => None,
            },
        }
    }

    pub fn from_agent_resource(resource: proto::agent::Resource) -> Resource {
        Resource {
            limit: match resource.limit {
                Some(r) => Some(Self::from_agent_resourcesummary(r)),
                None => None,
            },
            usage: match resource.usage {
                Some(r) => Some(Self::from_agent_resourcesummary(r)),
                None => None,
            },
        }
    }

    pub fn to_agent_resourcesummary(resource: ResourceSummary) -> proto::agent::ResourceSummary {
        proto::agent::ResourceSummary {
            cpu: resource.cpu,
            memory: resource.memory,
            disk: resource.disk,
        }
    }

    pub fn from_agent_resourcesummary(resource: proto::agent::ResourceSummary) -> ResourceSummary {
        ResourceSummary {
            cpu: resource.cpu,
            memory: resource.memory,
            disk: resource.disk,
        }
    }

    pub fn to_agent_ports(ports: Vec<Port>) -> Vec<proto::agent::Port> {
        ports
            .into_iter()
            .map(|port| proto::agent::Port {
                source: port.source,
                destination: port.destination,
            })
            .collect()
    }

    pub fn from_agent_ports(ports: Vec<proto::agent::Port>) -> Vec<Port> {
        ports
            .into_iter()
            .map(|port| Port {
                source: port.source,
                destination: port.destination,
            })
            .collect()
    }

    pub fn from_agent_instance_status(status: proto::agent::InstanceStatus) -> InstanceStatus {
        InstanceStatus {
            id: status.id,
            status: status.status.into(),
            status_description: status.description,
            resource: match status.resource {
                Some(r) => Some(Self::from_agent_resource(r)),
                None => None,
            },
        }
    }

    pub fn fake_agent_instance(id: String) -> proto::agent::Instance {
        proto::agent::Instance {
            id,
            name: "".to_string(),
            r#type: proto::agent::Type::Container.into(),
            status: Status::Stopping.into(),
            uri: "".to_string(),
            environment: vec![],
            resource: None,
            ports: vec![],
            ip: "".to_string(),
        }
    }
}
