use std::time::Duration;

use k8s_openapi::api::core::v1::Node;
use k8s_openapi::api::core::v1::Pod;
use kube::ResourceExt;
use kube::api::EvictParams;
use kube::api::ListParams;
use kube::api::Patch;
use kube::api::PatchParams;
use kube::Api;
use kube::Client;

use crate::strategies::Priority;
use crate::strategies::Strategy;

struct Kubernetes {
    node: String,
    client: Client,
}

impl Kubernetes {
    pub async fn new(node: String) -> Result<Self, crate::Error> {
        Ok(Kubernetes {
            node,
            client: Client::try_default().await?,
        })
    }
}

#[async_trait::async_trait]
impl Strategy for Kubernetes {
    async fn pre_reboot(&self) -> Result<(), crate::Error> {
        // cordon the node
        let node_api: Api<Node> = Api::all(self.client.clone());
        node_api.cordon(&self.node).await?;

        // evict pods
        let pod_api: Api<Pod> = Api::all(self.client.clone());
        let pods = pod_api.list(&ListParams::default().fields(&format!("spec.nodeName={}", &self.node))).await?;
        for pod in pods {
            let is_daemonset_managed = pod.owner_references().iter().any(|x| x.kind.eq("DaemonSet"));
            if !is_daemonset_managed {
                let status = pod_api.evict(&pod.name_any(), &EvictParams::default()).await?;
                if status.is_failure() {
                    return Err(crate::Error::Value(format!("failed to evict {}", pod.name_any())));
                }
            }
        }

        Ok(())
    }

    async fn post_reboot(&self) -> Result<(), crate::Error> {
        let node_api: Api<Node> = Api::all(self.client.clone());
        node_api.uncordon(&self.node).await?;

        todo!()
    }

    async fn timeout(&self) -> Result<(), crate::Error> {
        // No cleanup required, stay cordoned
        Ok(())
    }

    fn timeout_interval(&self) -> std::time::Duration {
        Duration::from_secs(60 * 10)
    }

    fn poll_interval(&self) -> std::time::Duration {
        Duration::from_secs(10)
    }

    fn priority(&self) -> Priority {
        Priority {
            pre: 100,
            post: 300,
        }
    }
}
