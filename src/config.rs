use crate::backend::{KeyPair, ADMIN_ORGANISATION, EMPTY_ORGANISATION};
use serde::Deserialize;
use tokio::sync::OnceCell;

static GLOBAL_CONFIG: OnceCell<Config> = OnceCell::const_new();

#[derive(Deserialize, Debug)]
#[serde(default)]
pub struct Config {
    admin_secret_key: Option<String>,
    admin_access_key: Option<String>,
    pub address: std::net::SocketAddr,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            admin_secret_key: None,
            admin_access_key: None,
            address: std::net::SocketAddr::from(([127, 0, 0, 1], 3030)),
        }
    }
}

impl Config {
    pub fn global() -> &'static Config {
        while !Config::setup_once() {}

        GLOBAL_CONFIG.get().unwrap()
    }

    /// return true if successfully set,
    /// returns false on conflict and
    /// panics on error to get env variables
    fn setup_once() -> bool {
        if GLOBAL_CONFIG.initialized() {
            true
        } else {
            match envy::prefixed("FILE_STORAGE_").from_env::<Config>() {
                Ok(config) => match GLOBAL_CONFIG.set(config) {
                    Ok(()) => true,
                    Err(tokio::sync::SetError::AlreadyInitializedError(_)) => true,
                    _ => false,
                },
                Err(error) => panic!("{:#?}", error),
            }
        }
    }

    /// returns admin keypair if configed
    pub fn admin_key() -> Option<KeyPair> {
        let config = Config::global();

        match (&config.admin_access_key, &config.admin_secret_key) {
            (Some(access_key), Some(secret)) => Some(KeyPair::new(
                access_key.to_string(),
                secret.to_string(),
                String::from(ADMIN_ORGANISATION),
            )),
            _ => None,
        }
    }
}
