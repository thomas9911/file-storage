use crate::backend::KeyPair;
use crate::config::Config;
use crate::Client;

use jsonwebtoken::{Algorithm, DecodingKey, Validation};

#[derive(Debug, serde::Deserialize)]
struct SimplePayload {
    sub: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct Payload {
    jti: String,
    sub: String,
    path: String,
    method: String,
}

fn b64_decode<T: AsRef<[u8]>>(input: T) -> Result<Vec<u8>, String> {
    base64::decode_config(input, base64::URL_SAFE_NO_PAD).map_err(|e| e.to_string())
}

fn get_sub_from_jwt<'a>(jwt: &'a str) -> Result<String, String> {
    let mut iter = jwt.split('.');
    match (iter.next(), iter.next(), iter.next(), iter.next()) {
        (Some(_), Some(payload), Some(_), None) => {
            let a = b64_decode(payload)?;
            let data: SimplePayload =
                serde_json::from_slice(&a).map_err(|_| String::from("jwt payload is not json"))?;
            Ok(data.sub)
        }
        _ => Err(String::from("invalid jwt")),
    }
}

async fn get_secret_from_sub(client: &Client, sub: String) -> Result<KeyPair, String> {
    if let Some(x) = get_admin_secret(&sub) {
        Ok(x)
    } else {
        crate::backend::get_keypair_with_access_key(client.clone(), sub).await
    }
}

fn get_admin_secret(sub: &str) -> Option<KeyPair> {
    // if let Ok(access_key) = std::env::var("FILE_STORAGE_ACCESS_KEY") {
    //     if access_key == sub {
    //         if let Ok(secret) = std::env::var("FILE_STORAGE_SECRET_KEY") {
    //             return Some(KeyPair::new(access_key, secret, String::from("admin")));
    //         }
    //     }
    // };

    // None
    let keypair = Config::admin_key()?;

    if keypair.access() == sub {
        Some(keypair)
    } else {
        None
    }
}

#[derive(Debug)]
pub struct Auth {
    payload: Payload,
    keypair: KeyPair,
}

impl Auth {
    pub async fn check_and_create(client: &Client, input: &str) -> Result<Auth, String> {
        let auth = match input {
            x if x.starts_with("Bearer ") => &input[7..],
            x if x.starts_with("bearer ") => &input[7..],
            &_ => return Err(String::from("invalid authorization header")),
        };
        let sub = get_sub_from_jwt(auth)?;
        let keypair = get_secret_from_sub(client, sub).await?;
        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_required_spec_claims(&["exp", "sub", "nbf"]);
        let token_message = jsonwebtoken::decode::<Payload>(
            &auth,
            &DecodingKey::from_secret(keypair.secret().as_bytes()),
            &validation,
        )
        .map_err(|e| e.to_string())?;

        Ok(Auth {
            payload: token_message.claims,
            keypair: keypair,
        })
    }

    pub fn validate_request(&self, method: &str, path: &str) -> bool {
        if self.payload.path == path && self.payload.method.to_ascii_uppercase() == method {
            true
        } else {
            false
        }
    }

    pub fn auth(&self) -> &Payload {
        &self.payload
    }

    pub fn organisation_id(&self) -> &str {
        self.keypair.organisation_id()
    }
}
