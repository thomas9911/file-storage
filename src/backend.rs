use crate::GeneralResult;
use warp::filters::path::Tail;
use warp::Rejection;

mod mongodb;

#[derive(Debug, serde::Deserialize)]
pub struct DeleteBucketOptions {
    purge: Option<bool>,
}

pub type Client = mongodb::Client;
type CreateBucketResult = mongodb::CreateBucketResult;
type DeleteBucketResult = mongodb::DeleteBucketResult;
type CreateObjectResult = mongodb::CreateObjectResult;
type DeleteObjectResult = mongodb::DeleteObjectResult;

pub async fn setup(client: &Client) -> GeneralResult<()> {
    mongodb::setup(client).await
}

pub async fn make_client() -> GeneralResult<Client> {
    mongodb::make_client().await
}

pub async fn create_bucket(
    client: Client,
    bucket_name: String,
) -> Result<CreateBucketResult, Rejection> {
    mongodb::create_bucket(client, bucket_name).await
}

pub async fn delete_bucket(
    client: Client,
    bucket_name: String,
    options: DeleteBucketOptions,
) -> Result<DeleteBucketResult, Rejection> {
    mongodb::delete_bucket(client, bucket_name, options).await
}

pub async fn create_object(
    client: Client,
    bucket_name: String,
    object_name: Tail,
    content_type: String,
    buffer: impl futures::Stream<Item = Result<impl warp::Buf, warp::Error>>,
) -> Result<CreateObjectResult, Rejection> {
    mongodb::create_object(
        client,
        bucket_name,
        object_name.as_str().to_string(),
        content_type,
        buffer,
    )
    .await
}

pub async fn get_object(
    client: Client,
    bucket_name: String,
    object_name: Tail,
) -> Result<warp::reply::Response, Rejection> {
    mongodb::get_object(client, bucket_name, object_name.as_str().to_string()).await
}

pub async fn delete_object(
    client: Client,
    bucket_name: String,
    object_name: Tail,
) -> Result<DeleteObjectResult, Rejection> {
    mongodb::delete_object(client, bucket_name, object_name.as_str().to_string()).await
}
