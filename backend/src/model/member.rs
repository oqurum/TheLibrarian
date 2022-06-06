use lazy_static::lazy_static;
use librarian_common::{MemberId, util::serialize_datetime, Permissions};
use chrono::{DateTime, TimeZone, Utc};
use rusqlite::{Row, params, OptionalExtension};
use serde::Serialize;

use crate::{Database, Result};

lazy_static! {
	pub static ref SYSTEM_MEMBER: MemberModel = MemberModel {
		id: MemberId::from(0),
		name: String::from("System"),
		email: None,
		password: None,
		permissions: Permissions::empty(),
		created_at: Utc.timestamp(0, 0),
		updated_at: Utc.timestamp(0, 0),
	};
}


pub struct NewMemberModel {
	pub name: String,
	pub email: Option<String>,
	pub password: Option<String>,

	pub permissions: Permissions,

	pub created_at: DateTime<Utc>,
	pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MemberModel {
	pub id: MemberId,

	pub name: String,
	pub email: Option<String>,
	pub password: Option<String>,

	pub permissions: Permissions,

	#[serde(serialize_with = "serialize_datetime")]
	pub created_at: DateTime<Utc>,

	#[serde(serialize_with = "serialize_datetime")]
	pub updated_at: DateTime<Utc>,
}

impl<'a> TryFrom<&Row<'a>> for MemberModel {
	type Error = rusqlite::Error;

	fn try_from(value: &Row<'a>) -> std::result::Result<Self, Self::Error> {
		Ok(Self {
			id: value.get(0)?,
			name: value.get(1)?,
			email: value.get(2)?,
			password: value.get(3)?,
			permissions: value.get(4)?,
			created_at: Utc.timestamp_millis(value.get(5)?),
			updated_at: Utc.timestamp_millis(value.get(6)?),
		})
	}
}

impl From<MemberModel> for librarian_common::Member {
	fn from(value: MemberModel) -> librarian_common::Member {
		librarian_common::Member {
			id: value.id,
			name: value.name,
			email: value.email,
			permissions: value.permissions,
			created_at: value.created_at,
			updated_at: value.updated_at,
		}
	}
}



impl NewMemberModel {
	pub async fn insert(self, db: &Database) -> Result<MemberModel> {
		let conn = db.write().await;

		conn.execute(r#"
			INSERT INTO members (name, email, password, permissions, created_at, updated_at)
			VALUES (?1, ?2, ?3, ?4, ?5, ?6)
		"#,
		params![
			&self.name, self.email.as_ref(), self.password.as_ref(), self.permissions,
			self.created_at.timestamp_millis(), self.updated_at.timestamp_millis()
		])?;

		Ok(MemberModel {
			id: MemberId::from(conn.last_insert_rowid() as usize),
			name: self.name,
			email: self.email,
			password: self.password,
			permissions: self.permissions,
			created_at: self.created_at,
			updated_at: self.updated_at,
		})
	}
}

impl MemberModel {
	pub async fn get_by_email(value: &str, db: &Database) -> Result<Option<Self>> {
		Ok(db.read().await.query_row(
			r#"SELECT * FROM members WHERE email = ?1 LIMIT 1"#,
			params![value],
			|v| Self::try_from(v)
		).optional()?)
	}

	pub async fn get_by_id(id: MemberId, db: &Database) -> Result<Option<Self>> {
		Ok(db.read().await.query_row(
			r#"SELECT * FROM members WHERE id = ?1 LIMIT 1"#,
			params![id],
			|v| Self::try_from(v)
		).optional()?)
	}
}