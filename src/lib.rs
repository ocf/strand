pub mod strategies;
pub mod lock;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("kubernetes api error")]
    Kube(#[from] kube::Error),
    #[error("bad value: {0}")]
    Value(String),
    #[error("lock held by {0}")]
    Held(String),
    #[error("unknown data store error")]
    Unknown,
}
