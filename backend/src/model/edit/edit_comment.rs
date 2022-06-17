use chrono::{DateTime, Utc, TimeZone};
use librarian_common::{EditId, MemberId, EditCommentId};
use rusqlite::params;

use crate::{Database, Result, model::{TableRow, AdvRow}};



pub struct NewEditCommentModel {
	pub edit_id: EditId,
	pub member_id: MemberId,

	pub text: String,
	pub deleted: bool,

	pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct EditCommentModel {
	pub id: EditCommentId,

	pub edit_id: EditId,
	pub member_id: MemberId,

	pub text: String,
	pub deleted: bool,

	pub created_at: DateTime<Utc>,
}


impl TableRow<'_> for EditCommentModel {
	fn create(row: &mut AdvRow<'_>) -> rusqlite::Result<Self> {
		Ok(Self {
			id: row.next()?,

			edit_id: row.next()?,
			member_id: row.next()?,

			text: row.next()?,
			deleted: row.next()?,

			created_at: Utc.timestamp_millis(row.next()?),
		})
	}
}


impl NewEditCommentModel {
	pub fn new(edit_id: EditId, member_id: MemberId, text: String) -> Self {
		Self {
			edit_id,
			member_id,
			text,
			deleted: false,
			created_at: Utc::now(),
		}
	}

	pub async fn insert(self, db: &Database) -> Result<EditCommentModel> {
		let lock = db.write().await;

		lock.execute(r#"
			INSERT INTO edit_comment (
				edit_id, member_id,
				text, deleted,
				created_at
			)
			VALUES (?1, ?2, ?3, ?4, ?5)"#,
			params![
				self.edit_id, self.member_id,
				self.text, self.deleted,
				self.created_at.timestamp_millis(),
			]
		)?;

		Ok(EditCommentModel {
			id: EditCommentId::from(lock.last_insert_rowid() as usize),

			edit_id: self.edit_id,
			member_id: self.member_id,

			text: self.text,
			deleted: self.deleted,

			created_at: self.created_at,
		})
	}
}


impl EditCommentModel {
	pub async fn get_by_edit_id(
		edit_id: EditId,
		offset: usize,
		limit: usize,
		deleted: Option<bool>,
		db: &Database
	) -> Result<Vec<Self>> {
		let this = db.read().await;

		if let Some(deleted) = deleted {
			let mut conn = this.prepare(r#"SELECT * FROM edit_comment WHERE edit_id = ?1 AND deleted = ?2 LIMIT ?3 OFFSET ?4"#)?;

			let map = conn.query_map(params![ edit_id, deleted, limit, offset ], |v| Self::from_row(v))?;

			Ok(map.collect::<std::result::Result<Vec<_>, _>>()?)
		} else {
			let mut conn = this.prepare(r#"SELECT * FROM edit_comment WHERE edit_id = ?1 LIMIT ?2 OFFSET ?3"#)?;

			let map = conn.query_map(params![ edit_id, limit, offset ], |v| Self::from_row(v))?;

			Ok(map.collect::<std::result::Result<Vec<_>, _>>()?)
		}
	}

	pub async fn get_count(edit_id: EditId, db: &Database) -> Result<usize> {
		Ok(db.read().await.query_row(r#"SELECT COUNT(*) FROM edit_comment WHERE edit_id = ?1"#, [edit_id], |v| v.get(0))?)
	}
}