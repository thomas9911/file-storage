use tokio::sync::oneshot;
use warp::filters::path::Tail;
use warp::path::{param, tail};
use warp::Filter;

pub type GeneralResult<T> = Result<T, Box<dyn std::error::Error>>;

mod backend;

fn basic_endpoint(client: backend::Client) -> warp::filters::BoxedFilter<(impl warp::Reply,)> {
    let cloned_client = client.clone();
    let create_bucket_endpoint = warp::any()
        .map(move || cloned_client.clone())
        .and(warp::filters::path::param::<String>())
        .and(warp::path::end())
        .and(warp::post())
        .and_then(backend::create_bucket);

    let cloned_client = client.clone();
    let delete_bucket_endpoint = warp::any()
        .map(move || cloned_client.clone())
        .and(warp::filters::path::param::<String>())
        .and(warp::query::<backend::DeleteBucketOptions>())
        .and(warp::path::end())
        .and(warp::delete())
        .and_then(backend::delete_bucket);

    let cloned_client = client.clone();
    let create_object_endpoint = warp::any()
        .map(move || cloned_client.clone())
        // .and(warp::path!(String / String))
        .and(param())
        .and(tail())
        .and(warp::post())
        .and(warp::header::<String>("content-type"))
        .and(warp::filters::body::stream())
        .and_then(backend::create_object);

    let cloned_client = client.clone();
    let delete_object_endpoint = warp::any()
        .map(move || cloned_client.clone())
        // .and(warp::path!(String / String))
        .and(param())
        .and(tail())
        .and(warp::delete())
        .and_then(backend::delete_object);

    let cloned_client = client.clone();
    let get_object_endpoint = warp::any()
        .map(move || cloned_client.clone())
        // .and(warp::path!(String / String))
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

    let address = std::net::SocketAddr::from(([127, 0, 0, 1], 3030));

    let client = backend::make_client().await?;
    backend::setup(&client).await?;
    let basic_route = warp::path("basic").and(basic_endpoint(client));
    let routes = warp::path("api").and(basic_route);

    let (addr, server) = warp::serve(routes).bind_with_graceful_shutdown(address, async {
        tokio::signal::ctrl_c().await.ok();
    });
    log::info!("running on {}", addr);

    server.await;

    Ok(())
}
