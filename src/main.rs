pub mod config;
pub mod handler;

use axum::{middleware, routing::post, Router};
use config::{Config, CONFIG};
use handler::{check_fleetlock_header, post_reboot_handler, pre_reboot_handler};

#[tokio::main]
async fn main() -> Result<(), strand::Error> {
    // TODO: Load config from TOML file, and do validation from config.rs.
    CONFIG
        .set(Config::default())
        .expect("Unable to set config, this should never happen.");

    let web = Router::new()
        .route("/v1/pre-reboot", post(pre_reboot_handler))
        .route("/v1/steady-state", post(post_reboot_handler))
        .route_layer(middleware::from_fn(check_fleetlock_header));

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(web.into_make_service())
        .await
        .unwrap();

    Ok(())
}
