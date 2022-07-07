use chrono::{DateTime, Utc, TimeZone};
use common::MemberId;
use librarian_common::{EditId, item::edit::SharedEditVoteModel, EditVoteId};
use rusqlite::{params, OptionalExtension};

use crate::{Result, Database, model::{TableRow, AdvRow}};



#[derive(Debug, Clone)]
pub struct EditVoteModel {
	pub id: EditVoteId,

	pub edit_id: EditId,
	pub member_id: MemberId,

	pub vote: bool,

	pub created_at: DateTime<Utc>,
}


#[derive(Debug, Clone)]
pub struct NewEditVoteModel {
	pub edit_id: EditId,
	pub member_id: MemberId,

	pub vote: bool,

	pub created_at: DateTime<Utc>,
}


impl TableRow<'_> for EditVoteModel {
	fn create(row: &mut AdvRow<'_>) -> rusqlite::Result<Self> {
		Ok(Self {
			id: row.next()?,
			edit_id: row.next()?,
			member_id: row.next()?,

			vote: row.next()?,

			created_at: Utc.timestamp_millis(row.next()?),
		})
	}
}


impl From<EditVoteModel> for SharedEditVoteModel {
	fn from(val: EditVoteModel) -> Self {
		SharedEditVoteModel {
			id: val.id,
			edit_id: val.edit_id,
			member_id: Some(val.member_id),
			vote: val.vote,
			created_at: val.created_at,
		}
	}
}


impl NewEditVoteModel {
	pub fn create(edit_id: EditId, member_id: MemberId, vote: bool) -> Self {
		Self {
			edit_id,
			member_id,
			vote,
			created_at: Utc::now(),
		}
	}

	pub async fn insert(self, db: &Database) -> Result<EditVoteModel> {
		let this = db.write().await;

		this.execute(
			"INSERT INTO edit_vote (edit_id, member_id, vote, created_at) VALUES (?1, ?2, ?3, ?4)",
			params![
				self.edit_id,
				self.member_id,
				self.vote,
				self.created_at.timestamp_millis(),
			]
		)?;

		Ok(EditVoteModel {
			id: EditVoteId::from(this.last_insert_rowid() as usize),
			edit_id: self.edit_id,
			member_id: self.member_id,
			vote: self.vote,
			created_at: self.created_at,
		})
	}
}

impl EditVoteModel {
	pub async fn update(&self, db: &Database) -> Result<()> {
		Self::update_vote(self.edit_id, self.member_id, self.vote, db).await
	}


	pub async fn update_vote(edit_id: EditId, member_id: MemberId, vote: bool, db: &Database) -> Result<()> {
		db.write().await
		.execute(
			"UPDATE edit_vote SET vote = ?3 WHERE edit_id = ?1 AND member_id = ?2",
			params![
				edit_id,
				member_id,
				vote,
			]
		)?;

		Ok(())
	}

	pub async fn remove(edit_id: EditId, member_id: MemberId, db: &Database) -> Result<usize> {
		Ok(db.write().await.execute(
			r#"DELETE FROM edit_vote WHERE edit_id = ?1 AND member_id = ?2"#,
			params![
				edit_id,
				member_id,
			]
		)?)
	}

	pub async fn find_one(edit_id: EditId, member_id: MemberId, db: &Database) -> Result<Option<Self>> {
		Ok(db.read().await.query_row(
			r#"SELECT * FROM edit_vote WHERE edit_id = ?1 AND member_id = ?2"#,
			params![
				edit_id,
				member_id,
			],
			|v| Self::from_row(v)
		).optional()?)
	}

	pub async fn find_by_edit_id(edit_id: EditId, offset: usize, limit: usize, db: &Database) -> Result<Vec<Self>> {
		let this = db.read().await;

		let mut conn = this.prepare("SELECT * FROM edit_vote WHERE edit_id = ?1 LIMIT ?2 OFFSET ?3")?;

		let map = conn.query_map(params![edit_id, limit, offset], |v| Self::from_row(v))?;

		Ok(map.collect::<std::result::Result<Vec<_>, _>>()?)
	}

	pub async fn count_by_edit_id(edit_id: EditId, db: &Database) -> Result<usize> {
		Ok(db.read().await.query_row("SELECT COUNT(*) FROM edit_vote WHERE edit_id = ?1", [edit_id], |v| v.get(0))?)
	}
}