const URL: &'static str = "http://localhost:3030/api/basic";
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Body;
use std::path::Path;
use tokio_util::io::ReaderStream;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let bucket = "test";
    client
        .delete(format!("{}/{}?purge=true", URL, bucket))
        .send()
        .await?;
    client.post(format!("{}/{}", URL, bucket)).send().await?;
    let path = Path::new(r"C:\Users\Thomas\Downloads\protoc-21.5-win64.zip");
    // let path = Path::new(r"C:\Users\Thomas\Downloads\postgresql-12.3-1-windows-x64-binaries.zip");
    let file = tokio::fs::File::open(path).await?;
    let metadata = file.metadata().await?;
    let size = metadata.len();
    if size == 0 {
        return Err("empty file".into());
    }
    let mime_type =
        mime_guess::from_path(path).first_or(mime_guess::mime::APPLICATION_OCTET_STREAM);

    let file_name = path
        .file_name()
        .ok_or("invalid filename")?
        .to_str()
        .ok_or("invalid filename")?;

    let bar = ProgressBar::new(size);
    bar.set_style(ProgressStyle::with_template(
        "[{elapsed_precise}] [{bytes_per_sec:>12}] {bar:40.cyan/blue} {bytes:>7}/{total_bytes:7}",
    )?);
    let body = Body::wrap_stream(ReaderStream::new(bar.wrap_async_read(file)));

    client
        .post(format!("{}/{}/{}", URL, bucket, file_name))
        .body(body)
        .header("content-type", mime_type.essence_str())
        .send()
        .await?;

    bar.finish();

    Ok(())
}
