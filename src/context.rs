use crate::auth::Auth;
use crate::backend::{Client, EMPTY_ORGANISATION};
use warp::http::Method;

pub struct Context {
    pub client: Client,
    pub auth: Option<Auth>,
    pub method: &'static Method,
    pub path: String,
}

impl Context {
    // pub fn new_anon(client: Client) -> Context {
    //     Context { client, auth: None }
    // }

    pub async fn from_auth_header(
        client: Client,
        method: &'static Method,
        auth_header: Option<String>,
    ) -> Context {
        let auth = if let Some(auth) = auth_header {
            Auth::check_and_create(&client, &auth).await.ok()
        } else {
            None
        };

        log::warn!("auth {:?}", auth);

        Context {
            client,
            auth,
            method,
            path: String::new(),
        }
    }

    pub fn organisation_id<'a>(&'a self) -> &'a str {
        self.auth
            .as_ref()
            .map(|x| x.organisation_id())
            .unwrap_or(EMPTY_ORGANISATION)
    }

    pub fn is_logged_in(&self) -> bool {
        self.auth.is_some()
    }
}
