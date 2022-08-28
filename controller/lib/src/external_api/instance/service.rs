use std::net::{Ipv4Addr, SocketAddr};
use std::sync::Arc;

use super::filter::FilterService;
use super::model::{Instance, InstanceError};
use crate::etcd::EtcdClient;
use crate::external_api::workload::model::Workload;
use crate::grpc_client::interface::SchedulerClientInterface;
use proto::controller::InstanceState;
use serde_json;
use tokio::sync::Mutex;
use tonic::Request;

pub struct InstanceService {
    grpc_service: SchedulerClientInterface,
    etcd_service: EtcdClient,
    filter_service: FilterService,
}

// `InstanceService` is a struct that is inspired from Controllers Provider Modules architectures. It is used as a service in the InstanceController. A service can use other services.
/// Properties:
///
/// * `grpc_service`: This is the service that will be used to filter to interact with grpc.
/// * `etcd_service`: This is the service that will be used to interact with etcd.
/// * `filter_service`: This is the service that will be used to filter the instances.

impl InstanceService {
    pub async fn new(grpc_address: &str, etcd_address: &SocketAddr) -> Result<Self, InstanceError> {
        Ok(InstanceService {
            grpc_service: SchedulerClientInterface::new(grpc_address.to_string())
                .await
                .map_err(|err| InstanceError::Grpc(err.to_string()))?,
            etcd_service: EtcdClient::new(etcd_address.to_string())
                .await
                .map_err(|err| InstanceError::Etcd(err.to_string()))?,
            filter_service: FilterService::new(),
        })
    }

    /// `generate_ip` is an async function that generate an ip address for a instance.
    /// # Description:
    /// It stores the last ip used in ETCD and increment it by 1 every time it is used .
    /// * Get all workload in the namespace
    /// # Return
    /// An Result<Ipv4Addr, InstanceError>
    pub async fn generate_ip(&mut self) -> Result<Ipv4Addr, InstanceError> {
        let mut ip = Ipv4Addr::new(10, 0, 0, 1);
        match self.etcd_service.get("last_ip").await {
            Some(value) => {
                let ip_address: Ipv4Addr = serde_json::from_str(&value)
                    .map_err(InstanceError::SerdeError)
                    .unwrap();
                let mut octets = ip_address.octets();
                for i in 0..3 {
                    if octets[3 - i] < 255 {
                        octets[3 - i] += 1;
                        break;
                    } else {
                        octets[3 - i] = 0;
                    }
                }
                ip = Ipv4Addr::from(octets);
                self.etcd_service
                    .put(
                        "last_ip",
                        &serde_json::to_string(&ip)
                            .map_err(InstanceError::SerdeError)
                            .unwrap(),
                    )
                    .await
                    .map_err(|err| InstanceError::Etcd(err.to_string()))?;
                Ok(ip)
            }
            None => {
                self.etcd_service
                    .put(
                        "last_ip",
                        &serde_json::to_string(&ip)
                            .map_err(InstanceError::SerdeError)
                            .unwrap(),
                    )
                    .await
                    .map_err(|err| InstanceError::Etcd(err.to_string()))?;
                Ok(ip)
            }
        }
    }

    /// It creates a new instance in etcd and call the `schedule` function of the grpc service
    ///
    /// # Arguments:
    ///
    /// * `this`: the method is called on the `this` object. Used to handle threads.
    /// * `workload_id`: The id of the workload that the instance will be created for.
    ///
    /// # Returns:
    ///
    /// A Result.
    pub async fn retrieve_and_start_instance_from_workload(
        this: Arc<Mutex<Self>>,
        workload_id: &str,
    ) -> Result<(), InstanceError> {
        let ip = this
            .clone()
            .lock()
            .await
            .generate_ip()
            .await
            .map_err(|err| InstanceError::GenerateIp(err.to_string()))
            .unwrap();
        match this
            .clone()
            .lock()
            .await
            .etcd_service
            .get(workload_id)
            .await
        {
            Some(workload) => {
                let workload_parsed: Workload = serde_json::from_str(&workload).unwrap();
                let mut instance = Instance::from(workload_parsed);
                instance.ip = ip;
                log::info!("Instance retrieved from workload: {:?}", instance);
                Self::schedule_instance(this, instance);

                Ok(())
            }
            None => Err(InstanceError::InstanceNotFound),
        }
    }

    /// Schedule an new instance by calling the GRPC client and recursively call this function if the instance is not scheduled.
    ///
    /// # Arguments:
    ///
    /// * `this`: the method is called on the `this` object. Used to handle threads.
    /// * `instance`: The instance to schedule
    fn schedule_instance(this: Arc<Mutex<Self>>, mut instance: Instance) {
        //Spawn a thread to start the instance
        tokio::spawn(async move {
            loop {
                let mut stream = this
                    .clone()
                    .lock()
                    .await
                    .grpc_service
                    .create_instance(Request::new(Instance::into(instance.clone())))
                    .await
                    .map_err(|err| InstanceError::Grpc(err.to_string()))
                    .unwrap()
                    .into_inner();

                let mut last_state = InstanceState::Scheduling;

                while let Some(instance_status) = stream
                    .message()
                    .await
                    .map_err(|err| InstanceError::Grpc(err.to_string()))
                    .unwrap()
                {
                    instance.update_instance(instance_status.clone());

                    this.clone()
                        .lock()
                        .await
                        .etcd_service
                        .put(
                            &instance.id,
                            &serde_json::to_string(&instance)
                                .map_err(InstanceError::SerdeError)
                                .unwrap(),
                        )
                        .await
                        .map_err(|err| InstanceError::Etcd(err.to_string()))
                        .unwrap();

                    last_state = InstanceState::from_i32(instance_status.status)
                        .unwrap_or(InstanceState::Scheduling);
                }

                if last_state == InstanceState::Terminated {
                    break;
                }

                instance.num_restarts += 1;

                this.clone()
                    .lock()
                    .await
                    .etcd_service
                    .put(
                        &instance.id,
                        &serde_json::to_string(&instance)
                            .map_err(InstanceError::SerdeError)
                            .unwrap(),
                    )
                    .await
                    .map_err(|err| InstanceError::Grpc(err.to_string()))
                    .unwrap();
            }
        });
    }

    /// Delete an instance from etcd and call the `destroy_instance` function of the grpc service
    ///
    /// # Arguments:
    ///
    /// * `instance`: The instance to delete
    ///
    /// # Returns:
    ///
    /// A Result.
    pub async fn delete_instance(&mut self, instance: Instance) -> Result<(), InstanceError> {
        match self.etcd_service.delete(instance.id.as_str()).await {
            Some(_) => {
                match self
                    .grpc_service
                    .destroy_instance(Request::new(proto::scheduler::InstanceIdentifier {
                        id: instance.id,
                    }))
                    .await
                {
                    Ok(_) => Ok(()),
                    Err(_) => Err(InstanceError::Grpc("Error stopping instance".to_string())),
                }
            }
            None => Err(InstanceError::InstanceNotFound),
        }
    }

    /// Get an instance by it's name
    ///
    /// # Arguments:
    ///
    /// * `namespace`: The namespace of the instance
    ///
    /// # Returns:
    ///
    /// A Instance.
    pub async fn get_instance(&mut self, instance_name: &str) -> Result<Instance, InstanceError> {
        match self
            .etcd_service
            .get(format!("instance.{}", instance_name).as_str())
            .await
        {
            Some(instance_str) => match serde_json::from_str::<Instance>(&instance_str) {
                Ok(instance) => Ok(instance),
                Err(_) => Err(InstanceError::InstanceNotFound),
            },
            None => Err(InstanceError::InstanceNotFound),
        }
    }

    /// This function gets all instances from etcd, filters them by namespace and slice the result by limit and offset
    /// If there is an error , the function always return an empty vector
    /// # Arguments:
    ///
    /// * `limit`: The number of instances to return.
    /// * `offset`: The offset of the instances to be returned.
    /// * `namespace`: The namespace to filter by.
    ///
    /// # Returns:
    ///
    /// A vector of instances
    pub async fn get_instances(
        &mut self,
        limit: u32,
        offset: u32,
        namespace: &str,
    ) -> Vec<Instance> {
        let mut vec: Vec<Instance> = Vec::new();
        match self.etcd_service.get_all().await {
            Some(instances) => {
                for instance in instances {
                    if let Ok(instance) = serde_json::from_str::<Instance>(&instance) {
                        if instance.namespace == namespace {
                            vec.push(instance);
                        }
                    }
                }
                if offset > 0 {
                    match self.filter_service.offset(&vec, limit) {
                        Ok(instances) => vec = instances,
                        Err(_) => return vec![],
                    }
                }
                if limit > 0 {
                    vec = self.filter_service.limit(&vec, limit);
                }
            }
            None => return vec![],
        }
        vec
    }
}
