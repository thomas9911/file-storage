use crate::GeneralResult;
use serde::{Deserialize, Serialize};
use warp::filters::path::Tail;
use warp::Rejection;

mod mongodb;
use crate::Context;

#[derive(Debug, serde::Deserialize)]
pub struct DeleteBucketOptions {
    purge: Option<bool>,
}

pub type Client = mongodb::Client;
type CreateBucketResult = mongodb::CreateBucketResult;
type DeleteBucketResult = mongodb::DeleteBucketResult;
type CreateObjectResult = mongodb::CreateObjectResult;
type DeleteObjectResult = mongodb::DeleteObjectResult;

pub const EMPTY_ORGANISATION: &'static str = "general";
pub const ADMIN_ORGANISATION: &'static str = "admin_organisation";

fn empty_organisation() -> String {
    String::from(EMPTY_ORGANISATION)
}

#[derive(Debug, Serialize, Deserialize, zeroize::Zeroize, zeroize::ZeroizeOnDrop)]
pub struct KeyPair {
    access: String,
    secret: String,
    #[serde(default = "empty_organisation")]
    organisation_id: String,
}

impl KeyPair {
    pub fn new(access: String, secret: String, organisation_id: String) -> KeyPair {
        KeyPair {
            access,
            secret,
            organisation_id,
        }
    }

    pub fn access(&self) -> &str {
        &self.access
    }

    pub fn secret(&self) -> &str {
        &self.secret
    }

    pub fn organisation_id(&self) -> &str {
        &self.organisation_id
    }
}

pub async fn setup(client: &Client) -> GeneralResult<()> {
    mongodb::setup(client).await
}

pub async fn make_client() -> GeneralResult<Client> {
    mongodb::make_client().await
}

pub async fn create_bucket(
    context: Context,
    bucket_name: String,
) -> Result<CreateBucketResult, Rejection> {
    mongodb::create_bucket(context, bucket_name).await
}

pub async fn delete_bucket(
    context: Context,
    bucket_name: String,
    options: DeleteBucketOptions,
) -> Result<DeleteBucketResult, Rejection> {
    mongodb::delete_bucket(context, bucket_name, options).await
}

pub async fn create_object(
    context: Context,
    bucket_name: String,
    object_name: Tail,
    content_type: String,
    buffer: impl futures::Stream<Item = Result<impl warp::Buf, warp::Error>>,
) -> Result<CreateObjectResult, Rejection> {
    mongodb::create_object(
        context,
        bucket_name,
        object_name.as_str().to_string(),
        content_type,
        buffer,
    )
    .await
}

pub async fn get_object(
    context: Context,
    bucket_name: String,
    object_name: Tail,
) -> Result<warp::reply::Response, Rejection> {
    mongodb::get_object(context, bucket_name, object_name.as_str().to_string()).await
}

pub async fn delete_object(
    context: Context,
    bucket_name: String,
    object_name: Tail,
) -> Result<DeleteObjectResult, Rejection> {
    mongodb::delete_object(context, bucket_name, object_name.as_str().to_string()).await
}

pub async fn get_keypair_with_access_key(
    client: Client,
    access_key: String,
) -> Result<KeyPair, String> {
    mongodb::get_keypair_with_access_key(client, access_key).await
}
