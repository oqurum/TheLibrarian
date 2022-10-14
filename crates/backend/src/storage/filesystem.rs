use std::path::PathBuf;
use tokio::fs;

use crate::{config::ConfigStoreFileSystem, Result};

pub struct FsService {
    pub directory: PathBuf,
}

impl FsService {
    pub fn new(config: &ConfigStoreFileSystem) -> Self {
        Self {
            directory: PathBuf::from(&config.directory),
        }
    }

    pub async fn init(&self) -> Result<()> {
        // Directory check
        if fs::metadata(&self.directory).await.is_err() {
            fs::create_dir_all(&self.directory).await?;
        }

        Ok(())
    }

    pub async fn upload(&self, full_file_path: PathBuf, contents: Vec<u8>) -> Result<()> {
        fs::create_dir_all(&full_file_path).await?;

        fs::write(&full_file_path, contents).await?;

        Ok(())
    }

    pub async fn delete(&self, full_file_path: PathBuf) -> Result<()> {
        fs::remove_file(full_file_path).await?;

        Ok(())
    }
}
