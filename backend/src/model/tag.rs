use common_local::{TagType, TagFE};
use chrono::{DateTime, TimeZone, Utc};
use common::TagId;
use rusqlite::{params, OptionalExtension};

use crate::{Database, Result};

use super::{AdvRow, TableRow};

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


impl TableRow<'_> for TagModel {
	fn create(row: &mut AdvRow<'_>) -> rusqlite::Result<Self> {
		Ok(Self {
			id: row.next()?,
			name: row.next()?,
			type_of: TagType::from_u8(row.next()?, row.next()?),
			created_at: Utc.timestamp_millis(row.next()?),
			updated_at: Utc.timestamp_millis(row.next()?),
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
			updated_at: val.updated_at
		}
	}
}


impl TagModel {
	pub async fn get_by_id(id: TagId, db: &Database) -> Result<Option<Self>> {
		Ok(db.read().await.query_row(
			r#"SELECT * FROM tags WHERE id = ?1"#,
			params![id],
			|v| Self::from_row(v)
		).optional()?)
	}

	pub async fn get_all(db: &Database) -> Result<Vec<Self>> {
		let this = db.read().await;

		let mut conn = this.prepare("SELECT * FROM tags")?;

		let map = conn.query_map([], |v| Self::from_row(v))?;

		Ok(map.collect::<std::result::Result<Vec<_>, _>>()?)
	}
}


impl NewTagModel {
	pub async fn insert(self, db: &Database) -> Result<TagModel> {
		let conn = db.write().await;

		let now = Utc::now();

		let (type_of, data) = self.type_of.clone().split();

		conn.execute(r#"
			INSERT INTO tags (name, type_of, data, created_at, updated_at)
			VALUES (?1, ?2, ?3, ?4, ?5)
		"#,
		params![
			&self.name,
			type_of,
			data,
			now.timestamp_millis(),
			now.timestamp_millis()
		])?;

		Ok(TagModel {
			id: TagId::from(conn.last_insert_rowid() as usize),

			name: self.name,
			type_of: self.type_of,

			created_at: now,
			updated_at: now,
		})
	}
}