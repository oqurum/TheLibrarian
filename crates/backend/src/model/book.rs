use common_local::{MetadataItemCached, DisplayMetaItem, util::{serialize_datetime, serialize_datetime_opt}, search::PublicBook};
use chrono::{DateTime, TimeZone, Utc};
use common::{ThumbnailStore, BookId, PersonId};
use rusqlite::{params, OptionalExtension};
use serde::Serialize;

use crate::{Database, Result};

use super::{TableRow, AdvRow};


#[derive(Debug, Clone, Serialize)]
pub struct BookModel {
	pub id: BookId,

	pub title: Option<String>,
	pub clean_title: Option<String>,
	pub description: Option<String>,
	pub rating: f64,

	pub thumb_path: ThumbnailStore,

	// TODO: Make table for all tags. Include publisher in it. Remove country.
	pub cached: MetadataItemCached,

	pub isbn_10: Option<String>,
	pub isbn_13: Option<String>,

	pub is_public: bool,
	pub edition_count: usize,

	pub available_at: Option<String>,
	pub language: Option<u16>,

	#[serde(serialize_with = "serialize_datetime")]
	pub created_at: DateTime<Utc>,
	#[serde(serialize_with = "serialize_datetime")]
	pub updated_at: DateTime<Utc>,
	#[serde(serialize_with = "serialize_datetime_opt")]
	pub deleted_at: Option<DateTime<Utc>>,
}


impl TableRow<'_> for BookModel {
	fn create(row: &mut AdvRow<'_>) -> rusqlite::Result<Self> {
		Ok(Self {
			id: row.next()?,
			title: row.next()?,
			clean_title: row.next()?,
			description: row.next()?,
			rating: row.next()?,
			thumb_path: ThumbnailStore::from(row.next_opt::<String>()?),
			cached: row.next_opt::<String>()?
				.map(|v| MetadataItemCached::from_string(&v))
				.unwrap_or_default(),
			isbn_10: row.next()?,
			isbn_13: row.next()?,
			is_public: row.next()?,
			edition_count: row.next()?,
			available_at: row.next()?,
			language: row.next()?,
			created_at: Utc.timestamp_millis(row.next()?),
			updated_at: Utc.timestamp_millis(row.next()?),
			deleted_at: row.next_opt()?.map(|v| Utc.timestamp_millis(v)),
		})
	}
}

// TODO: Consolidate all of these into one or two structs.
impl From<BookModel> for DisplayMetaItem {
	fn from(val: BookModel) -> Self {
		DisplayMetaItem {
			id: val.id,
			title: val.title,
			clean_title: val.clean_title,
			description: val.description,
			rating: val.rating,
			thumb_path: val.thumb_path,
			cached: val.cached,
			isbn_10: val.isbn_10,
			isbn_13: val.isbn_13,
			is_public: val.is_public,
			edition_count: val.edition_count,
			available_at: val.available_at,
			language: val.language,
			created_at: val.created_at,
			updated_at: val.updated_at,
			deleted_at: val.deleted_at,
		}
	}
}

impl From<DisplayMetaItem> for BookModel {
	fn from(val: DisplayMetaItem) -> Self {
		BookModel {
			id: val.id,
			title: val.title,
			clean_title: val.clean_title,
			description: val.description,
			rating: val.rating,
			thumb_path: val.thumb_path,
			cached: val.cached,
			isbn_10: val.isbn_10,
			isbn_13: val.isbn_13,
			is_public: val.is_public,
			edition_count: val.edition_count,
			available_at: val.available_at,
			language: val.language,
			created_at: val.created_at,
			updated_at: val.updated_at,
			deleted_at: val.deleted_at,
		}
	}
}

#[allow(clippy::from_over_into)]
impl Into<PublicBook> for BookModel {
	fn into(self) -> PublicBook {
		PublicBook {
			id: *self.id,
			title: self.title,
			clean_title: self.clean_title,
			description: self.description,
			rating: self.rating,
			// We create the thumb_url in the actix request.
			thumb_url: String::new(),
			cached: self.cached,
			isbn_10: self.isbn_10,
			isbn_13: self.isbn_13,
			is_public: self.is_public,
			edition_count: self.edition_count,
			available_at: self.available_at,
			language: self.language,
			created_at: self.created_at,
			updated_at: self.updated_at,
			deleted_at: self.deleted_at,
		}
	}
}


impl BookModel {
	pub async fn get_book_count(db: &Database) -> Result<usize> {
		Ok(db.read().await.query_row(r#"SELECT COUNT(*) FROM book"#, [], |v| v.get(0))?)
	}

	pub async fn add_or_update_book(&mut self, db: &Database) -> Result<()> {
		let does_book_exist = if self.id != 0 {
			// TODO: Make sure we don't for some use a non-existent id and remove this block.
			Self::get_by_id(self.id, db).await?.is_some()
		} else {
			false
		};

		if does_book_exist {
			self.update_book(db).await?;

			Ok(())
		} else {
			let lock = db.write().await;

			lock.execute(r#"
				INSERT INTO book (
					title, clean_title, description, rating, thumb_url,
					cached, is_public,
					isbn_10, isbn_13,
					available_at, language,
					created_at, updated_at, deleted_at
				)
				VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)"#,
				params![
					&self.title, &self.clean_title, &self.description, self.rating, self.thumb_path.to_optional_string(),
					&self.cached.as_string_optional(), self.is_public,
					&self.isbn_10, &self.isbn_13,
					&self.available_at, self.language,
					self.created_at.timestamp_millis(), self.updated_at.timestamp_millis(),
					self.deleted_at.as_ref().map(|v| v.timestamp_millis()),
				]
			)?;

			self.id = BookId::from(lock.last_insert_rowid() as usize);

			drop(lock);

			Ok(())
		}
	}

	pub async fn update_book(&mut self, db: &Database) -> Result<()> {
		self.updated_at = Utc::now();

		db.write().await
		.execute(r#"
			UPDATE book SET
				title = ?2, clean_title = ?3, description = ?4, rating = ?5, thumb_url = ?6,
				cached = ?7, is_public = ?8,
				isbn_10 = ?9, isbn_13 = ?10,
				available_at = ?11, language = ?12,
				updated_at = ?13, deleted_at = ?14
			WHERE id = ?1"#,
			params![
				self.id,
				&self.title, &self.clean_title, &self.description, &self.rating, self.thumb_path.to_optional_string(),
				&self.cached.as_string_optional(), self.is_public,
				&self.isbn_10, &self.isbn_13,
				&self.available_at, &self.language,
				&self.updated_at.timestamp_millis(), self.deleted_at.as_ref().map(|v| v.timestamp_millis()),
			]
		)?;

		Ok(())
	}

	pub async fn get_by_id(id: BookId, db: &Database) -> Result<Option<Self>> {
		Ok(db.read().await.query_row(
			r#"SELECT * FROM book WHERE id = ?1"#,
			params![id],
			|v| Self::from_row(v)
		).optional()?)
	}

	pub async fn remove_by_id(id: BookId, db: &Database) -> Result<usize> {
		Ok(db.write().await.execute(
			r#"DELETE FROM book WHERE id = ?1"#,
			params![id]
		)?)
	}

	pub async fn get_book_by(offset: usize, limit: usize, _only_public: bool, person_id: Option<PersonId>, db: &Database) -> Result<Vec<Self>> {
		let this = db.read().await;

		let inner_query = if let Some(pid) = person_id {
			format!(
				r#"WHERE id = (SELECT book_id FROM book_person WHERE person_id = {})"#,
				pid
			)
		} else {
			String::new()
		};

		let mut conn = this.prepare(&format!("SELECT * FROM book {} LIMIT ?1 OFFSET ?2", inner_query))?;

		let map = conn.query_map([limit, offset], |v| Self::from_row(v))?;

		Ok(map.collect::<std::result::Result<Vec<_>, _>>()?)
	}


	fn gen_search_query(query: Option<&str>, only_public: bool, person_id: Option<PersonId>) -> Option<String> {
		let mut sql = String::from("SELECT * FROM book WHERE ");
		let orig_len = sql.len();

		// Only Public

		if only_public {
			sql += "is_public = true ";
		}


		// Query

		if let Some(query) = query.as_ref() {
			if only_public {
				sql += "AND ";
			}

			let mut escape_char = '\\';
			// Change our escape character if it's in the query.
			if query.contains(escape_char) {
				for car in [ '!', '@', '#', '$', '^', '&', '*', '-', '=', '+', '|', '~', '`', '/', '?', '>', '<', ',' ] {
					if !query.contains(car) {
						escape_char = car;
						break;
					}
				}
			}

			// TODO: Utilize title > clean_title > description, and sort
			sql += &format!(
				"title LIKE '%{}%' ESCAPE '{}' ",
				query.replace('%', &format!("{}%", escape_char)).replace('_', &format!("{}_", escape_char)),
				escape_char
			);
		}


		// Search with specific person

		if let Some(pid) = person_id {
			if only_public || query.is_some() {
				sql += "AND ";
			}

			sql += &format!(
				r#"id = (SELECT book_id FROM book_person WHERE person_id = {}) "#,
				pid
			);
		}


		if sql.len() == orig_len {
			// If sql is still unmodified
			None
		} else {
			Some(sql)
		}
	}

	pub async fn search_book_list(
		query: Option<&str>,
		offset: usize,
		limit: usize,
		only_public: bool,
		person_id: Option<PersonId>,
		db: &Database
	) -> Result<Vec<Self>> {
		let mut sql = match Self::gen_search_query(query, only_public, person_id) {
			Some(v) => v,
			None => return Ok(Vec::new())
		};

		sql += "LIMIT ?1 OFFSET ?2";

		let this = db.read().await;

		let mut conn = this.prepare(&sql)?;

		let map = conn.query_map(params![limit, offset], |v| Self::from_row(v))?;

		Ok(map.collect::<std::result::Result<Vec<_>, _>>()?)
	}

	pub async fn count_search_book(
		query: Option<&str>,
		only_public: bool,
		person_id: Option<PersonId>,
		db: &Database
	) -> Result<usize> {
		let sql = match Self::gen_search_query(query, only_public, person_id) {
			Some(v) => v.replace("SELECT *", "SELECT COUNT(*)"),
			None => return Ok(0)
		};

		Ok(db.read().await.query_row(&sql, [], |v| v.get(0))?)
	}
}