use librarian_common::{TagType, TagFE};
use chrono::{DateTime, TimeZone, Utc};
use rusqlite::Row;


pub struct TagModel {
	pub id: usize,

	pub name: String,
	pub type_of: TagType,

	pub created_at: DateTime<Utc>,
	pub updated_at: DateTime<Utc>,
}


impl<'a> TryFrom<&Row<'a>> for TagModel {
	type Error = rusqlite::Error;

	fn try_from(value: &Row<'a>) -> std::result::Result<Self, Self::Error> {
		Ok(Self {
			id: value.get(0)?,
			name: value.get(1)?,
			type_of: TagType::from_u8(value.get(2)?, value.get(3)?),
			created_at: Utc.timestamp_millis(value.get(4)?),
			updated_at: Utc.timestamp_millis(value.get(5)?),
		})
	}
}

impl From<TagModel> for TagFE {
	fn from(val: TagModel) -> Self {
		TagFE {
			id: val.id,
			name: val.name,
			type_of: val.type_of,
			created_at: val.created_at,
			updated_at: val.updated_at
		}
	}
}