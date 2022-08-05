use std::sync::Mutex;

use common_local::{ConfigEmail, AuthConfig, ConfigServer, SharedConfig};
use lazy_static::lazy_static;
use serde::{Serialize, Deserialize};

use crate::Result;


pub static CONFIG_PATH: &str = "./app/config.json";


lazy_static! {
    static ref CONFIG_FILE: Mutex<Config> = {
        if let Ok(data) = std::fs::read(CONFIG_PATH) {
            #[allow(clippy::expect_used)]
            Mutex::new(serde_json::from_slice(&data).expect("Loading Config File"))
        } else {
            Mutex::default()
        }
    };
}


pub fn update_config<F: FnOnce(&mut Config) -> Result<()>>(value: F) -> Result<()> {
    let mut config = get_config();

    value(&mut config)?;

    *CONFIG_FILE.lock().unwrap() = config;

    Ok(())
}


pub fn get_config() -> Config {
    CONFIG_FILE.lock().unwrap().clone()
}


pub async fn save_config() -> Result<()> {
    tokio::fs::write(
        CONFIG_PATH,
        serde_json::to_string_pretty(&get_config())?,
    ).await?;

    Ok(())
}



#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ConfigServer,
    pub auth: AuthConfig,
    pub email: Option<ConfigEmail>,
    #[serde(default)]
    pub storage: ConfigStores,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ConfigServer::default(),
            auth: AuthConfig::default(),
            email: Some(ConfigEmail::default()),
            storage: ConfigStores::default(),
        }
    }
}

impl From<Config> for SharedConfig {
    fn from(mut val: Config) -> Self {
        val.auth.auth_key.clear();

        SharedConfig {
            server: val.server,
            auth: val.auth,
            email: val.email,
        }
    }
}



// Services

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ConfigStores {
    pub b2: ConfigStoreB2,
    pub filesystem: ConfigStoreFileSystem,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ConfigStoreFileSystem {
    pub enabled: bool,
    pub directory: String,
}

impl Default for ConfigStoreFileSystem {
    fn default() -> Self {
        Self {
            enabled: false,
            directory: String::from("./app/thumbnails")
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ConfigStoreB2 {
    pub enabled: bool,

    pub id: String,
    pub key: String,

    pub bucket_id: String,

    pub base_url: String,

    pub directory: String,
}