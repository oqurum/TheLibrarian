use std::sync::{Mutex, MutexGuard};

use crate::Result;
use rusqlite::{Connection, params, OptionalExtension};
// TODO: use tokio::task::spawn_blocking;

use crate::model::*;

pub async fn init() -> Result<Database> {
	let conn = rusqlite::Connection::open("database.db")?;

	// Book
	conn.execute(
		r#"CREATE TABLE IF NOT EXISTS "book" (
			"id"					INTEGER NOT NULL,

			"title"					TEXT,
			"clean_title"			TEXT,
			"description"			TEXT,
			"rating"				FLOAT,
			"thumb_url"				TEXT,

			"cached"				TEXT,

			"isbn_10"				TEXT,
			"isbn_13"				TEXT,

			"is_public"				INTEGER NOT NULL,
			"edition_count"			INTEGER NOT NULL DEFAULT 0,

			"available_at"			DATETIME,
			"language"				INTEGER,

			"created_at"			DATETIME,
			"updated_at"			DATETIME,
			"deleted_at"			DATETIME,

			PRIMARY KEY("id" AUTOINCREMENT)
		);"#,
		[]
	)?;

	// Book People
	conn.execute(
		r#"CREATE TABLE IF NOT EXISTS "book_person" (
			"book_id"		INTEGER NOT NULL,
			"person_id"		INTEGER NOT NULL,

			UNIQUE(book_id, person_id)
		);"#,
		[]
	)?;

	// People
	conn.execute(
		r#"CREATE TABLE IF NOT EXISTS "person" (
			"id"			INTEGER NOT NULL,

			"source" 		TEXT NOT NULL,

			"name"			TEXT NOT NULL COLLATE NOCASE,
			"description"	TEXT,
			"birth_date"	INTEGER,

			"thumb_url"		TEXT,

			"updated_at" 	DATETIME NOT NULL,
			"created_at" 	DATETIME NOT NULL,

			PRIMARY KEY("id" AUTOINCREMENT)
		);"#,
		[]
	)?;

	// People Other names
	conn.execute(
		r#"CREATE TABLE IF NOT EXISTS "person_alt" (
			"person_id"		INTEGER NOT NULL,

			"name"			TEXT NOT NULL COLLATE NOCASE,

			UNIQUE(person_id, name)
		);"#,
		[]
	)?;

	// Members
	conn.execute(
		r#"CREATE TABLE IF NOT EXISTS "members" (
			"id"			INTEGER NOT NULL,

			"name"			TEXT NOT NULL COLLATE NOCASE,
			"email"			TEXT COLLATE NOCASE,
			"password"		TEXT,
			"is_local"		INTEGER NOT NULL,
			"config"		TEXT,

			"created_at" 	DATETIME NOT NULL,
			"updated_at" 	DATETIME NOT NULL,

			UNIQUE(email),
			PRIMARY KEY("id" AUTOINCREMENT)
		);"#,
		[]
	)?;

	// Auths
	conn.execute(
		r#"CREATE TABLE IF NOT EXISTS "auths" (
			"oauth_token"			TEXT NOT NULL,
			"oauth_token_secret"	TEXT NOT NULL,

			"created_at"			DATETIME NOT NULL,

			UNIQUE(oauth_token)
		);"#,
		[]
	)?;

	// Tags
	conn.execute(
		r#"CREATE TABLE IF NOT EXISTS "tags" (
			"id"			INTEGER NOT NULL,

			"name"			TEXT NOT NULL COLLATE NOCASE,
			"type_of"		INTEGER NOT NULL,

			"data"			TEXT,

			"created_at" 	DATETIME NOT NULL,
			"updated_at" 	DATETIME NOT NULL,

			PRIMARY KEY("id" AUTOINCREMENT),
			UNIQUE("name", "type_of")
		);"#,
		[]
	)?;

	// Book Tags
	conn.execute(
		r#"CREATE TABLE IF NOT EXISTS "book_tags" (
			"id"			INTEGER NOT NULL,

			"book_id"		INTEGER NOT NULL,
			"tag_id"		INTEGER NOT NULL,

			"windex"		INTEGER NOT NULL,

			"created_at" 	DATETIME NOT NULL,

			PRIMARY KEY("id" AUTOINCREMENT),
			UNIQUE("book_id", "tag_id")
		);"#,
		[]
	)?;


	// TODO: type_of for Author, Book Meta, etc..
	// Uploaded Images
	conn.execute(
		r#"CREATE TABLE IF NOT EXISTS "uploaded_images" (
			"id"			INTEGER NOT NULL,

			"link_id"		INTEGER NOT NULL,

			"path"			TEXT NOT NULL,

			"created_at"	DATETIME NOT NULL,

			UNIQUE(link_id, path),
			PRIMARY KEY("id" AUTOINCREMENT)
		);"#,
		[]
	)?;

	Ok(Database(Mutex::new(conn)))
}

pub struct Database(Mutex<Connection>);


impl Database {
	pub fn lock(&self) -> Result<MutexGuard<Connection>> {
		Ok(self.0.lock()?)
	}

	// Search

	fn gen_search_query(query: Option<&str>, only_public: bool, person_id: Option<usize>) -> Option<String> {
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

	pub fn search_book_list(&self, query: Option<&str>, offset: usize, limit: usize, only_public: bool, person_id: Option<usize>) -> Result<Vec<BookModel>> {
		let mut sql = match Self::gen_search_query(query, only_public, person_id) {
			Some(v) => v,
			None => return Ok(Vec::new())
		};

		sql += "LIMIT ?1 OFFSET ?2";

		let this = self.lock()?;

		let mut conn = this.prepare(&sql)?;

		let map = conn.query_map(params![limit, offset], |v| BookModel::try_from(v))?;

		Ok(map.collect::<std::result::Result<Vec<_>, _>>()?)
	}

	pub fn count_search_book(&self, query: Option<&str>, only_public: bool, person_id: Option<usize>) -> Result<usize> {
		let sql = match Self::gen_search_query(query, only_public, person_id) {
			Some(v) => v.replace("SELECT *", "SELECT COUNT(*)"),
			None => return Ok(0)
		};

		Ok(self.lock()?.query_row(&sql, [], |v| v.get(0))?)
	}


	// Poster

	pub fn add_poster(&self, poster: &NewPosterModel) -> Result<usize> {
		if poster.path.is_none() {
			return Ok(0);
		}

		let conn = self.lock()?;

		conn.execute(r#"
			INSERT OR IGNORE INTO uploaded_images (link_id, path, created_at)
			VALUES (?1, ?2, ?3)
		"#,
		params![
			poster.link_id,
			poster.path.to_string(),
			poster.created_at.timestamp_millis()
		])?;

		Ok(conn.last_insert_rowid() as usize)
	}

	pub fn get_posters_by_linked_id(&self, id: usize) -> Result<Vec<PosterModel>> {
		let this = self.lock()?;

		let mut conn = this.prepare(r#"SELECT * FROM uploaded_images WHERE link_id = ?1"#)?;

		let map = conn.query_map([id], |v| PosterModel::try_from(v))?;

		Ok(map.collect::<std::result::Result<Vec<_>, _>>()?)
	}

	pub fn get_poster_by_id(&self, id: usize) -> Result<Option<PosterModel>> {
		Ok(self.lock()?.query_row(
			r#"SELECT * FROM uploaded_images WHERE id = ?1 LIMIT 1"#,
			params![id],
			|v| PosterModel::try_from(v)
		).optional()?)
	}
}