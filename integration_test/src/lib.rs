#[cfg(test)]
use serde_json::{json, Value};

#[cfg(test)]
const URL: &'static str = "http://localhost:3030/api/basic";

#[tokio::test]
async fn test_create_bucket() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    client.delete(format!("{}/test_bucket", URL)).send().await?;

    let res = client.post(format!("{}/test_bucket", URL)).send().await?;
    let status_code = res.status();
    let out: Value = res.json().await?;

    assert_eq!(
        json!({"bucket": "test_bucket", "created": true, "info": "OK"}),
        out
    );
    assert_eq!(reqwest::StatusCode::OK, status_code);

    let res = client.post(format!("{}/test_bucket", URL)).send().await?;
    let status_code = res.status();
    let out: Value = res.json().await?;

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
        .await?;

    let res = client
        .delete(format!("{}/test_delete_bucket", URL))
        .send()
        .await?;
    let status_code = res.status();
    let out: Value = res.json().await?;

    assert_eq!(json!({"bucket": "test_delete_bucket", "info": "OK"}), out);
    assert_eq!(reqwest::StatusCode::OK, status_code);

    let res = client
        .delete(format!("{}/test_delete_bucket", URL))
        .send()
        .await?;
    let status_code = res.status();
    let out: Value = res.json().await?;

    assert_eq!(json!({"bucket": "test_delete_bucket", "info": "OK"}), out);
    assert_eq!(reqwest::StatusCode::OK, status_code);

    Ok(())
}
