use crate::backend::{Client, EMPTY_ORGANISATION};
use crate::basic::auth::Auth;
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

        Context {
            client,
            auth,
            method,
            path: String::new(),
        }
    }

    pub fn organisation_id(&self) -> &str {
        self.auth
            .as_ref()
            .map(|x| x.organisation_id())
            .unwrap_or(EMPTY_ORGANISATION)
    }

    pub fn is_logged_in(&self) -> bool {
        self.auth.is_some()
    }

    pub fn validate_request(&self) -> bool {
        if self.is_logged_in() {
            self.auth
                .as_ref()
                .unwrap()
                .validate_request(self.method.as_str(), &self.path)
        } else {
            // do we allow public uploads?
            true
        }
    }
}
