use axum::{
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use kube::Client;
use strand::{
    fleetlock,
    lock::Lock,
    strategies::{self, Strategy},
};

use crate::config::CONFIG;

pub async fn check_fleetlock_header<B>(
    req: Request<B>,
    next: Next<B>,
) -> Result<Response, strand::Error> {
    let fl_header = req
        .headers()
        .get("fleet-lock-protocol")
        .and_then(|header| header.to_str().ok());

    match fl_header {
        Some(auth_header) if auth_header.eq("true") => Ok(next.run(req).await),
        _ => Err(strand::Error::Value("no fleetlock header set".into())),
    }
}

pub async fn pre_reboot_handler(
    Json(req): Json<fleetlock::Request>,
) -> Result<impl IntoResponse, strand::Error> {
    let lock: &Lock = &CONFIG.get().ok_or(strand::Error::Impossible)?.lock;
    let client: Client = Client::try_default().await?;

    lock.acquire(client.clone(), req.client_params.id.clone())
        .await?;
    let mut meta = lock.get_metadata(client.clone()).await?;

    let mut strategies = strategies::init_strategies(&req).await?;
    strategies.sort_by(|a, b| a.priority().pre.cmp(&b.priority().pre));

    for strategy in strategies {
        let prio = strategy.priority().pre;
        if prio > meta.progress_flag {
            strategy.pre_reboot().await?;
            meta.progress_flag = prio;
            lock.set_metadata(client.clone(), &meta).await?;
        }
    }

    Ok(Json(fleetlock::Response {
        kind: "lock_acquired".into(),
        value: "successfully acquired lock, ok to reboot".into(),
    }))
}

pub async fn post_reboot_handler(
    Json(req): Json<fleetlock::Request>,
) -> Result<impl IntoResponse, strand::Error> {
    let lock: &Lock = &CONFIG.get().ok_or(strand::Error::Impossible)?.lock;
    lock.release(Client::try_default().await?, req.client_params.id)
        .await?;

    Ok(Json(fleetlock::Response {
        kind: "lock_released".into(),
        value: "successfully released lock, no more retries needed".into(),
    }))
}
