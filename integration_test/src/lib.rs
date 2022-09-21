#[cfg(test)]
use serde_json::{json, Value};

#[cfg(test)]
const URL: &'static str = "http://localhost:3030/api/basic";

#[tokio::test]
async fn test_create_bucket() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    client.delete(format!("{}/test_bucket", URL)).send().await.unwrap();

    let res = client.post(format!("{}/test_bucket", URL)).send().await.unwrap();
    let status_code = res.status();
    let out: Value = res.json().await.unwrap();

    assert_eq!(
        json!({"bucket": "test_bucket", "created": true, "info": "OK"}),
        out
    );
    assert_eq!(reqwest::StatusCode::OK, status_code);

    let res = client.post(format!("{}/test_bucket", URL)).send().await.unwrap();
    let status_code = res.status();
    let out: Value = res.json().await.unwrap();

    assert_eq!(
        json!({"bucket": "test_bucket", "created": false, "info": "Bucket already exists"}),
        out
    );
    assert_eq!(reqwest::StatusCode::CONFLICT, status_code);

    Ok(())
}

#[tokio::test]
async fn test_delete_bucket() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    client
        .post(format!("{}/test_delete_bucket", URL))
        .send()
        .await.unwrap();

    let res = client
        .delete(format!("{}/test_delete_bucket", URL))
        .send()
        .await.unwrap();
    let status_code = res.status();
    let out: Value = res.json().await.unwrap();

    assert_eq!(json!({"bucket": "test_delete_bucket", "info": "OK"}), out);
    assert_eq!(reqwest::StatusCode::OK, status_code);

    let res = client
        .delete(format!("{}/test_delete_bucket", URL))
        .send()
        .await.unwrap();
    let status_code = res.status();
    let out: Value = res.json().await.unwrap();

    assert_eq!(json!({"bucket": "test_delete_bucket", "info": "OK"}), out);
    assert_eq!(reqwest::StatusCode::OK, status_code);

    Ok(())
}

#[tokio::test]
async fn test_create_object() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let bucket = "test_create_bucket";

    client
        .delete(format!("{}/{}?purge=true", URL, bucket))
        .send()
        .await.unwrap();

    let res = client.post(format!("{}/{}", URL, bucket)).send().await.unwrap();

    assert_eq!(reqwest::StatusCode::OK, res.status());

    let file = tokio::fs::File::open("./image.jpg").await.unwrap();
    let body = reqwest::Body::from(file);

    let res = client
        .post(format!("{}/{}/image.jpg", URL, bucket))
        .body(body)
        .header("content-type", "image/jpeg")
        .send()
        .await.unwrap();

    assert_eq!(reqwest::StatusCode::OK, res.status());
    assert_eq!(
        json!({
            "bucket": bucket,
            "info": "OK",
            "created": true,
            "filename": "image.jpg",
        }),
        res.json::<Value>().await.unwrap()
    );

    let file = tokio::fs::File::open("./image.jpg").await.unwrap();
    let body = reqwest::Body::from(file);

    let res = client
        .post(format!("{}/{}/image.jpg", URL, bucket))
        .body(body)
        .header("content-type", "image/jpeg")
        .send()
        .await.unwrap();

    assert_eq!(reqwest::StatusCode::CONFLICT, res.status());
    assert_eq!(
        json!({
            "bucket": bucket,
            "info": "Object already exists",
            "created": false,
            "filename": "image.jpg",
        }),
        res.json::<Value>().await.unwrap()
    );

    // recreate client to avoid broken connection error
    let client = reqwest::Client::new();

    let res = client.delete(format!("{}/{}", URL, bucket)).send().await.unwrap();

    assert_eq!(reqwest::StatusCode::BAD_REQUEST, res.status());
    assert_eq!(
        json!({"bucket": bucket, "info": "bucket is not empty"}),
        res.json::<Value>().await.unwrap()
    );

    let res = client
        .delete(format!("{}/{}/image.jpg", URL, bucket))
        .send()
        .await.unwrap();

    assert_eq!(reqwest::StatusCode::OK, res.status());
    assert_eq!(
        json!({
            "bucket": bucket,
            "info": "OK",
            "filename": "image.jpg",
        }),
        res.json::<Value>().await.unwrap()
    );

    let res = client
        .delete(format!("{}/{}/image.jpg", URL, bucket))
        .send()
        .await.unwrap();

    assert_eq!(reqwest::StatusCode::NOT_FOUND, res.status());
    assert_eq!(
        json!({
            "bucket": bucket,
            "info": "object not found",
            "filename": "image.jpg",
        }),
        res.json::<Value>().await.unwrap()
    );

    let res = client
        .delete(format!("{}/{}?purge=true", URL, bucket))
        .send()
        .await.unwrap();

    assert_eq!(reqwest::StatusCode::OK, res.status());
    assert_eq!(
        json!({"bucket": bucket, "info": "OK"}),
        res.json::<Value>().await.unwrap()
    );

    Ok(())
}
