use librarian_common::{ThumbnailStore, ImageId, BookId, util::serialize_datetime, PersonId, ImageType};
use chrono::{DateTime, TimeZone, Utc};
use rusqlite::{Row, params, OptionalExtension};
use serde::Serialize;

use crate::{Result, Database};


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





impl<'a> TryFrom<&Row<'a>> for UploadedImageModel {
	type Error = rusqlite::Error;

	fn try_from(value: &Row<'a>) -> std::result::Result<Self, Self::Error> {
		Ok(Self {
			id: value.get(0)?,
			path: ThumbnailStore::from(value.get::<_, String>(3)?),
			created_at: Utc.timestamp_millis(value.get(4)?),
		})
	}
}

impl<'a> TryFrom<&Row<'a>> for ImageLinkModel {
	type Error = rusqlite::Error;

	fn try_from(value: &Row<'a>) -> std::result::Result<Self, Self::Error> {
		Ok(Self {
			image_id: value.get(0)?,
			link_id: value.get(1)?,
			type_of: ImageType::from_number(value.get(2)?).unwrap(),
		})
	}
}


impl NewUploadedImageModel {
	pub fn new(path: ThumbnailStore) -> Self {
		Self { path, created_at: Utc::now() }
	}

	pub async fn insert(self, db: &Database) -> Result<UploadedImageModel> {
		let conn = db.write().await;

		conn.execute(r#"
			INSERT OR IGNORE INTO uploaded_images (path, created_at)
			VALUES (?1, ?2)
		"#,
		params![
			self.path.to_string(),
			self.created_at.timestamp_millis()
		])?;

		Ok(UploadedImageModel {
			id: ImageId::from(conn.last_insert_rowid() as usize),
			path: self.path,
			created_at: self.created_at,
		})
	}
}


impl UploadedImageModel {
	pub async fn get_by_id(id: ImageId, db: &Database) -> Result<Option<Self>> {
		Ok(db.read().await.query_row(
			r#"SELECT * FROM uploaded_images WHERE id = ?1 LIMIT 1"#,
			[id],
			|v| Self::try_from(v)
		).optional()?)
	}

	pub async fn remove(link_id: BookId, path: ThumbnailStore, db: &Database) -> Result<()> {
		// TODO: Check for currently set images
		// TODO: Remove image links.
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


	pub async fn insert(&self, db: &Database) -> Result<()> {
		let conn = db.write().await;

		conn.execute(r#"
			INSERT OR IGNORE INTO image_link (image_id, link_id, type_of)
			VALUES (?1, ?2, ?3)
		"#,
		params![
			self.image_id.to_string(),
			self.link_id.to_string(),
			self.type_of.as_num()
		])?;

		Ok(())
	}

	pub async fn remove(self, db: &Database) -> Result<()> {
		db.write().await
		.execute(r#"DELETE FROM image_link WHERE image_id = ?1 AND link_id = ?2 AND type_of = ?3"#,
			params![
				self.image_id,
				self.link_id,
				self.type_of.as_num(),
			]
		)?;

		Ok(())
	}

	// TODO: Place into ImageWithLink struct?
	pub async fn get_by_linked_id(id: usize, type_of: ImageType, db: &Database) -> Result<Vec<ImageWithLink>> {
		let this = db.read().await;

		let mut conn = this.prepare(r#"
			SELECT image_link.*, uploaded_images.path, uploaded_images.created_at
			FROM image_link
			INNER JOIN uploaded_images
				ON uploaded_images.id = image_link.image_id
			WHERE link_id = ?1 AND type_of = ?2
		"#)?;

		let map = conn.query_map(params![id, type_of.as_num()], |row| {
			Ok(ImageWithLink {
				image_id: row.get(0)?,
				link_id: row.get(1)?,
				type_of: ImageType::from_number(row.get(2)?).unwrap(),
				path: ThumbnailStore::from(row.get::<_, String>(3)?),
				created_at: Utc.timestamp_millis(row.get(4)?),
			})
		})?;

		Ok(map.collect::<std::result::Result<Vec<_>, _>>()?)
	}
}