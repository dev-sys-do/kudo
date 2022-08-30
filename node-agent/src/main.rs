use std::env;
use std::net::Ipv4Addr;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::Duration;

use cidr::Ipv4Inet;
use log::{debug, info, trace};
use network::node::request::SetupNodeRequest;
use tokio::time;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{transport::Server, Request, Response, Status};
use uuid::Uuid;

mod config;

use config::{GrpcServerConfig, NodeAgentConfig};
use network::node::setup_node;
use node_manager::NodeSystem;
use workload_manager::workload_manager::WorkloadManager;

use proto::agent::{
    instance_service_server::InstanceService, instance_service_server::InstanceServiceServer,
    Instance, InstanceStatus, SignalInstruction,
};
use proto::scheduler::{
    node_service_client::NodeServiceClient, NodeRegisterRequest, NodeRegisterResponse, NodeStatus,
    Resource, ResourceSummary, Status as SchedulerStatus,
};

const NUMBER_OF_CONNECTION_ATTEMPTS: u16 = 10;

///
/// This Struct implement the Instance service from Node Agent proto file
///
#[derive(Debug)]
pub struct InstanceServiceController {
    workload_manager: WorkloadManager,
}

impl InstanceServiceController {
    pub fn new(node_id: String) -> Self {
        Self {
            workload_manager: WorkloadManager::new(node_id),
        }
    }
}

#[tonic::async_trait]
impl InstanceService for InstanceServiceController {
    type createStream = ReceiverStream<Result<InstanceStatus, Status>>;

    async fn create(
        &self,
        request: Request<Instance>,
    ) -> Result<Response<Self::createStream>, Status> {
        let instance = request.into_inner();
        // let receiver = self.workload_manager.create(instance).await?;
        let receiver = self.workload_manager.create(instance).await;

        Ok(Response::new(ReceiverStream::new(receiver)))
    }

    async fn signal(&self, request: Request<SignalInstruction>) -> Result<Response<()>, Status> {
        let signal_instruction = request.into_inner();

        Ok(Response::new(
            self.workload_manager.signal(signal_instruction).await?,
        ))
    }
}

///
/// This function starts the grpc server of the Node Agent.
/// The server listens and responds to requests from the Scheduler.
/// The default port is 50053.
///
fn create_grpc_server(config: GrpcServerConfig, node_id: String) -> tokio::task::JoinHandle<()> {
    let addr = format!("http://{}:{}", config.host, config.port)
        .parse()
        .unwrap();
    let instance_service_controller = InstanceServiceController::new(node_id);

    info!("Node Agent server listening on {}", addr);

    tokio::spawn(async move {
        Server::builder()
            .add_service(InstanceServiceServer::new(instance_service_controller))
            .serve(addr)
            .await
            .unwrap()
    })
}

///
/// This function allows you to connect to the scheduler's grpc server.
///
async fn connect_to_scheduler(
    addr: String,
) -> Option<NodeServiceClient<tonic::transport::Channel>> {
    NodeServiceClient::connect(addr.clone()).await.ok()
}

///
/// This function allows you to register to the scheduler's grpc server.
///
async fn register_to_scheduler(
    client: &mut NodeServiceClient<tonic::transport::Channel>,
    certificate: String,
) -> Option<tonic::Response<NodeRegisterResponse>> {
    let register_request = tonic::Request::new(NodeRegisterRequest { certificate });

    client.register(register_request).await.ok()
}

///
/// This function allows you to send node status to the scheduler's grpc server.
///
async fn send_node_satus_to_scheduler(
    client: &mut NodeServiceClient<tonic::transport::Channel>,
    node_system_arc: Arc<std::sync::Mutex<NodeSystem>>,
    node_id: String,
) -> Option<tonic::Response<()>> {
    let node_status_stream = async_stream::stream! {
        let mut interval = time::interval(Duration::from_secs(1));

        loop {
            interval.tick().await;

            let cpu_limit = node_system_arc.lock().unwrap().total_cpu();
            let cpu_usage = node_system_arc.lock().unwrap().used_cpu();

            let memory_limit = node_system_arc.lock().unwrap().total_memory();
            let memory_usage = node_system_arc.lock().unwrap().used_memory();

            let disk_limit = node_system_arc.lock().unwrap().total_disk();
            let disk_usage = node_system_arc.lock().unwrap().used_disk();

            let node_status = NodeStatus {
                id: node_id.clone(),
                status: SchedulerStatus::Running as i32,
                status_description: "".into(),
                resource: Some(Resource {
                    limit: Some(ResourceSummary {
                        cpu: cpu_limit,
                        memory: memory_limit,
                        disk: disk_limit,
                    }),
                    usage: Some(ResourceSummary {
                        cpu: cpu_usage,
                        memory: memory_usage,
                        disk: disk_usage,
                    }),
                }),
            };

            debug!("Node resources sent to the Scheduler");

            yield node_status;
        }
    };

    client.status(Request::new(node_status_stream)).await.ok()
}

///
/// This function launch the Node Agent grpc client.
/// First, the client registered to the Scheduler.
/// Secondaly, once connected to it, it's send node resources to the Scheduler.
///
fn create_grpc_client(config: GrpcServerConfig, node_id: String) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        //  Connection to the Scheduler's grpc server

        let addr = format!("http://{}:{}", config.host, config.port);
        let mut connection = connect_to_scheduler(addr.clone()).await;

        let mut attempts: u16 = 0;
        while connection.is_none() {
            if attempts <= NUMBER_OF_CONNECTION_ATTEMPTS {
                sleep(Duration::from_secs(1));

                debug!("Connection to grpc scheduler server failed, retrying...");
                connection = connect_to_scheduler(addr.clone()).await;

                attempts += 1;
            } else {
                panic!("Error, unable to connect to the Scheduler server.");
            }
        }

        let mut client = connection.unwrap();

        info!("Node agent connected to the Scheduler at {}", addr);

        // Registration with the Scheduler

        let certificate = node_id.clone();
        let mut registration = register_to_scheduler(&mut client, certificate).await;

        // setup node

        let node_ip = registration.unwrap().into_inner().id;
        let node_ip_addr = Ipv4Addr::from_str(&node_ip).unwrap();
        let node_ip_cidr = Ipv4Inet::new(node_ip_addr, 24).unwrap();

        let request = SetupNodeRequest::new(node_id.to_string(), node_ip_cidr);
        let response = setup_node(request).unwrap();

        attempts = 0;
        while registration.is_none() {
            if attempts <= NUMBER_OF_CONNECTION_ATTEMPTS {
                sleep(Duration::from_secs(1));

                debug!("Registration to the Scheduler failed, retrying...");
                registration = register_to_scheduler(&mut client, certificate).await;

                attempts += 1;
            } else {
                panic!("Error, unable to register to the Scheduler.");
            }
        }

        info!("Node agent registered to the Scheduler");

        // Send Node status to the Scheduler

        let node_system = NodeSystem::new();
        let arc_node_system = Arc::new(Mutex::new(node_system));

        let mut send_node_status_to_scheduler =
            send_node_satus_to_scheduler(&mut client, Arc::clone(&arc_node_system), node_id).await;

        attempts = 0;
        while send_node_status_to_scheduler.is_none() {
            if attempts <= NUMBER_OF_CONNECTION_ATTEMPTS {
                sleep(Duration::from_secs(1));

                debug!("Sending node status to the Scheduler failed, retrying...");
                send_node_status_to_scheduler = send_node_satus_to_scheduler(
                    &mut client,
                    Arc::clone(&arc_node_system),
                    node_id,
                )
                .await;

                attempts += 1;
            } else {
                panic!("Error, unable to send node status to the Scheduler.");
            }
        }
    })
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    info!("Starting up node agent");

    info!("Loading config");
    let mut dir = env::current_exe()?; // get executable path
    dir.pop(); // remove executable name
    dir.push("agent.conf"); // add config file name

    trace!("Node Agent config at: {:?}", dir);

    // load config from path
    let config: NodeAgentConfig = confy::load_path(dir.as_path())?;
    debug!("config: {:?}", config);

    // generate node id
    let node_id = Uuid::new_v4().to_string();

    // start grpc server and client
    let client_handler = create_grpc_client(config.client, node_id.clone());
    let server_handler = create_grpc_server(config.server, node_id.clone());

    client_handler.await?;
    server_handler.await?;

    info!("Shutting down node agent");

    Ok(())
}
