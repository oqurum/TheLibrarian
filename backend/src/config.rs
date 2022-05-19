use std::sync::Mutex;

use lazy_static::lazy_static;
use rand::distributions::{Alphanumeric, DistString};
use serde::{Serialize, Deserialize};

use crate::Result;


pub static CONFIG_PATH: &str = "../app/config.json";


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


pub fn update_config<F: FnMut(&mut Config) -> Result<()>>(mut value: F) -> Result<()> {
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
	pub server_name: String,
	pub auth: AuthConfig,
}

impl Default for Config {
	fn default() -> Self {
		Self {
			server_name: String::from("Unnamed Server"),
			auth: AuthConfig::default()
		}
	}
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
	pub new_users: bool,
	pub auth_key: String,
}

impl Default for AuthConfig {
	fn default() -> Self {
		Self {
			new_users: false,
			auth_key: Alphanumeric.sample_string(&mut rand::thread_rng(), 48),
		}
	}
}