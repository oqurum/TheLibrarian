use common::ThumbnailStore;
use sha2::{Sha256, Digest};

use crate::{Result, Database, model::{NewUploadedImageModel, UploadedImageModel}, storage::get_storage};


pub async fn store_image(image: Vec<u8>, db: &Database) -> Result<UploadedImageModel> {
    // TODO: Resize? Function is currently only used for thumbnails.
    let image = image::load_from_memory(&image)?;

    let mut writer = std::io::Cursor::new(Vec::new());
    image.write_to(&mut writer, image::ImageFormat::Jpeg)?;

    let image = writer.into_inner();

    let hash: String = Sha256::digest(&image)
        .iter()
        .map(|v| format!("{:02x}", v))
        .collect();

    get_storage().await.upload(&hash, image).await?;

    NewUploadedImageModel::new(ThumbnailStore::from(hash))
        .get_or_insert(db).await
}