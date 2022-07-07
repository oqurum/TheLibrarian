use common::{BookId, PersonId};
use rusqlite::params;
use serde::Serialize;

use crate::{Result, Database};
use super::{AdvRow, TableRow};

#[derive(Debug, Serialize)]
pub struct BookPersonModel {
	pub book_id: BookId,
	pub person_id: PersonId,
}

impl TableRow<'_> for BookPersonModel {
	fn create(row: &mut AdvRow<'_>) -> rusqlite::Result<Self> {
		Ok(Self {
			book_id: row.next()?,
			person_id: row.next()?,
		})
	}
}


impl BookPersonModel {
	pub fn new(book_id: BookId, person_id: PersonId) -> Self {
		Self { book_id, person_id }
	}

	pub async fn insert(&self, db: &Database) -> Result<()> {
		db.write().await
		.execute(r#"INSERT OR IGNORE INTO book_person (book_id, person_id) VALUES (?1, ?2)"#,
			params![
				&self.book_id,
				&self.person_id
			]
		)?;

		Ok(())
	}

	pub async fn remove(&self, db: &Database) -> Result<()> {
		db.write().await
		.execute(r#"DELETE FROM book_person WHERE book_id = ?1 AND person_id = ?2"#,
			params![
				&self.book_id,
				&self.person_id
			]
		)?;

		Ok(())
	}

	pub async fn remove_by_book_id(id: BookId, db: &Database) -> Result<()> {
		db.write().await
		.execute(r#"DELETE FROM book_person WHERE book_id = ?1"#,
		params![
			id
		])?;

		Ok(())
	}

	pub async fn remove_by_person_id(id: PersonId, db: &Database) -> Result<()> {
		db.write().await
		.execute(r#"DELETE FROM book_person WHERE person_id = ?1"#,
			params![
				id
			]
		)?;

		Ok(())
	}

	pub async fn transfer(from_id: PersonId, to_id: PersonId, db: &Database) -> Result<usize> {
		Ok(db.write().await
		.execute(r#"UPDATE book_person SET person_id = ?2 WHERE person_id = ?1"#,
			params![
				from_id,
				to_id
			]
		)?)
	}

	pub async fn get_all_by_book_id(book_id: BookId, db: &Database) -> Result<Vec<Self>> {
		let this = db.read().await;

		let mut conn = this.prepare(r#"SELECT * FROM book_person WHERE book_id = ?1"#)?;

		let map = conn.query_map([book_id], |v| Self::from_row(v))?;

		Ok(map.collect::<std::result::Result<Vec<_>, _>>()?)
	}
}