use librarian_common::{ThumbnailStore, util::serialize_datetime};
use chrono::{DateTime, TimeZone, Utc};
use rusqlite::Row;
use serde::Serialize;


#[derive(Serialize)]
pub struct NewPosterModel {
	pub link_id: usize,

	pub path: ThumbnailStore,

	#[serde(serialize_with = "serialize_datetime")]
	pub created_at: DateTime<Utc>,
}


#[derive(Debug, Serialize)]
pub struct PosterModel {
	pub id: usize,

	pub link_id: usize,

	pub path: ThumbnailStore,

	#[serde(serialize_with = "serialize_datetime")]
	pub created_at: DateTime<Utc>,
}

impl<'a> TryFrom<&Row<'a>> for PosterModel {
	type Error = rusqlite::Error;

	fn try_from(value: &Row<'a>) -> std::result::Result<Self, Self::Error> {
		Ok(Self {
			id: value.get(0)?,
			link_id: value.get(1)?,
			path: ThumbnailStore::from(value.get::<_, String>(2)?),
			created_at: Utc.timestamp_millis(value.get(3)?),
		})
	}
}

