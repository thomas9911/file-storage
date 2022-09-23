use crate::GeneralResult;
use serde::{Deserialize, Serialize};
use warp::filters::path::Tail;
use warp::Rejection;

pub mod mongodb;
use crate::Context;

pub use self::mongodb as implementation;

#[derive(Debug, Deserialize)]
pub struct DeleteBucketOptions {
    purge: Option<bool>,
}

pub type Client = implementation::Client;
type CreateBucketResult = implementation::CreateBucketResult;
type DeleteBucketResult = implementation::DeleteBucketResult;
type CreateObjectResult = implementation::CreateObjectResult;
type DeleteObjectResult = implementation::DeleteObjectResult;

pub const EMPTY_ORGANISATION: &str = "general";
pub const ADMIN_ORGANISATION: &str = "admin_organisation";

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

#[derive(Debug)]
pub struct Unauthorised {
    pub reason: String,
}

impl warp::reject::Reject for Unauthorised {}

fn check_auth(context: &Context) -> Result<(), Rejection> {
    if context.validate_request() {
        Ok(())
    } else {
        Err(warp::reject::custom(Unauthorised {
            reason: format!("Unauthorised for path {} {}", context.method, context.path),
        }))
    }
}

pub async fn setup(client: &Client) -> GeneralResult<()> {
    implementation::setup(client).await
}

pub async fn make_client() -> GeneralResult<Client> {
    implementation::make_client().await
}

pub async fn create_bucket(
    mut context: Context,
    bucket_name: String,
) -> Result<CreateBucketResult, Rejection> {
    context.path = bucket_name.to_string();
    check_auth(&context)?;
    implementation::create_bucket(context, bucket_name).await
}

pub async fn delete_bucket(
    mut context: Context,
    bucket_name: String,
    options: DeleteBucketOptions,
) -> Result<DeleteBucketResult, Rejection> {
    context.path = bucket_name.to_string();
    check_auth(&context)?;
    implementation::delete_bucket(context, bucket_name, options).await
}

pub async fn create_object(
    mut context: Context,
    bucket_name: String,
    object_name: Tail,
    content_type: String,
    buffer: impl futures::Stream<Item = Result<impl warp::Buf, warp::Error>>,
) -> Result<CreateObjectResult, Rejection> {
    let object_name = object_name.as_str().to_string();
    context.path = format!("{}/{}", bucket_name, object_name);
    check_auth(&context)?;
    implementation::create_object(context, bucket_name, object_name, content_type, buffer).await
}

pub async fn get_object(
    mut context: Context,
    bucket_name: String,
    object_name: Tail,
) -> Result<warp::reply::Response, Rejection> {
    let object_name = object_name.as_str().to_string();
    context.path = format!("{}/{}", bucket_name, object_name);
    check_auth(&context)?;
    implementation::get_object(context, bucket_name, object_name).await
}

pub async fn delete_object(
    mut context: Context,
    bucket_name: String,
    object_name: Tail,
) -> Result<DeleteObjectResult, Rejection> {
    let object_name = object_name.as_str().to_string();
    context.path = format!("{}/{}", bucket_name, object_name);
    check_auth(&context)?;
    implementation::delete_object(context, bucket_name, object_name).await
}

pub async fn get_keypair_with_access_key(
    client: Client,
    access_key: String,
) -> Result<KeyPair, String> {
    implementation::get_keypair_with_access_key(client, access_key).await
}
