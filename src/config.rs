use serde::{Deserialize, Serialize};
use strand::lock::Lock;
use tokio::sync::OnceCell;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub api_version: String,
    pub kind: String,
    pub lock: Lock,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            api_version: "v1alpha1".into(),
            kind: "StrandConfig".into(),
            lock: Lock {
                name: "zincati-strand-lock".into(),
                namespace: "strand".into(),
            },
        }
    }
}

pub static CONFIG: OnceCell<Config> = OnceCell::const_new();
