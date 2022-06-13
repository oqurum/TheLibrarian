use librarian_common::{ThumbnailStore, ImageId, BookId, util::serialize_datetime, PersonId, ImageType};
use chrono::{DateTime, TimeZone, Utc};
use rusqlite::{Row, params, OptionalExtension};
use serde::Serialize;

use crate::{Result, Database};


#[derive(Serialize)]
pub struct NewImageModel {
	pub link_id: usize,
	pub type_of: ImageType,

	pub path: ThumbnailStore,

	#[serde(serialize_with = "serialize_datetime")]
	pub created_at: DateTime<Utc>,
}


#[derive(Debug, Serialize)]
pub struct ImageModel {
	pub id: ImageId,

	pub link_id: usize,
	pub type_of: ImageType,

	pub path: ThumbnailStore,

	#[serde(serialize_with = "serialize_datetime")]
	pub created_at: DateTime<Utc>,
}

impl<'a> TryFrom<&Row<'a>> for ImageModel {
	type Error = rusqlite::Error;

	fn try_from(value: &Row<'a>) -> std::result::Result<Self, Self::Error> {
		Ok(Self {
			id: value.get(0)?,
			link_id: value.get(1)?,
			type_of: ImageType::from_number(value.get(2)?),
			path: ThumbnailStore::from(value.get::<_, String>(3)?),
			created_at: Utc.timestamp_millis(value.get(4)?),
		})
	}
}


impl NewImageModel {
	pub fn new_book(link_id: BookId, path: ThumbnailStore) -> Self {
		Self { link_id: *link_id, path, type_of: ImageType::Book, created_at: Utc::now() }
	}

	pub fn new_person(link_id: PersonId, path: ThumbnailStore) -> Self {
		Self { link_id: *link_id, path, type_of: ImageType::Person, created_at: Utc::now() }
	}

	pub async fn insert(self, db: &Database) -> Result<ImageModel> {
		let conn = db.write().await;

		conn.execute(r#"
			INSERT OR IGNORE INTO uploaded_images (link_id, path, created_at)
			VALUES (?1, ?2, ?3)
		"#,
		params![
			self.link_id,
			self.path.to_string(),
			self.created_at.timestamp_millis()
		])?;

		Ok(ImageModel {
			id: ImageId::from(conn.last_insert_rowid() as usize),
			link_id: self.link_id,
			type_of: self.type_of,
			path: self.path,
			created_at: self.created_at,
		})
	}
}


impl ImageModel {
	pub async fn get_by_linked_id(id: BookId, db: &Database) -> Result<Vec<Self>> {
		let this = db.read().await;

		let mut conn = this.prepare(r#"SELECT * FROM uploaded_images WHERE link_id = ?1"#)?;

		let map = conn.query_map([id], |v| Self::try_from(v))?;

		Ok(map.collect::<std::result::Result<Vec<_>, _>>()?)
	}

	pub async fn get_by_id(id: ImageId, db: &Database) -> Result<Option<Self>> {
		Ok(db.read().await.query_row(
			r#"SELECT * FROM uploaded_images WHERE id = ?1 LIMIT 1"#,
			[id],
			|v| Self::try_from(v)
		).optional()?)
	}

	pub async fn remove(link_id: BookId, path: ThumbnailStore, db: &Database) -> Result<()> {
		db.write().await
		.execute(r#"DELETE FROM uploaded_images WHERE link_id = ?1 AND path = ?2"#,
			params![
				link_id,
				path.to_string(),
			]
		)?;

		Ok(())
	}
}