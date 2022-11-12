use std::time::Duration;

use crate::fleetlock;

pub mod ceph;
pub mod kubernetes;

#[async_trait::async_trait]
pub trait Strategy {
    async fn pre_reboot(&self) -> Result<(), crate::Error>;
    async fn post_reboot(&self) -> Result<(), crate::Error>;
    async fn timeout(&self) -> Result<(), crate::Error>;

    /// Maximum time taken after the first run of `post_reboot` before failure is assumed.
    fn timeout_interval(&self) -> Duration;

    /// Minimum interval between runs of `post_reboot`.
    fn poll_interval(&self) -> Duration;

    /// Higher priority number -> run first.
    fn priority(&self) -> Priority;
}

pub struct Priority {
    pub pre: u64,
    pub post: u64,
}

pub async fn init_strategies(
    req: &fleetlock::Request,
) -> Result<Vec<Box<impl Strategy>>, crate::Error> {
    Ok(vec![Box::new(
        kubernetes::Kubernetes::new(req.client_params.id.clone()).await?,
    )])
}
