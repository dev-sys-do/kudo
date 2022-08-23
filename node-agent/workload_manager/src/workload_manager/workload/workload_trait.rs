use anyhow::Result;

#[tonic::async_trait]
pub trait Workload {
    fn id(&self) -> String;

    //
    // Gracefully stops a workload
    //
    async fn stop(&self) -> Result<()>;

    //
    // Force a workload to stop
    // (equivalent to a `kill -9` on linux)
    //
    async fn kill(&self) -> Result<()>;
}
