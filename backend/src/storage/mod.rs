use std::{path::PathBuf, sync::{Arc, RwLock, RwLockReadGuard}};

use actix_web::{HttpResponse, HttpRequest};
use lazy_static::lazy_static;

use crate::{Result, config::{ConfigStores, ConfigStoreFileSystem, ConfigStoreB2}};

pub mod b2;
pub mod filesystem;


lazy_static! {
    pub static ref STORE: Arc<RwLock<Storage>> = Arc::new(RwLock::new(Storage::None));
}

pub fn get_storage<'a>() -> RwLockReadGuard<'a, Storage> {
    STORE.read().unwrap()
}


pub enum Storage {
    None,

	B2(b2::Service),
	FileSystem(filesystem::FsService),
}

impl Storage {
    pub async fn pick_service_from_config(config: &ConfigStores) -> Result<Self> {
		let enabled_count = [
			config.filesystem.enabled,
			config.b2.enabled,
		]
		.iter()
		.filter(|v| **v)
		.count();

		if enabled_count == 0 {
			panic!("Please enable a service.");
		} else if enabled_count > 1 {
			panic!("Only ONE service can be enabled at once currently.");
		}

        if config.filesystem.enabled {
			println!("Service Filesystem Enabled");
			return Self::new_file_system(&config.filesystem);
		}

		if config.b2.enabled {
			println!("Service B2 Enabled");
			return Self::new_b2(&config.b2).await;
		}

		unreachable!()
	}

	pub async fn new_b2(config: &ConfigStoreB2) -> Result<Self> {
		Ok(Self::B2(b2::Service::new(config).await?))
	}

	pub fn new_file_system(config: &ConfigStoreFileSystem) -> Result<Self> {
		Ok(Self::FileSystem(filesystem::FsService::new(config)))
	}


	//

	pub fn get_full_file_path(&self, file_name: &str) -> PathBuf {
		match self {
            Self::B2(v) => {
				let mut path = v.directory.clone();
				path.push(format!("{file_name}.jpg"));

				path
			},

            Self::FileSystem(v) => {
				let mut path = v.directory.clone();

				path.push(get_directories(file_name));
				path.push(format!("{file_name}.jpg"));

				path
			}

            Self::None => panic!("Storage not Initiated"),
        }
	}

	pub async fn get_http_response(&self, file_name: &str, req: &HttpRequest) -> Result<HttpResponse> {
		let file_path = self.get_full_file_path(file_name);

		match self {
			Self::B2(v) => {
				Ok(HttpResponse::Ok().streaming(reqwest::get(v.get_http_url(&file_path.display().to_string())?).await?.bytes_stream()))
			}

            Self::FileSystem(_) => {
				Ok(actix_files::NamedFile::open_async(file_path).await?.into_response(req))
			}

            Self::None => panic!("Storage not Initiated"),
        }
	}


    pub async fn upload(
        &self,
        file_name: &str,
        contents: Vec<u8>,
    ) -> Result<()> {
		let file_path = self.get_full_file_path(file_name);

        match self {
            Self::B2(v) => v.upload(file_path, contents).await,
            Self::FileSystem(v) => v.upload(file_path, contents).await,
            Self::None => panic!("Storage not Initiated"),
        }
    }

    pub async fn delete(
        &self,
        file_name: &str,
    ) -> Result<()> {
		let file_path = self.get_full_file_path(file_name);

        match self {
            Self::B2(v) => v.hide_file(file_path).await,
            Self::FileSystem(v) => v.delete(file_path).await,
            Self::None => panic!("Storage not Initiated"),
        }
    }
}

fn get_directories(file_name: &str) -> String {
	format!(
		"{}/{}/{}/{}",
		file_name.get(0..1).unwrap(),
		file_name.get(1..2).unwrap(),
		file_name.get(2..3).unwrap(),
		file_name.get(3..4).unwrap()
	)
}