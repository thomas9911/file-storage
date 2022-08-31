use async_compat::CompatExt;
use futures::stream::StreamExt;
use mongodb::bson::{doc, Document};
use mongodb::error::Error as MongoDBError;
use mongodb::error::ErrorKind;
use mongodb::error::WriteError;
use mongodb::error::WriteFailure;
use mongodb::options::IndexOptions;
use mongodb::IndexModel;
use mongodb::{options::ClientOptions, Client};
use mongodb_gridfs::options::GridFSFindOptions;
use mongodb_gridfs::GridFSBucket;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::io::AsyncWriteExt;
use warp::reject::{Reject, Rejection};
use warp::Filter;
use warp::Reply;

type GeneralResult<T> = Result<T, Box<dyn std::error::Error>>;

const INTERNAL_DB: &'static str = "_internal";

#[derive(Debug)]
struct Asdf {
    info: String,
}

impl Reject for Asdf {}

#[derive(Debug, Serialize, Deserialize)]
struct Bucket {
    name: String,
}

#[tokio::main]
async fn main() -> GeneralResult<()> {
    let client_options = ClientOptions::parse("mongodb://root:root@localhost:27017").await?;
    let client = Client::with_options(client_options)?;
    let db = client.database(INTERNAL_DB);
    let buckets = db.collection::<Bucket>("buckets");

    let unique_index = IndexOptions::builder().unique(true).build();
    let index = IndexModel::builder()
        .keys(doc! {"name": 1})
        .options(unique_index)
        .build();
    buckets.create_index(index, None).await?;

    // let resize_binary = warp::post()
    //     .and(warp::path("basic"))
    //     .and(warp::header::<String>("content-type"))
    //     .and(warp::filters::body::aggregate())
    //     .and(warp::query::<ResizeOptions>())
    //     .then(resize_binary_endpoint);

    let basic_endpoint = warp::any()
        .map(move || client.clone())
        .and(warp::filters::path::param::<String>())
        .and(warp::post())
        .and_then(create_bucket);

    let basic_route = warp::path("basic").and(basic_endpoint);

    let routes = basic_route;
    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;

    Ok(())
}

async fn create_bucket(client: Client, input: String) -> Result<impl Reply, Rejection> {
    let db = client.database(INTERNAL_DB);
    let buckets = db.collection::<Bucket>("buckets");
    
    match buckets.insert_one(Bucket { name: input }, None).await {
        Ok(_) => (),
        Err(MongoDBError { kind, .. })
            if matches!(
                &*kind,
                ErrorKind::Write(WriteFailure::WriteError(x)) if x.code == 11000,
            ) =>
        {
            ()
        }
        Err(e) => {
            return Err(warp::reject::custom(Asdf {
                info: e.kind.to_string(),
            }))
        }
    };

    Ok("oke")
}
