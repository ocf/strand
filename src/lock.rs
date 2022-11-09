use k8s_openapi::api::coordination::v1::{Lease, LeaseSpec};
use kube::{
    api::{PostParams, DeleteParams},
    core::ObjectMeta,
    Api, Client,
};

use crate::Error;

pub struct Lock {
    pub name: String,
    pub namespace: String,
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
        // let annotations: BTreeMap<String, String> = [
        //     ("io.ocf.strand/locked".into(), "true".into()),
        //     ("io.ocf.strand/holder".into(), self.name.clone()),
        // ]
        // .into();
        let desired = Lease {
            metadata: ObjectMeta {
                name: Some(self.name.clone()),
                namespace: Some(self.namespace.clone()),
                // annotations: Some(annotations),
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

    pub async fn acquire(&self, client: Client, holder: impl AsRef<str>) -> Result<(), crate::Error> {
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

    pub async fn release(&self, client: Client, holder: impl AsRef<str>) -> Result<(), crate::Error> {
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

        lease_api.delete(&self.name, &DeleteParams::default()).await?;
        Ok(())
    }

    pub async fn force_release(&self, client: Client) -> Result<(), crate::Error> {
        let lease_api: Api<Lease> = Api::default_namespaced(client);
        lease_api.delete(&self.name, &DeleteParams::default()).await?;
        Ok(())
    }
}
