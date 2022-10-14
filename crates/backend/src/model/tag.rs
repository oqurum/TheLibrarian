use chrono::{DateTime, Utc};
use common::TagId;
use common_local::{TagFE, TagType};

use crate::Result;

use super::{row_int_to_usize, AdvRow, TableRow};

pub struct NewTagModel {
    pub name: String,
    pub type_of: TagType,
}

pub struct TagModel {
    pub id: TagId,

    pub name: String,
    pub type_of: TagType,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TableRow for TagModel {
    fn create(row: &mut AdvRow) -> Result<Self> {
        Ok(Self {
            id: TagId::from(row.next::<i32>()? as usize),
            name: row.next()?,
            type_of: TagType::from_u8(row.next::<i16>()? as u8, row.next()?),
            created_at: row.next()?,
            updated_at: row.next()?,
        })
    }
}

impl From<TagModel> for TagFE {
    fn from(val: TagModel) -> Self {
        TagFE {
            id: val.id,
            name: val.name,
            type_of: val.type_of,
            created_at: val.created_at,
            updated_at: val.updated_at,
        }
    }
}

impl TagModel {
    pub async fn get_by_id(id: TagId, db: &tokio_postgres::Client) -> Result<Option<Self>> {
        db.query_opt("SELECT * FROM tag WHERE id = $1", params![*id as i32])
            .await?
            .map(Self::from_row)
            .transpose()
    }

    pub async fn get_all(db: &tokio_postgres::Client) -> Result<Vec<Self>> {
        let conn = db.query("SELECT * FROM tag", &[]).await?;

        conn.into_iter().map(Self::from_row).collect()
    }
}

impl NewTagModel {
    pub async fn insert(self, db: &tokio_postgres::Client) -> Result<TagModel> {
        let now = Utc::now();

        let (type_of, data) = self.type_of.clone().split();

        let row = db
            .query_one(
                r#"
            INSERT INTO tag (name, type_of, data, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5) RETURNING id
        "#,
                params![&self.name, type_of as i16, data, now, now],
            )
            .await?;

        Ok(TagModel {
            id: TagId::from(row_int_to_usize(row)?),

            name: self.name,
            type_of: self.type_of,

            created_at: now,
            updated_at: now,
        })
    }
}
