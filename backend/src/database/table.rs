use librarian_common::{MetadataItemCached, DisplayMetaItem, Person, Source, ThumbnailStore, TagType, TagFE, BookTag};
use chrono::{DateTime, TimeZone, Utc};
use rusqlite::Row;
use serde::{Serialize, Serializer};


// Metadata

// TODO: Place into common
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

	pub tags_author: Option<String>,
	pub tags_country: Option<String>,

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
			tags_author: value.get(9)?,
			tags_country: value.get(10)?,
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
			tags_author: val.tags_author,
			tags_country: val.tags_country,
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
            tags_author: val.tags_author,
            tags_country: val.tags_country,
            available_at: val.available_at,
            language: val.language,
            created_at: val.created_at,
            updated_at: val.updated_at,
            deleted_at: val.deleted_at,
        }
    }
}


// Tag Person Alt

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


// People

#[derive(Debug)]
pub struct NewPersonModel {
	pub source: Source,

	pub name: String,
	pub description: Option<String>,
	pub birth_date: Option<String>,

	pub thumb_url: ThumbnailStore,

	pub updated_at: DateTime<Utc>,
	pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct TagPersonModel {
	pub id: usize,

	pub source: Source,

	pub name: String,
	pub description: Option<String>,
	pub birth_date: Option<String>,

	pub thumb_url: ThumbnailStore,

	#[serde(serialize_with = "serialize_datetime")]
	pub updated_at: DateTime<Utc>,
	#[serde(serialize_with = "serialize_datetime")]
	pub created_at: DateTime<Utc>,
}

impl<'a> TryFrom<&Row<'a>> for TagPersonModel {
	type Error = rusqlite::Error;

	fn try_from(value: &Row<'a>) -> std::result::Result<Self, Self::Error> {
		Ok(Self {
			id: value.get(0)?,

			source: Source::try_from(value.get::<_, String>(1)?).unwrap(),

			name: value.get(2)?,
			description: value.get(3)?,
			birth_date: value.get(4)?,

			thumb_url: ThumbnailStore::from(value.get::<_, Option<String>>(5)?),

			created_at: Utc.timestamp_millis(value.get(6)?),
			updated_at: Utc.timestamp_millis(value.get(7)?),
		})
	}
}

impl From<TagPersonModel> for Person {
	fn from(val: TagPersonModel) -> Self {
		Person {
			id: val.id,
			source: val.source,
			name: val.name,
			description: val.description,
			birth_date: val.birth_date,
			thumb_url: val.thumb_url,
			updated_at: val.updated_at,
			created_at: val.created_at,
		}
	}
}


// Tag Person Alt

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


// Cached Images

#[derive(Debug, Serialize)]
pub struct CachedImageModel {
	pub item_id: usize,

	pub type_of: CacheType, // TODO: Enum

	pub path: ThumbnailStore,

	#[serde(serialize_with = "serialize_datetime")]
	pub created_at: DateTime<Utc>,
}

impl<'a> TryFrom<&Row<'a>> for CachedImageModel {
	type Error = rusqlite::Error;

	fn try_from(value: &Row<'a>) -> std::result::Result<Self, Self::Error> {
		Ok(Self {
			item_id: value.get(0)?,
			type_of: CacheType::from(value.get::<_, u8>(1)?),
			path: ThumbnailStore::from(value.get::<_, String>(2)?),
			created_at: Utc.timestamp_millis(value.get(3)?),
		})
	}
}


#[derive(Debug, Clone, Copy, Serialize)]
pub enum CacheType {
	BookPoster = 0,
	BookBackground,

	PersonPoster,
}

impl CacheType {
	pub fn into_num(self) -> u8 {
		self as u8
	}
}

impl From<u8> for CacheType {
    fn from(value: u8) -> Self {
        match value {
			0 => Self::BookPoster,
			1 => Self::BookBackground,
			2 => Self::PersonPoster,

			_ => unimplemented!()
		}
    }
}


// User

// TODO: type_of 0 = web page, 1 = local passwordless 2 = local password
// TODO: Enum.
pub struct NewMemberModel {
	pub name: String,
	pub email: Option<String>,
	pub password: Option<String>,

	pub type_of: u8,

	// TODO
	pub config: Option<String>,

	pub created_at: DateTime<Utc>,
	pub updated_at: DateTime<Utc>,
}

impl NewMemberModel {
	pub fn into_member(self, id: usize) -> MemberModel {
		MemberModel {
			id,
			name: self.name,
			email: self.email,
			password: self.password,
			type_of: self.type_of,
			config: self.config,
			created_at: self.created_at,
			updated_at: self.updated_at,
		}
	}
}

#[derive(Debug, Clone, Serialize)]
pub struct MemberModel {
	pub id: usize,

	pub name: String,
	pub email: Option<String>,
	pub password: Option<String>,

	pub type_of: u8,

	// TODO
	pub config: Option<String>,

	#[serde(serialize_with = "serialize_datetime")]
	pub created_at: DateTime<Utc>,

	#[serde(serialize_with = "serialize_datetime")]
	pub updated_at: DateTime<Utc>,
}

impl<'a> TryFrom<&Row<'a>> for MemberModel {
	type Error = rusqlite::Error;

	fn try_from(value: &Row<'a>) -> std::result::Result<Self, Self::Error> {
		Ok(Self {
			id: value.get(0)?,
			name: value.get(1)?,
			email: value.get(2)?,
			password: value.get(3)?,
			type_of: value.get(4)?,
			config: value.get(5)?,
			created_at: Utc.timestamp_millis(value.get(6)?),
			updated_at: Utc.timestamp_millis(value.get(7)?),
		})
	}
}

impl From<MemberModel> for librarian_common::Member {
	fn from(value: MemberModel) -> librarian_common::Member {
		librarian_common::Member {
			id: value.id,
			name: value.name,
			email: value.email,
			type_of: value.type_of,
			config: value.config,
			created_at: value.created_at,
			updated_at: value.updated_at,
		}
	}
}


// Auth

pub struct NewAuthModel {
	pub oauth_token: String,
	pub oauth_token_secret: String,
	pub created_at: DateTime<Utc>,
}


// Poster

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


// Tags

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


// Book Tags
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


pub struct BookTagInfo {
	pub id: usize,

	pub book_id: usize,

	pub index: usize,

	pub created_at: DateTime<Utc>,

	pub tag: TagModel,
}

impl<'a> TryFrom<&Row<'a>> for BookTagInfo {
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

impl From<BookTagInfo> for BookTag {
	fn from(val: BookTagInfo) -> Self {
		BookTag {
			id: val.id,
			book_id: val.book_id,
			index: val.index,
			created_at: val.created_at,
			tag: val.tag.into(),
		}
	}
}


// Utils

fn serialize_datetime<S>(value: &DateTime<Utc>, s: S) -> std::result::Result<S::Ok, S::Error> where S: Serializer {
	s.serialize_i64(value.timestamp_millis())
}

fn serialize_datetime_opt<S>(value: &Option<DateTime<Utc>>, s: S) -> std::result::Result<S::Ok, S::Error> where S: Serializer {
	match value {
		Some(v) => s.serialize_i64(v.timestamp_millis()),
		None => s.serialize_none()
	}
}