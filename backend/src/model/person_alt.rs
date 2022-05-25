use rusqlite::{Row, params, OptionalExtension};

use serde::Serialize;

use crate::{Database, Result};

#[derive(Debug, Serialize)]
pub struct PersonAltModel {
	pub person_id: usize,
	pub name: String,
}

impl<'a> TryFrom<&Row<'a>> for PersonAltModel {
	type Error = rusqlite::Error;

	fn try_from(value: &Row<'a>) -> std::result::Result<Self, Self::Error> {
		Ok(Self {
			person_id: value.get(0)?,
			name: value.get(1)?,
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
			|v| PersonAltModel::try_from(v)
		).optional()?)
	}

	pub async fn remove_by_person_id(id: usize, db: &Database) -> Result<usize> {
		Ok(db.write().await.execute(
			r#"DELETE FROM person_alt WHERE person_id = ?1"#,
			params![id]
		)?)
	}

	pub async fn transfer_by_person_id(&self, from_id: usize, to_id: usize, db: &Database) -> Result<usize> {
		Ok(db.write().await.execute(r#"UPDATE OR IGNORE person_alt SET person_id = ?2 WHERE person_id = ?1"#,
		params![
			from_id,
			to_id
		])?)
	}
}