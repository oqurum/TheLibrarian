use chrono::{DateTime, Utc, TimeZone};
use common::MemberId;
use librarian_common::EditId;

use crate::model::{TableRow, AdvRow};



#[derive(Debug, Clone)]
pub struct EditPersonModel {
	pub edit_id: EditId,
	pub member_id: MemberId,

	pub vote: bool,

	pub created_at: DateTime<Utc>,
}


impl TableRow<'_> for EditPersonModel {
	fn create(row: &mut AdvRow<'_>) -> rusqlite::Result<Self> {
		Ok(Self {
			edit_id: row.next()?,
			member_id: row.next()?,

			vote: row.next()?,

			created_at: Utc.timestamp_millis(row.next()?),
		})
	}
}
