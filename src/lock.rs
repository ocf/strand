use std::collections::BTreeMap;

use k8s_openapi::api::coordination::v1::{Lease, LeaseSpec};
use kube::{
    api::{DeleteParams, PostParams, Patch, PatchParams},
    core::ObjectMeta,
    Api, Client,
};

use crate::Error;

pub struct Lock {
    pub name: String,
    pub namespace: String,
}

pub struct LockMetadata {
    pub holder: String,
    pub progress_flag: u64,
}

impl Lock {
    async fn get_lease(&self, api: Api<Lease>) -> Result<Lease, kube::Error> {
        api.get(&self.name).await
    }

    async fn get_or_create_lease(
        &self,
        client: Client,
        holder: &str,
    ) -> Result<Lease, kube::Error> {
        // try to get the lease if it already exists
        let lease_api: Api<Lease> = Api::default_namespaced(client);
        let mut lease: Result<Lease, kube::Error> = self.get_lease(lease_api.clone()).await;

        // the desired lease
        let annotations: BTreeMap<String, String> =
            [("io.ocf.strand/progress".into(), "0".into())].into();
        let desired = Lease {
            metadata: ObjectMeta {
                name: Some(self.name.clone()),
                namespace: Some(self.namespace.clone()),
                annotations: Some(annotations),
                ..ObjectMeta::default()
            },
            spec: Some(LeaseSpec {
                holder_identity: Some(holder.to_string()),
                ..LeaseSpec::default()
            }),
        };

        // if the lease doesn't exist, create it
        if let Err(err) = &lease {
            if let kube::Error::Api(apierr) = &err {
                if apierr.reason.eq("NotFound") {
                    lease = lease_api.create(&PostParams::default(), &desired).await;
                }
            }
        }

        Ok(lease?)
    }

    pub async fn acquire(
        &self,
        client: Client,
        holder: impl AsRef<str>,
    ) -> Result<(), crate::Error> {
        let lease = self.get_or_create_lease(client, holder.as_ref()).await?;
        let current_holder = lease
            .spec
            .ok_or(Error::Value("Lease has no spec field".into()))?
            .holder_identity
            .ok_or(Error::Value("Lease has no holder_identity".into()))?;

        if !current_holder.eq(holder.as_ref()) {
            return Err(Error::Held(current_holder));
        }

        Ok(())
    }

    pub async fn release(
        &self,
        client: Client,
        holder: impl AsRef<str>,
    ) -> Result<(), crate::Error> {
        let lease_api: Api<Lease> = Api::default_namespaced(client);
        let lease = self.get_lease(lease_api.clone()).await?;

        let current_holder = lease
            .spec
            .ok_or(Error::Value("Lease has no spec field".into()))?
            .holder_identity
            .ok_or(Error::Value("Lease has no holder_identity".into()))?;

        if !current_holder.eq(holder.as_ref()) {
            return Err(Error::Held(current_holder));
        }

        lease_api
            .delete(&self.name, &DeleteParams::default())
            .await?;
        Ok(())
    }

    pub async fn get_metadata(&self, client: Client) -> Result<LockMetadata, crate::Error> {
        let api: Api<Lease> = Api::default_namespaced(client);
        let lease = self.get_lease(api).await?;

        let current_holder = lease
            .spec
            .ok_or(Error::Value("Lease has no spec field".into()))?
            .holder_identity
            .ok_or(Error::Value("Lease has no holder_identity".into()))?;
        let progress: u64 = lease
            .metadata
            .annotations
            .ok_or(Error::Value(
                "Lease doesn't have progress annotations".into(),
            ))?
            .get("io.ocf.strand/progress")
            .ok_or(Error::Value(
                "Lease doesn't have progress annotation".into(),
            ))?
            .parse()
            .map_err(|_| Error::Value("Unable to parse progress flags".into()))?;

        Ok(LockMetadata {
            holder: current_holder,
            progress_flag: progress,
        })
    }

    pub async fn set_metadata(&self, client: Client, meta: &LockMetadata) -> Result<(), crate::Error> {
        let api: Api<Lease> = Api::default_namespaced(client.clone());
        let cur_meta = self.get_metadata(client).await?;

        if cur_meta.holder.ne(&meta.holder) {
            return Err(Error::Value("Lock isn't held by given owner".into()));
        }

        let patch = Patch::Strategic(serde_json::json!({
            "metadata": {
                "annotations": {
                    "io.ocf.strand/progress": meta.progress_flag.to_string(),
                }
            }
        }));
        api.patch(&self.name, &PatchParams::default(), &patch).await?;

        Ok(())
    }

    pub async fn force_release(&self, client: Client) -> Result<(), crate::Error> {
        let lease_api: Api<Lease> = Api::default_namespaced(client);
        lease_api
            .delete(&self.name, &DeleteParams::default())
            .await?;
        Ok(())
    }
}
