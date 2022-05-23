use librarian_common::{TagType, BookTag};
use chrono::{DateTime, TimeZone, Utc};
use rusqlite::Row;

use super::TagModel;


pub struct BookTagModel {
	pub id: usize,

	pub book_id: usize,
	pub tag_id: usize,

	pub index: usize,

	pub created_at: DateTime<Utc>,
}

impl<'a> TryFrom<&Row<'a>> for BookTagModel {
	type Error = rusqlite::Error;

	fn try_from(value: &Row<'a>) -> std::result::Result<Self, Self::Error> {
		Ok(Self {
			id: value.get(0)?,

			book_id: value.get(1)?,
			tag_id: value.get(2)?,

			index: value.get(3)?,

			created_at: Utc.timestamp_millis(value.get(4)?),
		})
	}
}




pub struct BookTagWithTagModel {
	pub id: usize,

	pub book_id: usize,

	pub index: usize,

	pub created_at: DateTime<Utc>,

	pub tag: TagModel,
}

impl<'a> TryFrom<&Row<'a>> for BookTagWithTagModel {
	type Error = rusqlite::Error;

	fn try_from(value: &Row<'a>) -> std::result::Result<Self, Self::Error> {
		Ok(Self {
			id: value.get(0)?,

			book_id: value.get(1)?,

			index: value.get(2)?,

			created_at: Utc.timestamp_millis(value.get(3)?),

			tag: TagModel {
				id: value.get(4)?,
				name: value.get(5)?,
				type_of: TagType::from_u8(value.get(6)?, value.get(7)?),
				created_at: Utc.timestamp_millis(value.get(8)?),
				updated_at: Utc.timestamp_millis(value.get(9)?),
			}
		})
	}
}

impl From<BookTagWithTagModel> for BookTag {
	fn from(val: BookTagWithTagModel) -> Self {
		BookTag {
			id: val.id,
			book_id: val.book_id,
			index: val.index,
			created_at: val.created_at,
			tag: val.tag.into(),
		}
	}
}
