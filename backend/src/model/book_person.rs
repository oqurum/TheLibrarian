use rusqlite::{Row, params};
use serde::Serialize;

use crate::{Result, Database};

#[derive(Debug, Serialize)]
pub struct BookPersonModel {
	pub book_id: usize,
	pub person_id: usize,
}

impl<'a> TryFrom<&Row<'a>> for BookPersonModel {
	type Error = rusqlite::Error;

	fn try_from(value: &Row<'a>) -> std::result::Result<Self, Self::Error> {
		Ok(Self {
			book_id: value.get(0)?,
			person_id: value.get(1)?,
		})
	}
}


impl BookPersonModel {
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

	pub async fn remove_by_book_id(id: usize, db: &Database) -> Result<()> {
		db.write().await
		.execute(r#"DELETE FROM book_person WHERE book_id = ?1"#,
		params![
			id
		])?;

		Ok(())
	}

	pub async fn remove_by_person_id(id: usize, db: &Database) -> Result<()> {
		db.write().await
		.execute(r#"DELETE FROM book_person WHERE person_id = ?1"#,
			params![
				id
			]
		)?;

		Ok(())
	}

	pub async fn transfer(from_id: usize, to_id: usize, db: &Database) -> Result<usize> {
		Ok(db.write().await
		.execute(r#"UPDATE book_person SET person_id = ?2 WHERE person_id = ?1"#,
			params![
				from_id,
				to_id
			]
		)?)
	}

	pub async fn get_all_by_book_id(book_id: usize, db: &Database) -> Result<Vec<Self>> {
		let this = db.read().await;

		let mut conn = this.prepare(r#"SELECT * FROM book_person WHERE book_id = ?1"#)?;

		let map = conn.query_map([book_id], |v| Self::try_from(v))?;

		Ok(map.collect::<std::result::Result<Vec<_>, _>>()?)
	}
}