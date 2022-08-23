use anyhow::Result;
use proto::agent::{Instance, Type};
use workload_trait::Workload;

mod container;
pub mod workload_trait;

pub async fn create(instance: Instance) -> Result<impl Workload> {
    match instance.r#type() {
        Type::Container => container::Container::new(instance).await,
    }
}
