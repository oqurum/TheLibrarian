use rusqlite::Row;

use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct TagPersonAltModel {
	pub person_id: usize,
	pub name: String,
}

impl<'a> TryFrom<&Row<'a>> for TagPersonAltModel {
	type Error = rusqlite::Error;

	fn try_from(value: &Row<'a>) -> std::result::Result<Self, Self::Error> {
		Ok(Self {
			person_id: value.get(0)?,
			name: value.get(1)?,
		})
	}
}
