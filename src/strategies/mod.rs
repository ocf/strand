use std::time::Duration;

pub mod kubernetes;
pub mod ceph;

#[async_trait::async_trait]
trait Strategy {
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

struct Priority {
    pub pre: usize,
    pub post: usize,
}
