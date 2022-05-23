use rusqlite::Row;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct BookPersonModel {
	pub book_id: usize,
	pub person_id: usize,
}

impl<'a> TryFrom<&Row<'a>> for BookPersonModel {
	type Error = rusqlite::Error;

	fn try_from(value: &Row<'a>) -> std::result::Result<Self, Self::Error> {
		Ok(Self {
			book_id: value.get(0)?,
			person_id: value.get(1)?,
		})
	}
}