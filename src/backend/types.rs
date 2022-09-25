use warp::http::header::{HeaderValue, CONTENT_TYPE};
use warp::http::StatusCode;
use warp::reject::{Reject, Rejection};
use warp::reply::Response;

#[derive(Debug)]
/// error that always raises
struct CustomError {
    info: String,
}

impl Reject for CustomError {}

/// create 500 internal server error
pub fn raises(info: String) -> Rejection {
    warp::reject::custom(CustomError { info })
}

#[derive(Debug)]
pub struct CreateBucketResult {
    pub created: bool,
    pub bucket: String,
    pub validation_error: Option<String>,
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
    pub bucket: String,
    pub message: Option<&'static str>,
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
pub enum CreateObjectValidationError {
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
    pub created: bool,
    pub bucket: String,
    pub filename: String,
    pub validation_error: Option<CreateObjectValidationError>,
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
    pub bucket: String,
    pub filename: String,
    pub message: Option<&'static str>,
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
