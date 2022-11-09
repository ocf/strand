use std::time::Duration;

use crate::strategies::Strategy;

struct Ceph {}

#[async_trait::async_trait]
impl Strategy for Ceph {
    async fn pre_reboot(&self) -> Result<(), crate::Error> {
        todo!("set noout")
    }

    async fn post_reboot(&self) -> Result<(), crate::Error> {
        todo!("poll for ceph healthy")
    }

    async fn timeout(&self) -> Result<(), crate::Error> {
        todo!("unset noout")
    }

    fn timeout_interval(&self) -> Duration {
        Duration::from_secs(60 * 15)
    }

    fn poll_interval(&self) -> Duration {
        Duration::from_secs(10)
    }

    fn priority(&self) -> super::Priority {
        todo!()
    }
}
