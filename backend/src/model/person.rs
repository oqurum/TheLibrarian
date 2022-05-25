use librarian_common::{Person, Source, ThumbnailStore, util::serialize_datetime};
use chrono::{DateTime, TimeZone, Utc};
use rusqlite::{Row, params, OptionalExtension};
use serde::Serialize;

use crate::{Database, Result};

use super::PersonAltModel;



#[derive(Debug)]
pub struct NewPersonModel {
	pub source: Source,

	pub name: String,
	pub description: Option<String>,
	pub birth_date: Option<String>,

	pub thumb_url: ThumbnailStore,

	pub updated_at: DateTime<Utc>,
	pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct PersonModel {
	pub id: usize,

	pub source: Source,

	pub name: String,
	pub description: Option<String>,
	pub birth_date: Option<String>,

	pub thumb_url: ThumbnailStore,

	#[serde(serialize_with = "serialize_datetime")]
	pub updated_at: DateTime<Utc>,
	#[serde(serialize_with = "serialize_datetime")]
	pub created_at: DateTime<Utc>,
}

impl<'a> TryFrom<&Row<'a>> for PersonModel {
	type Error = rusqlite::Error;

	fn try_from(value: &Row<'a>) -> std::result::Result<Self, Self::Error> {
		Ok(Self {
			id: value.get(0)?,

			source: Source::try_from(value.get::<_, String>(1)?).unwrap(),

			name: value.get(2)?,
			description: value.get(3)?,
			birth_date: value.get(4)?,

			thumb_url: ThumbnailStore::from(value.get::<_, Option<String>>(5)?),

			created_at: Utc.timestamp_millis(value.get(6)?),
			updated_at: Utc.timestamp_millis(value.get(7)?),
		})
	}
}

impl From<PersonModel> for Person {
	fn from(val: PersonModel) -> Self {
		Person {
			id: val.id,
			source: val.source,
			name: val.name,
			description: val.description,
			birth_date: val.birth_date,
			thumb_url: val.thumb_url,
			updated_at: val.updated_at,
			created_at: val.created_at,
		}
	}
}


impl NewPersonModel {
	pub async fn insert(self, db: &Database) -> Result<PersonModel> {
		let conn = db.write().await;

		conn.execute(r#"
			INSERT INTO person (source, name, description, birth_date, thumb_url, updated_at, created_at)
			VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
		"#,
		params![
			self.source.to_string(), &self.name, &self.description, &self.birth_date, self.thumb_url.to_optional_string(),
			self.updated_at.timestamp_millis(), self.created_at.timestamp_millis()
		])?;

		Ok(PersonModel {
			id: conn.last_insert_rowid() as usize,
			source: self.source,
			name: self.name,
			description: self.description,
			birth_date: self.birth_date,
			thumb_url: self.thumb_url,
			updated_at: self.updated_at,
			created_at: self.created_at,
		})
	}
}


impl PersonModel {
	pub async fn get_all(offset: usize, limit: usize, db: &Database) -> Result<Vec<Self>> {
		let this = db.read().await;

		let mut conn = this.prepare(r#"SELECT * FROM person LIMIT ?1 OFFSET ?2"#)?;

		let map = conn.query_map([limit, offset], |v| Self::try_from(v))?;

		Ok(map.collect::<std::result::Result<Vec<_>, _>>()?)
	}

	pub async fn get_all_by_book_id(book_id: usize, db: &Database) -> Result<Vec<Self>> {
		let this = db.read().await;

		let mut conn = this.prepare(r#"
			SELECT person.* FROM book_person
			LEFT JOIN
				person ON person.id = book_person.person_id
			WHERE book_id = ?1
		"#)?;

		let map = conn.query_map([book_id], |v| Self::try_from(v))?;

		Ok(map.collect::<std::result::Result<Vec<_>, _>>()?)
	}

	pub async fn search(query: &str, offset: usize, limit: usize, db: &Database) -> Result<Vec<Self>> {
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


		let this = db.read().await;

		let mut conn = this.prepare(&sql)?;

		let map = conn.query_map(params![limit, offset], |v| Self::try_from(v))?;

		Ok(map.collect::<std::result::Result<Vec<_>, _>>()?)
	}

	pub async fn get_by_name(value: &str, db: &Database) -> Result<Option<Self>> {
		let person = db.read().await.query_row(
			r#"SELECT * FROM person WHERE name = ?1 LIMIT 1"#,
			params![value],
			|v| Self::try_from(v)
		).optional()?;

		if let Some(person) = person {
			Ok(Some(person))
		} else if let Some(alt) = PersonAltModel::get_by_name(value, db).await? {
			Self::get_by_id(alt.person_id, db).await
		} else {
			Ok(None)
		}
	}

	pub async fn get_by_id(id: usize, db: &Database) -> Result<Option<Self>> {
		Ok(db.read().await.query_row(
			r#"SELECT * FROM person WHERE id = ?1 LIMIT 1"#,
			params![id],
			|v| Self::try_from(v)
		).optional()?)
	}

	pub async fn get_by_source(value: &str, db: &Database) -> Result<Option<Self>> {
		Ok(db.read().await.query_row(
			r#"SELECT * FROM person WHERE source = ?1 LIMIT 1"#,
			params![value],
			|v| Self::try_from(v)
		).optional()?)
	}

	pub async fn get_count(db: &Database) -> Result<usize> {
		Ok(db.read().await.query_row(r#"SELECT COUNT(*) FROM person"#, [], |v| v.get(0))?)
	}

	pub async fn update(&self, db: &Database) -> Result<()> {
		db.write().await
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
				self.id,
				self.source.to_string(), &self.name, &self.description, &self.birth_date, self.thumb_url.to_string(),
				self.updated_at.timestamp_millis(), self.created_at.timestamp_millis()
			]
		)?;

		Ok(())
	}

	pub async fn remove_by_id(id: usize, db: &Database) -> Result<usize> {
		Ok(db.write().await.execute(
			r#"DELETE FROM person WHERE id = ?1"#,
			params![id]
		)?)
	}
}