use std::sync::Mutex;

use lazy_static::lazy_static;
use librarian_common::Config;

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