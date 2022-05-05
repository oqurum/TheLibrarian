use std::sync::{Mutex, MutexGuard};

use crate::Result;
use librarian_common::api;
use rusqlite::{Connection, params, OptionalExtension};
// TODO: use tokio::task::spawn_blocking;

pub mod table;
use table::*;


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

			"tags_genre"			TEXT,
			"tags_collection"		TEXT,
			"tags_author"			TEXT,
			"tags_country"			TEXT,

			"available_at"			DATETIME,
			"year"					INTEGER,

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

// TODO: Replace with tokio Mutex?
pub struct Database(Mutex<Connection>);


impl Database {
	fn lock(&self) -> Result<MutexGuard<Connection>> {
		Ok(self.0.lock()?)
	}


	// Book

	pub fn get_book_count(&self) -> Result<usize> {
		Ok(self.lock()?.query_row(r#"SELECT COUNT(*) FROM book"#, [], |v| v.get(0))?)
	}

	pub fn add_or_update_book(&self, meta: &BookModel) -> Result<BookModel> {
		let table_meta = if meta.id != 0 {
			self.get_book_by_id(meta.id)?
		} else {
			None
		};

		if let Some(og_meta) = table_meta {
			self.update_book(meta)?;
			self.get_book_by_id(og_meta.id).map(|v| v.unwrap())
		} else {
			let lock = self.lock()?;

			lock.execute(r#"
				INSERT INTO book (
					title, clean_title, description, rating, thumb_url,
					cached,
					isbn_10, isbn_13, tags_author, tags_country,
					available_at, year,
					created_at, updated_at, deleted_at
				)
				VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)"#,
				params![
					&meta.title, &meta.clean_title, &meta.description, &meta.rating, meta.thumb_path.to_optional_string(),
					&meta.cached.as_string_optional(),
					&meta.isbn_10, &meta.isbn_13, &meta.tags_author, &meta.tags_country,
					&meta.available_at, &meta.year,
					&meta.created_at.timestamp_millis(), &meta.updated_at.timestamp_millis(),
					meta.deleted_at.as_ref().map(|v| v.timestamp_millis()),
				]
			)?;

			let id = lock.last_insert_rowid() as usize;

			drop(lock);

			Ok(self.get_book_by_id(id)?.unwrap())
		}
	}

	pub fn update_book(&self, meta: &BookModel) -> Result<()> {
		self.lock()?
		.execute(r#"
			UPDATE book SET
				title = ?2, clean_title = ?3, description = ?4, rating = ?5, thumb_url = ?6,
				cached = ?7,
				isbn_10 = ?8, isbn_13 = ?9, tags_author = ?10, tags_country = ?11,
				available_at = ?12, year = ?13,
				created_at = ?14, updated_at = ?15, deleted_at = ?16
			WHERE id = ?1"#,
			params![
				meta.id,
				&meta.title, &meta.clean_title, &meta.description, &meta.rating, meta.thumb_path.to_optional_string(),
				&meta.cached.as_string_optional(),
				&meta.isbn_10, &meta.isbn_13, &meta.tags_author, &meta.tags_country,
				&meta.available_at, &meta.year,
				&meta.created_at.timestamp_millis(), &meta.updated_at.timestamp_millis(),
				meta.deleted_at.as_ref().map(|v| v.timestamp_millis()),
			]
		)?;

		Ok(())
	}

	pub fn get_book_by_id(&self, id: usize) -> Result<Option<BookModel>> {
		Ok(self.lock()?.query_row(
			r#"SELECT * FROM book WHERE id = ?1 LIMIT 1"#,
			params![id],
			|v| BookModel::try_from(v)
		).optional()?)
	}

	pub fn remove_book_by_id(&self, id: usize) -> Result<usize> {
		Ok(self.lock()?.execute(
			r#"DELETE FROM book WHERE id = ?1"#,
			params![id]
		)?)
	}

	pub fn get_book_by(&self, offset: usize, limit: usize) -> Result<Vec<BookModel>> {
		let this = self.lock()?;

		let mut conn = this.prepare("SELECT * FROM book LIMIT ?1 OFFSET ?2")?;

		let map = conn.query_map([limit, offset], |v| BookModel::try_from(v))?;

		Ok(map.collect::<std::result::Result<Vec<_>, _>>()?)
	}


	// Search

	fn gen_search_query(search: &api::SearchQuery) -> Option<String> {
		let mut sql = String::from("SELECT * FROM book WHERE ");
		let orig_len = sql.len();

		// Query

		if let Some(query) = search.query.as_deref() {
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


		// Source

		if let Some(source) = search.source.as_deref() {
			if search.query.is_some() {
				sql += "AND ";
			}

			sql += &format!("source LIKE '{}%' ", source);
		}

		if sql.len() == orig_len {
			// If sql is still unmodified
			None
		} else {
			Some(sql)
		}
	}

	pub fn search_book_list(&self, search: &api::SearchQuery, offset: usize, limit: usize) -> Result<Vec<BookModel>> {
		let mut sql = match Self::gen_search_query(search) {
			Some(v) => v,
			None => return Ok(Vec::new())
		};

		sql += "LIMIT ?1 OFFSET ?2";

		let this = self.lock()?;

		let mut conn = this.prepare(&sql)?;

		let map = conn.query_map(params![limit, offset], |v| BookModel::try_from(v))?;

		Ok(map.collect::<std::result::Result<Vec<_>, _>>()?)
	}

	pub fn count_search_book(&self, search: &api::SearchQuery) -> Result<usize> {
		let sql = match Self::gen_search_query(search) {
			Some(v) => v.replace("SELECT *", "SELECT COUNT(*)"),
			None => return Ok(0)
		};

		Ok(self.lock()?.query_row(&sql, [], |v| v.get(0))?)
	}


	// Book Person

	pub fn add_book_person(&self, person: &BookPersonModel) -> Result<()> {
		self.lock()?.execute(r#"INSERT OR IGNORE INTO book_person (book_id, person_id) VALUES (?1, ?2)"#,
		params![
			&person.book_id,
			&person.person_id
		])?;

		Ok(())
	}

	pub fn remove_book_person(&self, person: &BookPersonModel) -> Result<()> {
		self.lock()?.execute(r#"DELETE FROM book_person WHERE book_id = ?1 AND person_id = ?2"#,
		params![
			&person.book_id,
			&person.person_id
		])?;

		Ok(())
	}

	pub fn remove_persons_by_book_id(&self, id: usize) -> Result<()> {
		self.lock()?.execute(r#"DELETE FROM book_person WHERE book_id = ?1"#,
		params![
			id
		])?;

		Ok(())
	}

	pub fn remove_book_person_by_person_id(&self, id: usize) -> Result<()> {
		self.lock()?.execute(r#"DELETE FROM book_person WHERE person_id = ?1"#,
		params![
			id
		])?;

		Ok(())
	}

	pub fn transfer_book_person(&self, from_id: usize, to_id: usize) -> Result<usize> {
		Ok(self.lock()?.execute(r#"UPDATE book_person SET person_id = ?2 WHERE person_id = ?1"#,
		params![
			from_id,
			to_id
		])?)
	}

	pub fn get_book_person_list(&self, id: usize) -> Result<Vec<BookPersonModel>> {
		let this = self.lock()?;

		let mut conn = this.prepare(r#"SELECT * FROM book_person WHERE book_id = ?1"#)?;

		let map = conn.query_map([id], |v| BookPersonModel::try_from(v))?;

		Ok(map.collect::<std::result::Result<Vec<_>, _>>()?)
	}


	// Person

	pub fn add_person(&self, person: &NewPersonModel) -> Result<usize> {
		let conn = self.lock()?;

		conn.execute(r#"
			INSERT INTO person (source, name, description, birth_date, thumb_url, updated_at, created_at)
			VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
		"#,
		params![
			person.source.to_string(), &person.name, &person.description, &person.birth_date, person.thumb_url.to_optional_string(),
			person.updated_at.timestamp_millis(), person.created_at.timestamp_millis()
		])?;

		Ok(conn.last_insert_rowid() as usize)
	}

	pub fn get_person_list(&self, offset: usize, limit: usize) -> Result<Vec<TagPersonModel>> {
		let this = self.lock()?;

		let mut conn = this.prepare(r#"SELECT * FROM person LIMIT ?1 OFFSET ?2"#)?;

		let map = conn.query_map([limit, offset], |v| TagPersonModel::try_from(v))?;

		Ok(map.collect::<std::result::Result<Vec<_>, _>>()?)
	}

	pub fn get_person_list_by_meta_id(&self, id: usize) -> Result<Vec<TagPersonModel>> {
		let this = self.lock()?;

		let mut conn = this.prepare(r#"
			SELECT person.* FROM book_person
			LEFT JOIN
				person ON person.id = book_person.person_id
			WHERE book_id = ?1
		"#)?;

		let map = conn.query_map([id], |v| TagPersonModel::try_from(v))?;

		Ok(map.collect::<std::result::Result<Vec<_>, _>>()?)
	}

	pub fn search_person_list(&self, query: &str, offset: usize, limit: usize) -> Result<Vec<TagPersonModel>> {
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

		let sql = format!(
			r#"SELECT * FROM person WHERE name LIKE '%{}%' ESCAPE '{}' LIMIT ?1 OFFSET ?2"#,
			query.replace('%', &format!("{}%", escape_char)).replace('_', &format!("{}_", escape_char)),
			escape_char
		);


		let this = self.lock()?;

		let mut conn = this.prepare(&sql)?;

		let map = conn.query_map(params![limit, offset], |v| TagPersonModel::try_from(v))?;

		Ok(map.collect::<std::result::Result<Vec<_>, _>>()?)
	}

	pub fn get_person_by_name(&self, value: &str) -> Result<Option<TagPersonModel>> {
		let person = self.lock()?.query_row(
			r#"SELECT * FROM person WHERE name = ?1 LIMIT 1"#,
			params![value],
			|v| TagPersonModel::try_from(v)
		).optional()?;

		if let Some(person) = person {
			Ok(Some(person))
		} else if let Some(alt) = self.get_person_alt_by_name(value)? {
			self.get_person_by_id(alt.person_id)
		} else {
			Ok(None)
		}
	}

	pub fn get_person_by_id(&self, id: usize) -> Result<Option<TagPersonModel>> {
		Ok(self.lock()?.query_row(
			r#"SELECT * FROM person WHERE id = ?1 LIMIT 1"#,
			params![id],
			|v| TagPersonModel::try_from(v)
		).optional()?)
	}

	pub fn get_person_by_source(&self, value: &str) -> Result<Option<TagPersonModel>> {
		Ok(self.lock()?.query_row(
			r#"SELECT * FROM person WHERE source = ?1 LIMIT 1"#,
			params![value],
			|v| TagPersonModel::try_from(v)
		).optional()?)
	}

	pub fn get_person_count(&self) -> Result<usize> {
		Ok(self.lock()?.query_row(r#"SELECT COUNT(*) FROM person"#, [], |v| v.get(0))?)
	}

	pub fn update_person(&self, person: &TagPersonModel) -> Result<()> {
		self.lock()?
		.execute(r#"
			UPDATE person SET
				source = ?2,
				name = ?3,
				description = ?4,
				birth_date = ?5,
				thumb_url = ?6,
				updated_at = ?7,
				created_at = ?8
			WHERE id = ?1"#,
			params![
				person.id,
				person.source.to_string(), &person.name, &person.description, &person.birth_date, person.thumb_url.to_string(),
				person.updated_at.timestamp_millis(), person.created_at.timestamp_millis()
			]
		)?;

		Ok(())
	}

	pub fn remove_person_by_id(&self, id: usize) -> Result<usize> {
		Ok(self.lock()?.execute(
			r#"DELETE FROM person WHERE id = ?1"#,
			params![id]
		)?)
	}


	// Person Alt

	pub fn add_person_alt(&self, person: &TagPersonAltModel) -> Result<()> {
		self.lock()?.execute(r#"INSERT INTO person_alt (name, person_id) VALUES (?1, ?2)"#,
		params![
			&person.name, &person.person_id
		])?;

		Ok(())
	}

	pub fn get_person_alt_by_name(&self, value: &str) -> Result<Option<TagPersonAltModel>> {
		Ok(self.lock()?.query_row(
			r#"SELECT * FROM person_alt WHERE name = ?1 LIMIT 1"#,
			params![value],
			|v| TagPersonAltModel::try_from(v)
		).optional()?)
	}

	pub fn remove_person_alt(&self, person: &TagPersonAltModel) -> Result<usize> {
		Ok(self.lock()?.execute(
			r#"DELETE FROM person_alt WHERE name = ?1 AND person_id = ?2"#,
			params![
				&person.name,
				&person.person_id
			]
		)?)
	}

	pub fn remove_person_alt_by_person_id(&self, id: usize) -> Result<usize> {
		Ok(self.lock()?.execute(
			r#"DELETE FROM person_alt WHERE person_id = ?1"#,
			params![id]
		)?)
	}

	pub fn transfer_person_alt(&self, from_id: usize, to_id: usize) -> Result<usize> {
		Ok(self.lock()?.execute(r#"UPDATE OR IGNORE person_alt SET person_id = ?2 WHERE person_id = ?1"#,
		params![
			from_id,
			to_id
		])?)
	}


	// Members

	pub fn add_member(&self, member: &NewMemberModel) -> Result<usize> {
		let conn = self.lock()?;

		conn.execute(r#"
			INSERT INTO members (name, email, password, is_local, config, created_at, updated_at)
			VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
		"#,
		params![
			&member.name, member.email.as_ref(), member.password.as_ref(), member.type_of, member.config.as_ref(),
			member.created_at.timestamp_millis(), member.updated_at.timestamp_millis()
		])?;

		Ok(conn.last_insert_rowid() as usize)
	}

	pub fn get_member_by_email(&self, value: &str) -> Result<Option<MemberModel>> {
		Ok(self.lock()?.query_row(
			r#"SELECT * FROM members WHERE email = ?1 LIMIT 1"#,
			params![value],
			|v| MemberModel::try_from(v)
		).optional()?)
	}

	pub fn get_member_by_id(&self, id: usize) -> Result<Option<MemberModel>> {
		Ok(self.lock()?.query_row(
			r#"SELECT * FROM members WHERE id = ?1 LIMIT 1"#,
			params![id],
			|v| MemberModel::try_from(v)
		).optional()?)
	}


	// Verify

	pub fn add_verify(&self, auth: &NewAuthModel) -> Result<usize> {
		let conn = self.lock()?;

		conn.execute(r#"
			INSERT INTO auths (oauth_token, oauth_token_secret, created_at)
			VALUES (?1, ?2, ?3)
		"#,
		params![
			&auth.oauth_token,
			&auth.oauth_token_secret,
			auth.created_at.timestamp_millis()
		])?;

		Ok(conn.last_insert_rowid() as usize)
	}

	pub fn remove_verify_if_found_by_oauth_token(&self, value: &str) -> Result<bool> {
		Ok(self.lock()?.execute(
			r#"DELETE FROM auths WHERE oauth_token = ?1 LIMIT 1"#,
			params![value],
		)? != 0)
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