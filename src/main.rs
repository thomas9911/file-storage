use async_compat::CompatExt;
use futures::stream::StreamExt;
use mongodb::bson::{doc, Document};
use mongodb::{options::ClientOptions, Client};
use mongodb_gridfs::options::GridFSFindOptions;
use mongodb_gridfs::GridFSBucket;
use std::path::PathBuf;
use tokio::io::AsyncWriteExt;

type GeneralResult = Result<(), Box<dyn std::error::Error>>;

#[tokio::main]
async fn main() -> GeneralResult {
    let client_options =
        ClientOptions::parse("mongodb://my_user:password123@localhost:27017/my_database").await?;
    let client = Client::with_options(client_options)?;
    let db = client.database("my_database");

    let mut bucket = GridFSBucket::new(db, None);
    bucket.drop().await?;

    let name = "text.txt";

    save_file(&mut bucket, name).await?;
    get_file(&bucket, name).await?;

    Ok(())
}

async fn save_file(bucket: &mut GridFSBucket, name: &str) -> GeneralResult {
    let mut path = PathBuf::from("in_folder");
    path.push(name);

    let file = tokio::fs::File::open(path).await?;

    let _id = bucket.upload_from_stream(name, file.compat(), None).await?;

    Ok(())
}

async fn get_file(bucket: &GridFSBucket, name: &str) -> GeneralResult {
    let mut cursor = bucket
        .find(doc! {"filename": name}, GridFSFindOptions::default())
        .await?;

    while let Some(doc) = cursor.next().await {
        // dbg!(&doc);
        if let Err(e) = store_to_path(&bucket, "tmp_folder".into(), doc).await {
            println!("error: {}", e)
        };
    }

    Ok(())
}

async fn store_to_path(
    bucket: &GridFSBucket,
    mut folder: PathBuf,
    doc: Result<Document, mongodb::error::Error>,
) -> GeneralResult {
    let doc = doc?;
    let id = doc.get_object_id("_id")?;
    let file_length = doc.get_i64("length").unwrap_or(0);
    let (mut cursor, filename) = bucket.open_download_stream_with_filename(id).await?;

    folder.push(filename);
    let mut file = tokio::fs::File::create(folder).await?;
    while let Some(buffer) = cursor.next().await {
        file.write_all(&buffer).await?;
        file.flush().await?;
    }

    let metadata = file.metadata().await?;
    if metadata.len() != (file_length as u64) {
        return Err("File stored and file written have different size".into());
    }

    Ok(())
}
