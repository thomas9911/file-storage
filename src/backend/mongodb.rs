use crate::backend::{KeyPair, ADMIN_ORGANISATION, EMPTY_ORGANISATION};
use crate::Context;
use crate::GeneralResult;

use async_compat::CompatExt;
use futures::stream::{StreamExt, TryStreamExt};
use mongodb::bson::doc;
use mongodb::error::Error as MongoDBError;
use mongodb::error::ErrorKind;
use mongodb::error::WriteFailure;
use mongodb::options::{ClientOptions, IndexOptions};
pub use mongodb::Client;
use mongodb::IndexModel;
use mongodb_gridfs::options::{GridFSBucketOptions, GridFSFindOptions, GridFSUploadOptions};
use mongodb_gridfs::GridFSBucket;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use tokio_util::io::StreamReader;
use warp::http::header::{HeaderValue, CONTENT_TYPE};
use warp::http::StatusCode;
use warp::reject::{Reject, Rejection};
use warp::reply::Response;

const INTERNAL_DB: &str = "_internal";
const BUCKET_COLLECTION: &str = "buckets";
const KEYPAIRS_COLLECTION: &str = "keypairs";
const BUCKET_BLACKLIST: [&str; 6] = [
    INTERNAL_DB,
    EMPTY_ORGANISATION,
    ADMIN_ORGANISATION,
    "config",
    "admin",
    "local",
];

#[derive(Debug)]
struct CustomError {
    info: String,
}

impl Reject for CustomError {}

#[derive(Debug)]
pub struct CreateBucketResult {
    created: bool,
    bucket: String,
    validation_error: Option<String>,
}

impl warp::Reply for CreateBucketResult {
    fn into_response(self) -> warp::reply::Response {
        let is_validation_error = self.validation_error.is_some();
        let message = if let Some(validation_error) = self.validation_error {
            format!(
                r#"{{"created": {}, "error": {:?}}}"#,
                self.created, validation_error
            )
        } else {
            let info = if self.created {
                "OK"
            } else {
                "Bucket already exists"
            };

            format!(
                r#"{{"bucket": {:?}, "created": {}, "info": {:?}}}"#,
                self.bucket, self.created, info
            )
        };

        let mut response = Response::new(message.into());
        response
            .headers_mut()
            .insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        if self.created {
            *response.status_mut() = StatusCode::OK;
        } else if !is_validation_error {
            *response.status_mut() = StatusCode::CONFLICT;
        } else {
            *response.status_mut() = StatusCode::BAD_REQUEST;
        }

        response
    }
}

#[derive(Debug)]
pub struct DeleteBucketResult {
    bucket: String,
    message: Option<&'static str>,
}

impl warp::Reply for DeleteBucketResult {
    fn into_response(self) -> warp::reply::Response {
        let info = if let Some(message) = self.message {
            message
        } else {
            "OK"
        };

        let message = format!(r#"{{"bucket": {:?}, "info": {:?}}}"#, self.bucket, info);

        let mut response = Response::new(message.into());
        response
            .headers_mut()
            .insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        if self.message.is_some() {
            *response.status_mut() = StatusCode::BAD_REQUEST;
        } else {
            *response.status_mut() = StatusCode::OK;
        };

        response
    }
}

#[derive(Debug)]
enum CreateObjectValidationError {
    BucketNotFound,
}

impl std::fmt::Display for CreateObjectValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CreateObjectValidationError::BucketNotFound => write!(f, "Bucket not found"),
        }
    }
}

#[derive(Debug)]
pub struct CreateObjectResult {
    created: bool,
    bucket: String,
    filename: String,
    validation_error: Option<CreateObjectValidationError>,
}

impl warp::Reply for CreateObjectResult {
    fn into_response(self) -> warp::reply::Response {
        let message = if let Some(validation_error) = &self.validation_error {
            format!(
                r#"{{"created": {}, "error": {:?}}}"#,
                self.created, validation_error
            )
        } else {
            let info = if self.created {
                "OK"
            } else {
                "Object already exists"
            };

            format!(
                r#"{{"bucket": {:?}, "filename": {:?}, "created": {}, "info": {:?}}}"#,
                self.bucket, self.filename, self.created, info
            )
        };

        let mut response = Response::new(message.into());
        response
            .headers_mut()
            .insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        match self.validation_error {
            None if self.created => {
                *response.status_mut() = StatusCode::OK;
            }
            None => {
                *response.status_mut() = StatusCode::CONFLICT;
            }
            Some(CreateObjectValidationError::BucketNotFound) => {
                *response.status_mut() = StatusCode::NOT_FOUND;
            }
        }

        response
    }
}

#[derive(Debug)]
pub struct DeleteObjectResult {
    bucket: String,
    filename: String,
    message: Option<&'static str>,
}

impl warp::Reply for DeleteObjectResult {
    fn into_response(self) -> warp::reply::Response {
        let info = if let Some(message) = self.message {
            message
        } else {
            "OK"
        };

        let message = format!(
            r#"{{"bucket": {:?}, "filename": {:?}, "info": {:?}}}"#,
            self.bucket, self.filename, info
        );

        let mut response = Response::new(message.into());
        response
            .headers_mut()
            .insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        match self.message {
            Some("object not found") => {
                *response.status_mut() = StatusCode::NOT_FOUND;
            }
            Some(_) => {
                *response.status_mut() = StatusCode::BAD_REQUEST;
            }
            None => {
                *response.status_mut() = StatusCode::OK;
            }
        }

        response
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Bucket {
    name: String,
}

pub async fn setup(client: &Client) -> GeneralResult<()> {
    let db = client.database(INTERNAL_DB);
    let buckets = db.collection::<Bucket>(BUCKET_COLLECTION);

    let unique_index = IndexOptions::builder().unique(true).build();
    let index = IndexModel::builder()
        .keys(doc! {"name": 1})
        .options(unique_index)
        .build();
    buckets.create_index(index, None).await?;

    let keypairs = db.collection::<KeyPair>(KEYPAIRS_COLLECTION);

    let unique_index = IndexOptions::builder().unique(true).build();
    let index = IndexModel::builder()
        .keys(doc! {"access": 1})
        .options(unique_index)
        .build();
    keypairs.create_index(index, None).await?;

    Ok(())
}

fn validate_bucket_name(bucket_name: &str) -> Result<(), CreateBucketResult> {
    if bucket_name.is_empty() {
        return Err(CreateBucketResult {
            bucket: bucket_name.to_string(),
            created: false,
            validation_error: Some(String::from("invalid bucket name, empty bucket name")),
        });
    };

    if bucket_name.len() > 100 {
        return Err(CreateBucketResult {
            bucket: "".to_string(),
            created: false,
            validation_error: Some(String::from("invalid bucket name, bucket name too long")),
        });
    };

    if BUCKET_BLACKLIST.contains(&bucket_name) {
        return Err(CreateBucketResult {
            bucket: bucket_name.to_string(),
            created: false,
            validation_error: Some(String::from("invalid bucket name, blacklisted name")),
        });
    }

    Ok(())
}

async fn inner_create_bucket(context: Context, bucket_name: String) -> mongodb::error::Result<()> {
    context
        .client
        .database(INTERNAL_DB)
        .collection::<Bucket>(BUCKET_COLLECTION)
        .insert_one(
            Bucket {
                name: bucket_name.to_string(),
            },
            None,
        )
        .await?;

    let db = context.client.database(context.organisation_id());
    let buckets = db.collection::<()>(&format!("{}.files", bucket_name));

    let unique_index = IndexOptions::builder().unique(true).build();
    let index = IndexModel::builder()
        .keys(doc! {"filename": 1})
        .options(unique_index)
        .build();
    buckets.create_index(index, None).await?;

    Ok(())
}

pub async fn create_bucket<'a>(
    context: Context,
    bucket_name: String,
) -> Result<CreateBucketResult, Rejection> {
    match validate_bucket_name(&bucket_name) {
        Ok(()) => (),
        Err(e) => return Ok(e),
    };

    let created = match inner_create_bucket(context, bucket_name.to_string()).await {
        Ok(_) => true,
        // uniqueness error
        Err(MongoDBError { kind, .. })
            if matches!(
                &*kind,
                ErrorKind::Write(WriteFailure::WriteError(x)) if x.code == 11000,
            ) =>
        {
            false
        }
        Err(e) => {
            return Err(warp::reject::custom(CustomError {
                info: e.kind.to_string(),
            }))
        }
    };

    Ok(CreateBucketResult {
        bucket: bucket_name,
        created,
        validation_error: None,
    })
}

async fn delete_gridfs_collections(
    client: &Client,
    database_name: &str,
    bucket_name: &str,
) -> Result<(), mongodb::error::Error> {
    let db = client.database(database_name);

    db.collection::<Bucket>(&format!("{}.files", bucket_name))
        .drop(None)
        .await?;
    db.collection::<Bucket>(&format!("{}.chunks", bucket_name))
        .drop(None)
        .await?;

    Ok(())
}

async fn inner_delete_bucket(
    context: Context,
    bucket_name: &str,
    options: &crate::backend::DeleteBucketOptions,
) -> Result<Option<&'static str>, mongodb::error::Error> {
    if options.purge.unwrap_or(false) {
        delete_gridfs_collections(&context.client, context.organisation_id(), bucket_name).await?;
    } else {
        // check if bucket is empty
        let db = context.client.database(context.organisation_id());
        let bucket_options = GridFSBucketOptions::builder()
            .bucket_name(bucket_name.to_string())
            .build();
        let bucket = GridFSBucket::new(db, Some(bucket_options));

        let mut cursor = bucket
            .find(doc! {}, GridFSFindOptions::builder().limit(Some(1)).build())
            .await?;

        if cursor.next().await.is_some() {
            return Ok(Some("bucket is not empty"));
        } else {
            delete_gridfs_collections(&context.client, context.organisation_id(), bucket_name)
                .await?;
        }
    }

    context
        .client
        .database(INTERNAL_DB)
        .collection::<Bucket>(BUCKET_COLLECTION)
        .delete_one(doc! { "name": bucket_name.to_string() }, None)
        .await?;

    Ok(None)
}

pub async fn delete_bucket(
    context: Context,
    bucket_name: String,
    options: crate::backend::DeleteBucketOptions,
) -> Result<DeleteBucketResult, Rejection> {
    match inner_delete_bucket(context, &bucket_name, &options).await {
        Ok(Some(message)) => {
            return Ok(DeleteBucketResult {
                bucket: bucket_name,
                message: Some(message),
            })
        }
        Ok(_) => (),
        Err(e) => {
            return Err(warp::reject::custom(CustomError {
                info: e.kind.to_string(),
            }))
        }
    };

    Ok(DeleteBucketResult {
        bucket: bucket_name,
        message: None,
    })
}

pub async fn create_object(
    context: Context,
    bucket_name: String,
    object_name: String,
    content_type: String,
    buffer: impl futures::Stream<Item = Result<impl warp::Buf, warp::Error>>,
) -> Result<CreateObjectResult, Rejection> {
    let buckets = context
        .client
        .database(INTERNAL_DB)
        .collection::<Bucket>(BUCKET_COLLECTION);

    match buckets
        .find_one(
            doc! {
                "name": bucket_name.to_string(),
            },
            None,
        )
        .await
    {
        Ok(Some(bucket)) => bucket,
        Ok(None) => {
            return Ok(CreateObjectResult {
                bucket: bucket_name,
                filename: object_name,
                created: false,
                validation_error: Some(CreateObjectValidationError::BucketNotFound),
            })
        }
        Err(e) => {
            return Err(warp::reject::custom(CustomError {
                info: e.kind.to_string(),
            }))
        }
    };

    let db = context.client.database(context.organisation_id());
    let bucket_options = GridFSBucketOptions::builder()
        .bucket_name(bucket_name.to_string())
        .build();
    let mut bucket = GridFSBucket::new(db, Some(bucket_options));

    let reader = Box::pin(
        StreamReader::new(buffer.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e)))
            .compat(),
    );

    let upload_options = GridFSUploadOptions::builder()
        .metadata(Some(doc! {"contentType": content_type}))
        .build();

    let created = match bucket
        .upload_from_stream(&object_name, reader, Some(upload_options))
        .await
    {
        Ok(_) => true,
        // uniqueness error
        Err(MongoDBError { kind, .. })
            if matches!(
                &*kind,
                ErrorKind::Write(WriteFailure::WriteError(x)) if x.code == 11000,
            ) =>
        {
            false
        }
        Err(e) => {
            return Err(warp::reject::custom(CustomError {
                info: e.kind.to_string(),
            }))
        }
    };

    Ok(CreateObjectResult {
        bucket: bucket_name,
        filename: object_name,
        created,
        validation_error: None,
    })
}

pub async fn get_object(
    context: Context,
    bucket_name: String,
    object_name: String,
) -> Result<warp::reply::Response, Rejection> {
    let db = context.client.database(context.organisation_id());
    let bucket_options = GridFSBucketOptions::builder()
        .bucket_name(bucket_name.to_string())
        .build();
    let bucket = GridFSBucket::new(db, Some(bucket_options));

    let mut cursor = bucket
        .find(
            doc! {"filename": object_name},
            GridFSFindOptions::builder().limit(Some(1)).build(),
        )
        .await
        .map_err(|e| {
            warp::reject::custom(CustomError {
                info: e.to_string(),
            })
        })?;

    let id = if let Some(Ok(object_doc)) = cursor.next().await {
        object_doc
            .get_object_id("_id")
            .expect("all documentent have _id")
    } else {
        return Err(warp::reject::not_found());
    };

    let (cursor, _filename) = bucket
        .open_download_stream_with_filename(id)
        .await
        .map_err(|e| {
            warp::reject::custom(CustomError {
                info: e.to_string(),
            })
        })?;
    let stream = warp::hyper::body::Body::wrap_stream(cursor.map::<Result<_, Infallible>, _>(Ok));

    Ok(warp::reply::Response::new(stream))
}

pub async fn delete_object(
    context: Context,
    bucket_name: String,
    object_name: String,
) -> Result<DeleteObjectResult, Rejection> {
    let db = context.client.database(context.organisation_id());
    let bucket_options = GridFSBucketOptions::builder()
        .bucket_name(bucket_name.to_string())
        .build();
    let bucket = GridFSBucket::new(db, Some(bucket_options));

    let mut cursor = bucket
        .find(
            doc! {"filename": &object_name},
            GridFSFindOptions::builder().limit(Some(1)).build(),
        )
        .await
        .map_err(|e| {
            warp::reject::custom(CustomError {
                info: e.to_string(),
            })
        })?;

    let id = if let Some(Ok(object_doc)) = cursor.next().await {
        object_doc
            .get_object_id("_id")
            .expect("all documentent have _id")
    } else {
        return Ok(DeleteObjectResult {
            bucket: bucket_name,
            filename: object_name,
            message: Some("object not found"),
        });
    };

    bucket.delete(id).await.map_err(|e| {
        warp::reject::custom(CustomError {
            info: e.to_string(),
        })
    })?;

    Ok(DeleteObjectResult {
        bucket: bucket_name,
        filename: object_name,
        message: None,
    })
}

pub async fn get_keypair_with_access_key(
    client: Client,
    access_key: String,
) -> Result<KeyPair, String> {
    let keypairs = client
        .database(INTERNAL_DB)
        .collection::<KeyPair>(KEYPAIRS_COLLECTION);

    match keypairs
        .find_one(
            doc! {
                "access": access_key.to_string(),
            },
            None,
        )
        .await
    {
        Ok(Some(keypair)) => Ok(keypair),
        Ok(None) => Err(String::from("access key not found")),
        Err(e) => Err(e.to_string()),
    }
}

pub async fn make_client() -> crate::GeneralResult<Client> {
    let client_options = ClientOptions::parse("mongodb://root:root@localhost:27017").await?;
    let client = Client::with_options(client_options)?;
    Ok(client)
}
