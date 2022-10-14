use chrono::{DateTime, Utc};
use common::{BookId, ImageId, ImageType, PersonId, ThumbnailStore};
use common_local::util::serialize_datetime;
use serde::Serialize;

use crate::Result;

use super::{row_bigint_to_usize, row_int_to_usize, AdvRow, TableRow};

#[derive(Debug, Serialize)]
pub struct ImageLinkModel {
    pub image_id: ImageId,

    pub link_id: usize,
    pub type_of: ImageType,
}

#[derive(Serialize)]
pub struct NewUploadedImageModel {
    pub path: ThumbnailStore,

    pub width: u32,
    pub height: u32,
    pub ratio: f32,

    #[serde(serialize_with = "serialize_datetime")]
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct UploadedImageModel {
    pub id: ImageId,

    pub path: ThumbnailStore,

    pub width: u32,
    pub height: u32,
    pub ratio: f32,

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
            image_id: ImageId::from(row.next::<i32>()? as usize),
            link_id: row.next::<i32>()? as usize,
            type_of: ImageType::from_number(row.next::<i32>()? as u8).unwrap(),
            path: ThumbnailStore::from(row.next::<String>()?),
            created_at: row.next()?,
        })
    }
}

impl TableRow for UploadedImageModel {
    fn create(row: &mut AdvRow) -> Result<Self> {
        Ok(Self {
            id: ImageId::from(row.next::<i32>()? as usize),
            path: ThumbnailStore::from(row.next::<String>()?),

            width: row.next::<i32>()? as u32,
            height: row.next::<i32>()? as u32,
            ratio: row.next()?,

            created_at: row.next()?,
        })
    }
}

impl TableRow for ImageLinkModel {
    fn create(row: &mut AdvRow) -> Result<Self> {
        Ok(Self {
            image_id: ImageId::from(row.next::<i32>()? as usize),
            link_id: row.next::<i32>()? as usize,
            type_of: ImageType::from_number(row.next::<i32>()? as u8).unwrap(),
        })
    }
}

impl NewUploadedImageModel {
    pub fn new(path: ThumbnailStore, width: u32, height: u32) -> Self {
        Self {
            path,
            width,
            height,
            ratio: height as f32 / width as f32,
            created_at: Utc::now(),
        }
    }

    pub async fn get_or_insert(self, db: &tokio_postgres::Client) -> Result<UploadedImageModel> {
        if let Some(value) =
            UploadedImageModel::get_by_path(self.path.as_value().unwrap(), db).await?
        {
            Ok(value)
        } else {
            self.insert(db).await
        }
    }

    pub async fn insert(self, db: &tokio_postgres::Client) -> Result<UploadedImageModel> {
        let row = db.query_one(
            "INSERT INTO uploaded_image (path, width, height, ratio, created_at) VALUES ($1, $2, $3, $4, $5) RETURNING id",
            params![
                self.path.as_value(),
                self.width as i32,
                self.height as i32,
                self.ratio,
                self.created_at,
            ]
        ).await?;

        Ok(UploadedImageModel {
            id: ImageId::from(row_int_to_usize(row)?),
            path: self.path,
            width: self.width,
            height: self.height,
            ratio: self.ratio,
            created_at: self.created_at,
        })
    }

    pub async fn path_exists(path: &str, db: &tokio_postgres::Client) -> Result<bool> {
        Ok(row_bigint_to_usize(
            db.query_one(
                "SELECT COUNT(*) FROM uploaded_image WHERE path = $1",
                params![path],
            )
            .await?,
        )? != 0)
    }
}

impl UploadedImageModel {
    pub async fn get_all(db: &tokio_postgres::Client) -> Result<Vec<Self>> {
        db.query(r#"SELECT * FROM uploaded_image"#, &[])
            .await?
            .into_iter()
            .map(Self::from_row)
            .collect()
    }

    pub async fn get_by_path(value: &str, db: &tokio_postgres::Client) -> Result<Option<Self>> {
        db.query_opt(
            r#"SELECT * FROM uploaded_image WHERE path = $1"#,
            params![value],
        )
        .await?
        .map(Self::from_row)
        .transpose()
    }

    pub async fn get_by_id(value: ImageId, db: &tokio_postgres::Client) -> Result<Option<Self>> {
        db.query_opt(
            r#"SELECT * FROM uploaded_image WHERE id = $1"#,
            params![*value as i32],
        )
        .await?
        .map(Self::from_row)
        .transpose()
    }

    pub async fn remove_by_path(path: ThumbnailStore, db: &tokio_postgres::Client) -> Result<()> {
        db.execute(
            "DELETE FROM uploaded_image WHERE path = $1",
            params![path.as_value()],
        )
        .await?;

        Ok(())
    }

    pub async fn remove_by_id(id: ImageId, db: &tokio_postgres::Client) -> Result<()> {
        db.execute(
            "DELETE FROM uploaded_image WHERE id = $1",
            params![*id as i32],
        )
        .await?;

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
        db.execute("INSERT INTO image_link (image_id, link_id, type_of) VALUES ($1, $2, $3) ON CONFLICT (image_id, link_id, type_of) DO NOTHING",
        params![
            *self.image_id as i32,
            self.link_id as i32,
            self.type_of.as_num() as i32
        ]).await?;

        Ok(())
    }

    pub async fn remove(self, db: &tokio_postgres::Client) -> Result<()> {
        db.execute(
            "DELETE FROM image_link WHERE image_id = $1 AND link_id = $2 AND type_of = $3",
            params![
                *self.image_id as i32,
                self.link_id as i32,
                self.type_of.as_num() as i32,
            ],
        )
        .await?;

        Ok(())
    }

    pub async fn count_by_image_id(id: ImageId, db: &tokio_postgres::Client) -> Result<usize> {
        row_bigint_to_usize(
            db.query_one(
                "SELECT COUNT(*) FROM image_link WHERE image_id = $1",
                params![*id as i32],
            )
            .await?,
        )
    }

    pub async fn find_by_link_id(
        id: usize,
        type_of: ImageType,
        db: &tokio_postgres::Client,
    ) -> Result<Vec<Self>> {
        let values = db
            .query(
                "SELECT * FROM image_link WHERE link_id = $1 AND type_of = $2",
                params![id as i32, type_of.as_num() as i32],
            )
            .await?;

        values.into_iter().map(Self::from_row).collect()
    }

    // TODO: Place into ImageWithLink struct?
    pub async fn find_by_linked_id_w_image(
        id: usize,
        type_of: ImageType,
        db: &tokio_postgres::Client,
    ) -> Result<Vec<ImageWithLink>> {
        let values = db
            .query(
                r#"SELECT image_link.*, uploaded_image.path, uploaded_image.created_at
                FROM image_link
                INNER JOIN uploaded_image
                    ON uploaded_image.id = image_link.image_id
                WHERE link_id = $1 AND type_of = $2
            "#,
                params![id as i32, type_of.as_num() as i32],
            )
            .await?;

        values.into_iter().map(ImageWithLink::from_row).collect()
    }
}
