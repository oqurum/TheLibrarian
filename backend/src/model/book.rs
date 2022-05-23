use librarian_common::{MetadataItemCached, DisplayMetaItem, ThumbnailStore, util::{serialize_datetime, serialize_datetime_opt}, search::PublicBook};
use chrono::{DateTime, TimeZone, Utc};
use rusqlite::Row;
use serde::Serialize;


#[derive(Debug, Clone, Serialize)]
pub struct BookModel {
	pub id: usize,

	pub title: Option<String>,
	pub clean_title: Option<String>,
	pub description: Option<String>,
	pub rating: f64,

	pub thumb_path: ThumbnailStore,
	/// Not in Database
	pub all_thumb_urls: Vec<String>,

	// TODO: Make table for all tags. Include publisher in it. Remove country.
	pub cached: MetadataItemCached,

	pub isbn_10: Option<String>,
	pub isbn_13: Option<String>,

	pub is_public: bool,
	pub edition_count: usize,

	pub available_at: Option<String>,
	pub language: Option<u16>,

	#[serde(serialize_with = "serialize_datetime")]
	pub created_at: DateTime<Utc>,
	#[serde(serialize_with = "serialize_datetime")]
	pub updated_at: DateTime<Utc>,
	#[serde(serialize_with = "serialize_datetime_opt")]
	pub deleted_at: Option<DateTime<Utc>>,
}


impl<'a> TryFrom<&Row<'a>> for BookModel {
	type Error = rusqlite::Error;

	fn try_from(value: &Row<'a>) -> std::result::Result<Self, Self::Error> {
		Ok(Self {
			id: value.get(0)?,
			title: value.get(1)?,
			clean_title: value.get(2)?,
			description: value.get(3)?,
			rating: value.get(4)?,
			thumb_path: ThumbnailStore::from(value.get::<_, Option<String>>(5)?),
			all_thumb_urls: Vec::new(),
			cached: value.get::<_, Option<String>>(6)?
				.map(|v| MetadataItemCached::from_string(&v))
				.unwrap_or_default(),
			isbn_10: value.get(7)?,
			isbn_13: value.get(8)?,
			is_public: value.get(9)?,
			edition_count: value.get(10)?,
			available_at: value.get(11)?,
			language: value.get(12)?,
			created_at: Utc.timestamp_millis(value.get(13)?),
			updated_at: Utc.timestamp_millis(value.get(14)?),
			deleted_at: value.get::<_, Option<_>>(15)?.map(|v| Utc.timestamp_millis(v)),
		})
	}
}

impl From<BookModel> for DisplayMetaItem {
	fn from(val: BookModel) -> Self {
		DisplayMetaItem {
			id: val.id,
			title: val.title,
			clean_title: val.clean_title,
			description: val.description,
			rating: val.rating,
			thumb_path: val.thumb_path,
			cached: val.cached,
			isbn_10: val.isbn_10,
			isbn_13: val.isbn_13,
			is_public: val.is_public,
			edition_count: val.edition_count,
			available_at: val.available_at,
			language: val.language,
			created_at: val.created_at,
			updated_at: val.updated_at,
			deleted_at: val.deleted_at,
		}
	}
}

impl From<DisplayMetaItem> for BookModel {
	fn from(val: DisplayMetaItem) -> Self {
		BookModel {
			id: val.id,
			title: val.title,
			clean_title: val.clean_title,
			description: val.description,
			rating: val.rating,
			thumb_path: val.thumb_path,
			all_thumb_urls: Vec::new(),
			cached: val.cached,
			isbn_10: val.isbn_10,
			isbn_13: val.isbn_13,
			is_public: val.is_public,
			edition_count: val.edition_count,
			available_at: val.available_at,
			language: val.language,
			created_at: val.created_at,
			updated_at: val.updated_at,
			deleted_at: val.deleted_at,
		}
	}
}

#[allow(clippy::from_over_into)]
impl Into<PublicBook> for BookModel {
	fn into(self) -> PublicBook {
		PublicBook {
			id: self.id,
			title: self.title,
			clean_title: self.clean_title,
			description: self.description,
			rating: self.rating,
			// We create the thumb_url in the actix request.
			thumb_url: String::new(),
			cached: self.cached,
			isbn_10: self.isbn_10,
			isbn_13: self.isbn_13,
			is_public: self.is_public,
			edition_count: self.edition_count,
			available_at: self.available_at,
			language: self.language,
			created_at: self.created_at,
			updated_at: self.updated_at,
			deleted_at: self.deleted_at,
		}
	}
}

