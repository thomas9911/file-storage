pub mod auth;

use warp::http::{Method, StatusCode};
use warp::path::{param, tail};
use warp::{Filter, Rejection};

use crate::backend::{Client, Unauthorised};
use crate::context::Context;

const POST_METHOD: Method = warp::http::Method::POST;
const GET_METHOD: Method = warp::http::Method::GET;
const DELETE_METHOD: Method = warp::http::Method::DELETE;

fn with_base(
    client: Client,
    method: &'static Method,
) -> impl Filter<Extract = (Context,), Error = warp::Rejection> + Clone {
    warp::any()
        .map(move || client.clone())
        .map(move |client| (client, method))
        .untuple_one()
        .and(warp::header::optional::<String>("authorization"))
        .then(Context::from_auth_header)
}

async fn handle_rejection(err: Rejection) -> Result<impl warp::Reply, std::convert::Infallible> {
    if err.is_not_found() {
        Ok(warp::reply::with_status(
            "NOT_FOUND".to_string(),
            StatusCode::NOT_FOUND,
        ))
    } else if let Some(e) = err.find::<Unauthorised>() {
        Ok(warp::reply::with_status(
            serde_json::json!({"error": e.reason}).to_string(),
            StatusCode::UNAUTHORIZED,
        ))
    } else {
        eprintln!("unhandled rejection: {:?}", err);
        Ok(warp::reply::with_status(
            "INTERNAL_SERVER_ERROR".to_string(),
            StatusCode::INTERNAL_SERVER_ERROR,
        ))
    }
}

pub fn basic_endpoint(client: Client) -> warp::filters::BoxedFilter<(impl warp::Reply,)> {
    let create_bucket_endpoint = warp::any()
        .and(with_base(client.clone(), &POST_METHOD))
        .and(warp::filters::path::param::<String>())
        .and(warp::path::end())
        .and(warp::post())
        .and_then(crate::backend::create_bucket);

    let delete_bucket_endpoint = warp::any()
        .and(with_base(client.clone(), &DELETE_METHOD))
        .and(warp::filters::path::param::<String>())
        .and(warp::query::<crate::backend::DeleteBucketOptions>())
        .and(warp::path::end())
        .and(warp::delete())
        .and_then(crate::backend::delete_bucket);

    let create_object_endpoint = warp::any()
        .and(with_base(client.clone(), &POST_METHOD))
        .and(param())
        .and(tail())
        .and(warp::post())
        .and(warp::header::<String>("content-type"))
        .and(warp::filters::body::stream())
        .and_then(crate::backend::create_object);

    let delete_object_endpoint = warp::any()
        .and(with_base(client.clone(), &DELETE_METHOD))
        .and(param())
        .and(tail())
        .and(warp::delete())
        .and_then(crate::backend::delete_object);

    let get_object_endpoint = warp::any()
        .and(with_base(client, &GET_METHOD))
        .and(param())
        .and(tail())
        .and(warp::get())
        .and_then(crate::backend::get_object);

    let basic_endpoint = create_bucket_endpoint
        .or(delete_bucket_endpoint)
        .or(create_object_endpoint)
        .or(delete_object_endpoint)
        .or(get_object_endpoint);

    basic_endpoint.recover(handle_rejection).boxed()
}
