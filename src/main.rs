use warp::http::Method;
use warp::path::{param, tail};
use warp::Filter;

pub type GeneralResult<T> = Result<T, Box<dyn std::error::Error>>;

pub mod auth;
pub mod backend;
pub mod config;
pub mod context;

use backend::Client;
use config::Config;
use context::Context;

// const POST_METHOD: &'static str = "POST";
const POST_METHOD: Method = warp::http::Method::POST;
const GET_METHOD: Method = warp::http::Method::GET;
const DELETE_METHOD: Method = warp::http::Method::DELETE;

fn with_base(
    client: backend::Client,
    method: &'static Method,
) -> impl Filter<Extract = (Context,), Error = warp::Rejection> + Clone {
    warp::any()
        .map(move || client.clone())
        .map(move |client| (client, method))
        .untuple_one()
        .and(warp::header::optional::<String>("authorization"))
        .then(Context::from_auth_header)
}

fn basic_endpoint(client: backend::Client) -> warp::filters::BoxedFilter<(impl warp::Reply,)> {
    let create_bucket_endpoint = warp::any()
        .and(with_base(client.clone(), &POST_METHOD))
        .and(warp::filters::path::param::<String>())
        .and(warp::path::end())
        .and(warp::post())
        .and_then(backend::create_bucket);

    let delete_bucket_endpoint = warp::any()
        .and(with_base(client.clone(), &DELETE_METHOD))
        .and(warp::filters::path::param::<String>())
        .and(warp::query::<backend::DeleteBucketOptions>())
        .and(warp::path::end())
        .and(warp::delete())
        .and_then(backend::delete_bucket);

    let create_object_endpoint = warp::any()
        .and(with_base(client.clone(), &POST_METHOD))
        .and(param())
        .and(tail())
        .and(warp::post())
        .and(warp::header::<String>("content-type"))
        .and(warp::filters::body::stream())
        .and_then(backend::create_object);

    let delete_object_endpoint = warp::any()
        .and(with_base(client.clone(), &DELETE_METHOD))
        .and(param())
        .and(tail())
        .and(warp::delete())
        .and_then(backend::delete_object);

    let get_object_endpoint = warp::any()
        .and(with_base(client.clone(), &GET_METHOD))
        .and(param())
        .and(tail())
        .and(warp::get())
        .and_then(backend::get_object);

    let basic_endpoint = create_bucket_endpoint
        .or(delete_bucket_endpoint)
        .or(create_object_endpoint)
        .or(delete_object_endpoint)
        .or(get_object_endpoint);

    basic_endpoint.boxed()
}

#[tokio::main]
async fn main() -> GeneralResult<()> {
    pretty_env_logger::init();
    let config = Config::global();
    println!("{:?}", config);

    let client = backend::make_client().await?;
    backend::setup(&client).await?;
    let basic_route = warp::path("basic").and(basic_endpoint(client));
    let routes = warp::path("api").and(basic_route);

    let (addr, server) = warp::serve(routes).bind_with_graceful_shutdown(config.address, async {
        tokio::signal::ctrl_c().await.ok();
        log::info!("shutting down");
    });
    log::info!("running on {}", addr);

    server.await;

    Ok(())
}
