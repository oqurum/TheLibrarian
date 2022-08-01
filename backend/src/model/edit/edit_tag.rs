use chrono::{DateTime, Utc, TimeZone};
use librarian_common::{EditId, MemberId};
use rusqlite::Row;



#[derive(Debug, Clone)]
pub struct EditTagModel {
	pub edit_id: EditId,
	pub member_id: MemberId,

	pub vote: bool,

	pub created_at: DateTime<Utc>,
}


impl TableRow<'_> for EditModel {
	fn create(row: &mut AdvRow<'_>) -> rusqlite::Result<Self> {
		Ok(Self {
			edit_id: row.next()?,
			member_id: row.next()?,

			vote: row.next()?,

			created_at: Utc.timestamp_millis(row.next()?),
		})
	}
}



// type TagEdit struct {
// 	EditID         uuid.UUID  `json:"-"`
// 	Name           *string    `json:"name,omitempty"`
// 	Description    *string    `json:"description,omitempty"`
// 	AddedAliases   []string   `json:"added_aliases,omitempty"`
// 	RemovedAliases []string   `json:"removed_aliases,omitempty"`
// 	CategoryID     *uuid.UUID `json:"category_id,omitempty"`
// }