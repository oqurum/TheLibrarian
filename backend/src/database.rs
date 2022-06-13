use std::{sync::Arc, ops::Deref};

use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard, Semaphore, SemaphorePermit};

use crate::Result;
use rusqlite::Connection;
// TODO: use tokio::task::spawn_blocking;

const DATABASE_PATH: &str = "database.db";


pub async fn init() -> Result<Database> {
	let database = Database::open(5, || Ok(Connection::open(DATABASE_PATH)?))?;

	let conn = database.write().await;

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

			"permissions"	TEXT NOT NULL,

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

	// Uploaded Images
	conn.execute(
		r#"CREATE TABLE IF NOT EXISTS "uploaded_images" (
			"id"			INTEGER NOT NULL,

			"link_id"		INTEGER NOT NULL,
			"type_of"		INTEGER NOT NULL,

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
			"model_id"		INTEGER,

			"is_applied"	INTEGER NOT NULL,
			"vote_count"	INTEGER NOT NULL,

			"data"			TEXT NOT NULL,

			"ended_at"		DATETIME,
			"expires_at"	DATETIME,
			"created_at"	DATETIME NOT NULL,
			"updated_at"	DATETIME NOT NULL,

			PRIMARY KEY("id" AUTOINCREMENT)
		);"#,
		[]
	)?;

	// Edit Vote
	conn.execute(
		r#"CREATE TABLE IF NOT EXISTS "edit_vote" (
			"id"			INTEGER NOT NULL,

			"edit_id"		INTEGER NOT NULL,
			"member_id"		INTEGER NOT NULL,

			"vote"			INTEGER NOT NULL,

			"created_at"	DATETIME NOT NULL,

			UNIQUE("edit_id", "member_id"),
			PRIMARY KEY("id" AUTOINCREMENT)
		);"#,
		[]
	)?;

	// Edit Comment
	conn.execute(
		r#"CREATE TABLE IF NOT EXISTS "edit_comment" (
			"id"			INTEGER NOT NULL,

			"edit_id"		INTEGER NOT NULL,
			"member_id"		INTEGER NOT NULL,

			"text"			TEXT NOT NULL,
			"deleted"		INTEGER NOT NULL,

			"created_at"	DATETIME NOT NULL,

			PRIMARY KEY("id" AUTOINCREMENT)
		);"#,
		[]
	)?;

	drop(conn);

	Ok(database)
}



pub struct Database {
	// Using RwLock to engage the r/w locks.
	lock: RwLock<()>,

	// Store all our open connections to the database.
	read_conns: Vec<Arc<Connection>>,

	// Max concurrent read connections
	max_read_aquires: Semaphore,
}

unsafe impl Send for Database {}
unsafe impl Sync for Database {}

impl Database {
	pub fn open<F: Fn() -> Result<Connection>>(count: usize, open_conn: F) -> Result<Self> {
		let mut read_conns = Vec::new();

		for _ in 0..count {
			read_conns.push(Arc::new(open_conn()?));
		}

		Ok(Self {
			lock: RwLock::new(()),

			read_conns,

			max_read_aquires: Semaphore::new(count),
		})
	}

	pub async fn read(&self) -> DatabaseReadGuard<'_> {
		let _guard = self.lock.read().await;

		let _permit = self.max_read_aquires.acquire().await.unwrap();

		let conn = {
			let mut value = None;

			for conn in &self.read_conns {
				let new_conn = conn.clone();
				// TODO: Race Conditions
				// Strong count should eq 2 (original + cloned)
				if Arc::strong_count(&new_conn) == 2 {
					value = Some(new_conn);
					break;
				}
			}

			// This should never be reached.
			#[allow(clippy::expect_used)]
			value.expect("Unable to find available Read Connection")
		};

		DatabaseReadGuard {
			_permit,
			_guard,
			conn
		}
	}

	pub async fn write(&self) -> DatabaseWriteGuard<'_> {
		let _guard = self.lock.write().await;

		DatabaseWriteGuard {
			_guard,
			conn: &*self.read_conns[0]
		}
	}
}


pub struct DatabaseReadGuard<'a> {
	_permit: SemaphorePermit<'a>,
	_guard: RwLockReadGuard<'a, ()>,
	conn: Arc<Connection>,
}

impl<'a> Deref for DatabaseReadGuard<'a> {
	type Target = Connection;

	fn deref(&self) -> &Self::Target {
		&*self.conn
	}
}


pub struct DatabaseWriteGuard<'a> {
	_guard: RwLockWriteGuard<'a, ()>,
	conn: &'a Connection,
}

impl<'a> Deref for DatabaseWriteGuard<'a> {
	type Target = Connection;

	fn deref(&self) -> &Self::Target {
		self.conn
	}
}




#[cfg(test)]
mod tests {
	use super::*;

	use std::{thread, time::Duration};
	use tokio::{runtime::Runtime, time::sleep, sync::Mutex};


	fn create_db() -> Result<Arc<Database>> {
		Ok(Arc::new(Database::open(4, || Ok(Connection::open_in_memory()?))?))
	}


	#[test]
	fn write_read() -> Result<()> {
		let database = create_db()?;

		let value = Arc::new(Mutex::new(false));

		let handle_read = {
			let db2 = database.clone();
			let val2 = value.clone();

			thread::spawn(move || {
				Runtime::new().unwrap()
				.block_on(async {
					sleep(Duration::from_millis(100)).await;

					let _read = db2.read().await;

					assert!(*val2.lock().await);
				});
			})
		};



		Runtime::new().unwrap()
		.block_on(async {
			let _write = database.write().await;

			*value.lock().await = true;
		});

		handle_read.join().unwrap();

		Ok(())
	}

	#[test]
	fn multiread_write_read() -> Result<()> {
		let database = create_db()?;

		let value = Arc::new(Mutex::new(false));

		// Create 5 reads
		let handle_reads = (0..5usize)
			.map(|_| {
				let db2 = database.clone();
				let val2 = value.clone();

				thread::spawn(move || {
					Runtime::new().unwrap()
					.block_on(async {
						let _read = db2.read().await;

						sleep(Duration::from_millis(100)).await;

						assert!(!*val2.lock().await);
					});
				})
			})
			.collect::<Vec<_>>();

		// Write
		Runtime::new().unwrap()
		.block_on(async {
			sleep(Duration::from_millis(150)).await;

			let _write = database.write().await;

			*value.lock().await = true;
		});

		for handle_read in handle_reads {
			handle_read.join().unwrap();
		}

		// Read again
		Runtime::new().unwrap()
		.block_on(async {
			let _read = database.read().await;

			assert!(*value.lock().await);
		});

		Ok(())
	}

	// #[test]
	// fn multiple_reads() {}

	// #[test]
	// fn single_writes() {}
}