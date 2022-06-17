use rusqlite::{params, OptionalExtension};

use serde::Serialize;

use librarian_common::PersonId;

use crate::{Database, Result};

use super::{TableRow, AdvRow};

#[derive(Debug, Serialize)]
pub struct PersonAltModel {
	pub person_id: PersonId,
	pub name: String,
}

impl TableRow<'_> for PersonAltModel {
	fn create(row: &mut AdvRow<'_>) -> rusqlite::Result<Self> {
		Ok(Self {
			person_id: row.next()?,
			name: row.next()?,
		})
	}
}


impl PersonAltModel {
	pub async fn insert(&self, db: &Database) -> Result<()> {
		db.write().await
		.execute(r#"INSERT INTO person_alt (name, person_id) VALUES (?1, ?2)"#,
		params![
			&self.name, &self.person_id
		])?;

		Ok(())
	}

	pub async fn remove(&self, db: &Database) -> Result<usize> {
		Ok(db.write().await.execute(
			r#"DELETE FROM person_alt WHERE name = ?1 AND person_id = ?2"#,
			params![
				&self.name,
				&self.person_id
			]
		)?)
	}


	pub async fn get_by_name(value: &str, db: &Database) -> Result<Option<PersonAltModel>> {
		Ok(db.read().await.query_row(
			r#"SELECT * FROM person_alt WHERE name = ?1 LIMIT 1"#,
			params![value],
			|v| PersonAltModel::from_row(v)
		).optional()?)
	}

	pub async fn remove_by_person_id(id: PersonId, db: &Database) -> Result<usize> {
		Ok(db.write().await.execute(
			r#"DELETE FROM person_alt WHERE person_id = ?1"#,
			params![id]
		)?)
	}

	pub async fn transfer_by_person_id(&self, from_id: PersonId, to_id: PersonId, db: &Database) -> Result<usize> {
		Ok(db.write().await.execute(r#"UPDATE OR IGNORE person_alt SET person_id = ?2 WHERE person_id = ?1"#,
		params![
			from_id,
			to_id
		])?)
	}
}