use librarian_common::{TagType, BookTag};
use chrono::{DateTime, TimeZone, Utc};
use rusqlite::{Row, OptionalExtension, params};

use crate::{Database, Result};

use super::TagModel;


pub struct BookTagModel {
	pub id: usize,

	pub book_id: usize,
	pub tag_id: usize,

	pub index: usize,

	pub created_at: DateTime<Utc>,
}

impl<'a> TryFrom<&Row<'a>> for BookTagModel {
	type Error = rusqlite::Error;

	fn try_from(value: &Row<'a>) -> std::result::Result<Self, Self::Error> {
		Ok(Self {
			id: value.get(0)?,

			book_id: value.get(1)?,
			tag_id: value.get(2)?,

			index: value.get(3)?,

			created_at: Utc.timestamp_millis(value.get(4)?),
		})
	}
}




pub struct BookTagWithTagModel {
	pub id: usize,

	pub book_id: usize,

	pub index: usize,

	pub created_at: DateTime<Utc>,

	pub tag: TagModel,
}

impl<'a> TryFrom<&Row<'a>> for BookTagWithTagModel {
	type Error = rusqlite::Error;

	fn try_from(value: &Row<'a>) -> std::result::Result<Self, Self::Error> {
		Ok(Self {
			id: value.get(0)?,

			book_id: value.get(1)?,

			index: value.get(2)?,

			created_at: Utc.timestamp_millis(value.get(3)?),

			tag: TagModel {
				id: value.get(4)?,
				name: value.get(5)?,
				type_of: TagType::from_u8(value.get(6)?, value.get(7)?),
				created_at: Utc.timestamp_millis(value.get(8)?),
				updated_at: Utc.timestamp_millis(value.get(9)?),
			}
		})
	}
}

impl From<BookTagWithTagModel> for BookTag {
	fn from(val: BookTagWithTagModel) -> Self {
		BookTag {
			id: val.id,
			book_id: val.book_id,
			index: val.index,
			created_at: val.created_at,
			tag: val.tag.into(),
		}
	}
}



impl BookTagModel {
	pub async fn get_by_id(id: usize, db: &Database) -> Result<Option<Self>> {
		Ok(db.read().await.query_row(
			r#"SELECT * FROM book_tags WHERE id = ?1"#,
			params![id],
			|v| Self::try_from(v)
		).optional()?)
	}

	pub async fn remove(book_id: usize, tag_id: usize, db: &Database) -> Result<usize> {
		Ok(db.write().await.execute(
			r#"DELETE FROM book_tags WHERE book_id = ?1 AND tag_id = ?2"#,
			[book_id, tag_id],
		)?)
	}

	pub async fn insert(book_id: usize, tag_id: usize, index: Option<usize>, db: &Database) -> Result<Self> {
		let index = if let Some(index) = index {
			db.write().await.execute(
				r#"UPDATE book_tags
				SET windex = windex + 1
				WHERE book_id = ?1 AND tag_id = ?2 AND windex >= ?3"#,
				[book_id, tag_id, index],
			)?;

			index
		} else {
			Self::count_book_tags_by_bid_tid(book_id, tag_id, db).await?
		};

		let conn = db.write().await;

		let created_at = Utc::now();

		conn.execute(r#"
			INSERT INTO book_tags (book_id, tag_id, windex, created_at)
			VALUES (?1, ?2, ?3, ?4)
		"#,
		params![
			book_id,
			tag_id,
			index,
			created_at.timestamp_millis(),
		])?;

		Ok(Self {
			id: conn.last_insert_rowid() as usize,
			book_id,
			tag_id,
			index,
			created_at,
		})
	}

	pub async fn count_book_tags_by_bid_tid(book_id: usize, tag_id: usize, db: &Database) -> Result<usize> {
		Ok(db.read().await.query_row(
			r#"SELECT COUNT(*) FROM book_tags WHERE book_id = ?1 AND tag_id = ?2"#,
			[book_id, tag_id],
			|v| v.get(0)
		)?)
	}

	pub async fn get_books_by_book_id(book_id: usize, db: &Database) -> Result<Vec<Self>> {
		let this = db.read().await;

		let mut conn = this.prepare("SELECT * FROM book_tags WHERE book_id = ?1")?;

		let map = conn.query_map([book_id], |v| Self::try_from(v))?;

		Ok(map.collect::<std::result::Result<Vec<_>, _>>()?)
	}
}


impl BookTagWithTagModel {
	pub async fn get_by_book_id(book_id: usize, db: &Database) -> Result<Vec<Self>> {
		let this = db.read().await;

		let mut conn = this.prepare(
			r#"SELECT book_tags.id, book_tags.book_id, windex, book_tags.created_at, tags.*
			FROM book_tags
			JOIN tags ON book_tags.tag_id == tags.id
			WHERE book_id = ?1"#
		)?;

		let map = conn.query_map([book_id], |v| Self::try_from(v))?;

		Ok(map.collect::<std::result::Result<Vec<_>, _>>()?)
	}

	pub async fn get_by_book_id_and_tag_id(book_id: usize, tag_id: usize, db: &Database) -> Result<Option<Self>> {
		Ok(db.read().await.query_row(
			r#"SELECT book_tags.id, book_tags.book_id, windex, book_tags.created_at, tags.*
			FROM book_tags
			JOIN tags ON book_tags.tag_id == tags.id
			WHERE book_id = ?1 AND tag_id = ?2"#,
			params![book_id, tag_id],
			|v| Self::try_from(v)
		).optional()?)
	}
}