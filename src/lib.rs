pub mod fleetlock;
pub mod lock;
pub mod strategies;

use axum::{http::StatusCode, response::IntoResponse, Json};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("kubernetes api error")]
    Kube(#[from] kube::Error),
    #[error("bad value: {0}")]
    Value(String),
    #[error("lock held by {0}")]
    Lock(String),
    #[error("impossible error")]
    Impossible,
    #[error("unknown error")]
    Unknown,
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        let slug = match &self {
            Error::Kube(_) => "err_kube_api",
            Error::Value(_) => "err_value",
            Error::Lock(_) => "err_lock",
            Error::Unknown => "err_unknown",
            Error::Impossible => "err_impossible",
        };

        let response = fleetlock::Response {
            kind: slug.into(),
            value: format!("{:?}", &self),
        };

        match &self {
            Error::Lock(_) => (StatusCode::NOT_FOUND, Json(response)),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, Json(response)),
        }
        .into_response()
    }
}
