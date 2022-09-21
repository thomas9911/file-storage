use warp::http::{Method, StatusCode};
use warp::path::{param, tail};
use warp::{Filter, Rejection};

pub type GeneralResult<T> = Result<T, Box<dyn std::error::Error>>;

pub mod backend;
pub mod basic;
pub mod config;
pub mod context;

use backend::Client;
use config::Config;
use context::Context;

#[tokio::main]
async fn main() -> GeneralResult<()> {
    pretty_env_logger::init();
    let config = Config::global();
    println!("{:?}", config);

    let client = backend::make_client().await?;
    backend::setup(&client).await?;
    let basic_route = warp::path("basic").and(basic::basic_endpoint(client));
    let routes = warp::path("api").and(basic_route);

    let (addr, server) = warp::serve(routes).bind_with_graceful_shutdown(config.address, async {
        tokio::signal::ctrl_c().await.ok();
        log::info!("shutting down");
    });
    log::info!("running on {}", addr);

    server.await;

    Ok(())
}
