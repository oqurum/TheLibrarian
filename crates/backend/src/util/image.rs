use common::ThumbnailStore;
use sha2::{Digest, Sha256};

use crate::{
    model::{NewUploadedImageModel, UploadedImageModel},
    storage::get_storage,
    Result,
};

pub async fn store_image(
    image: Vec<u8>,
    db: &tokio_postgres::Client,
) -> Result<UploadedImageModel> {
    // TODO: Resize? Function is currently only used for converting to jpeg
    let image = image::load_from_memory(&image)?;

    let mut writer = std::io::Cursor::new(Vec::new());
    image.write_to(&mut writer, image::ImageFormat::Jpeg)?;

    let image_data = writer.into_inner();

    let hash: String = Sha256::digest(&image_data)
        .iter()
        .map(|v| format!("{:02x}", v))
        .collect();

    if let Some(v) = UploadedImageModel::get_by_path(&hash, db).await? {
        Ok(v)
    } else {
        get_storage().await.upload(&hash, image_data).await?;

        NewUploadedImageModel::new(ThumbnailStore::from(hash), image.width(), image.height())
            .get_or_insert(db)
            .await
    }
}
