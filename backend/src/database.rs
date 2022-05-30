use std::sync::Arc;

use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::Result;
use rusqlite::Connection;
// TODO: use tokio::task::spawn_blocking;


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


	// Edit
	conn.execute(
		r#"CREATE TABLE IF NOT EXISTS "edit" (
			"id"			INTEGER NOT NULL,

			"type_of"		INTEGER NOT NULL,
			"operation"		INTEGER NOT NULL,
			"status"		INTEGER NOT NULL,

			"member_id"		INTEGER NOT NULL,

			"is_applied"	INTEGER NOT NULL,
			"vote_count"	INTEGER NOT NULL,

			"data"			TEXT NOT NULL,

			"ended_at"		DATETIME NOT NULL,
			"expires_at"	DATETIME NOT NULL,
			"created_at"	DATETIME NOT NULL,
			"updated_at"	DATETIME NOT NULL,

			PRIMARY KEY("id" AUTOINCREMENT)
		);"#,
		[]
	)?;



	Ok(Database(Arc::new(RwLock::new(conn))))
}

pub struct Database(Arc<RwLock<Connection>>);

// TODO: Why did Mutex<Connection> work without this but Arc<tokio::RwLock<Connection>> doesn't.
unsafe impl Send for Database {}
unsafe impl Sync for Database {}


impl Database {
	pub async fn read(&self) -> RwLockReadGuard<'_, Connection> {
		self.0.read().await
	}

	pub async fn write(&self) -> RwLockWriteGuard<'_, Connection> {
		self.0.write().await
	}
}