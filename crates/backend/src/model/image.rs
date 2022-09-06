use common_local::util::serialize_datetime;
use common::{ThumbnailStore, ImageId, BookId, PersonId, ImageType};
use chrono::{DateTime, TimeZone, Utc};
use serde::Serialize;

use crate::{Result};

use super::{TableRow, AdvRow, row_to_usize};


#[derive(Debug, Serialize)]
pub struct ImageLinkModel {
    pub image_id: ImageId,

    pub link_id: usize,
    pub type_of: ImageType,
}


#[derive(Serialize)]
pub struct NewUploadedImageModel {
    pub path: ThumbnailStore,

    #[serde(serialize_with = "serialize_datetime")]
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct UploadedImageModel {
    pub id: ImageId,

    pub path: ThumbnailStore,

    #[serde(serialize_with = "serialize_datetime")]
    pub created_at: DateTime<Utc>,
}


#[derive(Debug, Serialize)]
pub struct ImageWithLink {
    pub image_id: ImageId,

    pub link_id: usize,
    pub type_of: ImageType,

    pub path: ThumbnailStore,

    #[serde(serialize_with = "serialize_datetime")]
    pub created_at: DateTime<Utc>,
}


impl TableRow for ImageWithLink {
    fn create(row: &mut AdvRow) -> Result<Self> {
        Ok(Self {
            image_id: ImageId::from(row.next::<i64>()? as usize),
            link_id: row.next::<i64>()? as usize,
            type_of: ImageType::from_number(row.next::<i16>()? as u8).unwrap(),
            path: ThumbnailStore::from(row.next::<String>()?),
            created_at: Utc.timestamp_millis(row.next()?),
        })
    }
}



impl TableRow for UploadedImageModel {
    fn create(row: &mut AdvRow) -> Result<Self> {
        Ok(Self {
            id: ImageId::from(row.next::<i64>()? as usize),
            path: ThumbnailStore::from(row.next::<String>()?),
            created_at: Utc.timestamp_millis(row.next()?),
        })
    }
}

impl TableRow for ImageLinkModel {
    fn create(row: &mut AdvRow) -> Result<Self> {
        Ok(Self {
            image_id: ImageId::from(row.next::<i64>()? as usize),
            link_id: row.next::<i64>()? as usize,
            type_of: ImageType::from_number(row.next::<i16>()? as u8).unwrap(),
        })
    }
}



impl NewUploadedImageModel {
    pub fn new(path: ThumbnailStore) -> Self {
        Self { path, created_at: Utc::now() }
    }

    pub async fn get_or_insert(self, db: &tokio_postgres::Client) -> Result<UploadedImageModel> {
        if let Some(value) = UploadedImageModel::get_by_path(self.path.as_value().unwrap(), db).await? {
            Ok(value)
        } else {
            self.insert(db).await
        }
    }

    pub async fn insert(self, db: &tokio_postgres::Client) -> Result<UploadedImageModel> {
        let row = db.query_one(
            "INSERT OR IGNORE INTO uploaded_images (path, created_at) VALUES (?1, ?2)",
            params![
                self.path.as_value(),
                self.created_at.timestamp_millis()
            ]
        ).await?;

        Ok(UploadedImageModel {
            id: ImageId::from(row_to_usize(row)?),
            path: self.path,
            created_at: self.created_at,
        })
    }

    pub async fn path_exists(path: &str, db: &tokio_postgres::Client) -> Result<bool> {
        Ok(row_to_usize(db.query_one(
            "SELECT COUNT(*) FROM uploaded_images WHERE path = ?1",
            params![ path ],
        ).await?)? != 0)
    }
}


impl UploadedImageModel {
    pub async fn get_by_path(value: &str, db: &tokio_postgres::Client) -> Result<Option<Self>> {
        db.query_opt(
            r#"SELECT * FROM uploaded_images WHERE path = ?1"#,
            params![ value ],
        ).await?.map(Self::from_row).transpose()
    }

    pub async fn get_by_id(value: ImageId, db: &tokio_postgres::Client) -> Result<Option<Self>> {
        db.query_opt(
            r#"SELECT * FROM uploaded_images WHERE id = ?1"#,
            params![ *value as i64 ],
        ).await?.map(Self::from_row).transpose()
    }

    pub async fn remove(link_id: BookId, path: ThumbnailStore, db: &tokio_postgres::Client) -> Result<()> {
        // TODO: Check for currently set images
        // TODO: Remove image links.
        db.execute("DELETE FROM uploaded_images WHERE link_id = ?1 AND path = ?2",
            params![ *link_id as i64, path.as_value() ]
        ).await?;

        Ok(())
    }
}


impl ImageLinkModel {
    pub fn new_book(image_id: ImageId, link_id: BookId) -> Self {
        Self {
            image_id,
            link_id: *link_id,
            type_of: ImageType::Book,
        }
    }

    pub fn new_person(image_id: ImageId, link_id: PersonId) -> Self {
        Self {
            image_id,
            link_id: *link_id,
            type_of: ImageType::Person,
        }
    }


    pub async fn insert(&self, db: &tokio_postgres::Client) -> Result<()> {
        db.execute("INSERT OR IGNORE INTO image_link (image_id, link_id, type_of) VALUES (?1, ?2, ?3)",
        params![
            *self.image_id as i64,
            self.link_id as i64,
            self.type_of.as_num() as i16
        ]).await?;

        Ok(())
    }

    pub async fn remove(self, db: &tokio_postgres::Client) -> Result<()> {
        db.execute("DELETE FROM image_link WHERE image_id = ?1 AND link_id = ?2 AND type_of = ?3",
            params![
                *self.image_id as i64,
                self.link_id as i64,
                self.type_of.as_num() as i16,
            ]
        ).await?;

        Ok(())
    }

    // TODO: Place into ImageWithLink struct?
    pub async fn get_by_linked_id(id: usize, type_of: ImageType, db: &tokio_postgres::Client) -> Result<Vec<ImageWithLink>> {
        let values = db.query(
            r#"SELECT image_link.*, uploaded_images.path, uploaded_images.created_at
                FROM image_link
                INNER JOIN uploaded_images
                    ON uploaded_images.id = image_link.image_id
                WHERE link_id = ?1 AND type_of = ?2
            "#,
            params![ id as i64, type_of.as_num() as i16 ]
        ).await?;

        values.into_iter().map(ImageWithLink::from_row).collect()
    }
}