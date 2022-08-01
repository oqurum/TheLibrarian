use common::{BookId, TagId, BookTagId};
use chrono::{DateTime, TimeZone, Utc};
use common_local::BookTag;
use rusqlite::{OptionalExtension, params};


use crate::{Database, Result};

use super::{TagModel, AdvRow, TableRow};

pub struct BookTagModel {
	pub id: BookTagId,

	pub book_id: BookId,
	pub tag_id: TagId,

	pub index: usize,

	pub created_at: DateTime<Utc>,
}

impl TableRow<'_> for BookTagModel {
	fn create(row: &mut AdvRow<'_>) -> rusqlite::Result<Self> {
		Ok(Self {
			id: row.next()?,

			book_id: row.next()?,
			tag_id: row.next()?,

			index: row.next()?,

			created_at: Utc.timestamp_millis(row.next()?),
		})
	}
}



pub struct BookTagWithTagModel {
	pub id: BookTagId,

	pub book_id: BookId,

	pub index: usize,

	pub created_at: DateTime<Utc>,

	pub tag: TagModel,
}

impl TableRow<'_> for BookTagWithTagModel {
	fn create(row: &mut AdvRow<'_>) -> rusqlite::Result<Self> {
		Ok(Self {
			id: row.next()?,

			book_id: row.next()?,
			index: row.next()?,

			created_at: Utc.timestamp_millis(row.next()?),

			tag: TagModel::create(row)?
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
	pub async fn get_by_id(id: BookTagId, db: &Database) -> Result<Option<Self>> {
		Ok(db.read().await.query_row(
			r#"SELECT * FROM book_tags WHERE id = ?1"#,
			params![id],
			|v| Self::from_row(v)
		).optional()?)
	}

	pub async fn remove(book_id: BookId, tag_id: TagId, db: &Database) -> Result<usize> {
		Ok(db.write().await.execute(
			r#"DELETE FROM book_tags WHERE book_id = ?1 AND tag_id = ?2"#,
			params![book_id, tag_id],
		)?)
	}

	pub async fn insert(book_id: BookId, tag_id: TagId, index: Option<usize>, db: &Database) -> Result<Self> {
		let index = if let Some(index) = index {
			db.write().await.execute(
				r#"UPDATE book_tags
				SET windex = windex + 1
				WHERE book_id = ?1 AND tag_id = ?2 AND windex >= ?3"#,
				params![book_id, tag_id, index],
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
			id: BookTagId::from(conn.last_insert_rowid() as usize),
			book_id,
			tag_id,
			index,
			created_at,
		})
	}

	pub async fn count_book_tags_by_bid_tid(book_id: BookId, tag_id: TagId, db: &Database) -> Result<usize> {
		Ok(db.read().await.query_row(
			r#"SELECT COUNT(*) FROM book_tags WHERE book_id = ?1 AND tag_id = ?2"#,
			params![book_id, tag_id],
			|v| v.get(0)
		)?)
	}

	pub async fn get_books_by_book_id(book_id: BookId, db: &Database) -> Result<Vec<Self>> {
		let this = db.read().await;

		let mut conn = this.prepare("SELECT * FROM book_tags WHERE book_id = ?1")?;

		let map = conn.query_map([book_id], |v| Self::from_row(v))?;

		Ok(map.collect::<std::result::Result<Vec<_>, _>>()?)
	}
}


impl BookTagWithTagModel {
	pub async fn get_by_book_id(book_id: BookId, db: &Database) -> Result<Vec<Self>> {
		let this = db.read().await;

		let mut conn = this.prepare(
			r#"SELECT book_tags.id, book_tags.book_id, windex, book_tags.created_at, tags.*
			FROM book_tags
			JOIN tags ON book_tags.tag_id == tags.id
			WHERE book_id = ?1"#
		)?;

		let map = conn.query_map([book_id], |v| Self::from_row(v))?;

		Ok(map.collect::<std::result::Result<Vec<_>, _>>()?)
	}

	pub async fn get_by_book_id_and_tag_id(book_id: BookId, tag_id: TagId, db: &Database) -> Result<Option<Self>> {
		Ok(db.read().await.query_row(
			r#"SELECT book_tags.id, book_tags.book_id, windex, book_tags.created_at, tags.*
			FROM book_tags
			JOIN tags ON book_tags.tag_id == tags.id
			WHERE book_id = ?1 AND tag_id = ?2"#,
			params![book_id, tag_id],
			|v| Self::from_row(v)
		).optional()?)
	}
}