use chrono::{DateTime, Utc, TimeZone};
use librarian_common::{EditId, MemberId};
use rusqlite::{Row, params, OptionalExtension};

use crate::{Result, Database};



#[derive(Debug, Clone)]
pub struct EditVoteModel {
	pub edit_id: EditId,
	pub member_id: MemberId,

	pub vote: bool,

	pub created_at: DateTime<Utc>,
}


impl<'a> TryFrom<&Row<'a>> for EditVoteModel {
	type Error = rusqlite::Error;

	fn try_from(value: &Row<'a>) -> std::result::Result<Self, Self::Error> {
		Ok(Self {
			edit_id: value.get(0)?,
			member_id: value.get(1)?,

			vote: value.get(2)?,

			created_at: Utc.timestamp_millis(value.get(3)?),
		})
	}
}


impl EditVoteModel {
	pub fn new(edit_id: EditId, member_id: MemberId, vote: bool) -> Self {
		Self {
			edit_id,
			member_id,
			vote,
			created_at: Utc::now(),
		}
	}


	pub async fn insert(&self, db: &Database) -> Result<()> {
		db.write().await
		.execute(
			"INSERT INTO edit_vote (edit_id, member_id, vote, created_at) VALUES (?1, ?2, ?3, ?4)",
			params![
				self.edit_id,
				self.member_id,
				self.vote,
				self.created_at.timestamp_millis(),
			]
		)?;

		Ok(())
	}

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
			|v| Self::try_from(v)
		).optional()?)
	}

	pub async fn find_by_edit_id(edit_id: EditId, db: &Database) -> Result<Vec<Self>> {
		let this = db.read().await;

		let mut conn = this.prepare("SELECT * FROM edit_vote WHERE edit_id = ?1")?;

		let map = conn.query_map([edit_id], |v| Self::try_from(v))?;

		Ok(map.collect::<std::result::Result<Vec<_>, _>>()?)
	}
}