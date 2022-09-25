#![warn(unused)]


use std::path::PathBuf;

use chrono::{DateTime, Utc, Date, NaiveDate};
use common::{TagId, BookTagId, BookId, MemberId, PersonId, Source, ThumbnailStore, ImageId, ImageIdType};
use item::{edit::BookEdit, member::MemberSettings};
use num_enum::{FromPrimitive, IntoPrimitive};
use serde::{Serialize, Deserialize};

use util::*;

mod http;
pub mod util;
pub mod error;
pub mod specific;
pub mod item;

pub use http::*;
pub use specific::*;
pub use error::{Result, Error};



// Collection


#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Collection {
    pub id: CollectionId,

    pub name: String,
    pub description: Option<String>,
    pub type_of: CollectionType,

    #[serde(serialize_with = "serialize_datetime", deserialize_with = "deserialize_datetime")]
    pub created_at: DateTime<Utc>,
    #[serde(serialize_with = "serialize_datetime", deserialize_with = "deserialize_datetime")]
    pub updated_at: DateTime<Utc>,
}



#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, FromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum CollectionType {
    #[num_enum(default)]
    List,
    Series,
}



// Searches

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchGroup {
    pub id: SearchGroupId,

    pub query: String,
    pub calls: usize,
    pub last_found_amount: usize,
    #[serde(serialize_with = "serialize_date", deserialize_with = "deserialize_date")]
    pub timeframe: Date<Utc>,
    pub found_id: Option<ImageIdType>,

    #[serde(serialize_with = "serialize_datetime", deserialize_with = "deserialize_datetime")]
    pub created_at: DateTime<Utc>,
    #[serde(serialize_with = "serialize_datetime", deserialize_with = "deserialize_datetime")]
    pub updated_at: DateTime<Utc>,
}




// Tags

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct TagFE {
    pub id: TagId,

    pub name: String,
    pub type_of: TagType,

    #[serde(serialize_with = "serialize_datetime", deserialize_with = "deserialize_datetime")]
    pub created_at: DateTime<Utc>,
    #[serde(serialize_with = "serialize_datetime", deserialize_with = "deserialize_datetime")]
    pub updated_at: DateTime<Utc>,
}


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TagType {
    /// Will have data
    Collection,
    /// No Data
    Genre,
    /// No Data
    Subject,
    /// No Data
    Language,
    /// No Data
    LocationFiction,
    /// No Data
    LocationNonFiction,
    /// No Data
    // Timeframe,
    /// No Data
    PersonFiction,
    /// No Data
    PersonNonFiction,
    // Pace,
    // Difficulty,
    // Mood,
    // Length,
    // Purpose,
    /// No Data
    Award,
}

impl TagType {
    pub fn into_u8(&self) -> u8 {
        match self {
            Self::Collection => 0,
            Self::Genre => 1,
            Self::Subject => 2,
            Self::Language => 3,
            Self::LocationFiction => 4,
            Self::LocationNonFiction => 5,
            // Self::Timeframe => 6,
            Self::PersonFiction => 7,
            Self::PersonNonFiction => 8,
            // Self::Pace => 9,
            // Self::Difficulty => 10,
            // Self::Mood => 11,
            // Self::Length => 12,
            // Self::Purpose => 13,
            Self::Award => 14,
        }
    }

    pub fn from_u8(value: u8, _data: Option<String>) -> Self {
        match value {
            0 => Self::Collection,
            1 => Self::Genre,
            2 => Self::Subject,
            3 => Self::Language,
            4 => Self::LocationFiction,
            5 => Self::LocationNonFiction,
            // 6 => Self::Timeframe,
            7 => Self::PersonFiction,
            8 => Self::PersonNonFiction,
            // 9 => Self::Pace,
            // 10 => Self::Difficulty,
            // 11 => Self::Mood,
            // 12 => Self::Length,
            // 13 => Self::Purpose,
            14 => Self::Award,

            _ => unreachable!(),
        }
    }

    /// Split up the u8 value and data stored.
    pub fn split(self) -> (u8, Option<String>) {
        (self.into_u8(), None) // TODO
    }
}


#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct BookTag {
    pub id: BookTagId,

    pub book_id: BookId,

    pub index: usize,

    #[serde(serialize_with = "serialize_datetime", deserialize_with = "deserialize_datetime")]
    pub created_at: DateTime<Utc>,

    pub tag: TagFE,
}





// Member

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Member {
    pub id: MemberId,

    pub name: String,
    pub email: Option<String>,

    pub permissions: Permissions,

    pub localsettings: MemberSettings,

    #[serde(serialize_with = "serialize_datetime", deserialize_with = "deserialize_datetime")]
    pub created_at: DateTime<Utc>,
    #[serde(serialize_with = "serialize_datetime", deserialize_with = "deserialize_datetime")]
    pub updated_at: DateTime<Utc>,
}



// Used for People View

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
pub struct Person {
    pub id: PersonId,

    pub source: Source,

    pub name: String,
    pub description: Option<String>,
    #[serde(serialize_with = "serialize_naivedate_opt", deserialize_with = "deserialize_naivedate_opt")]
    pub birth_date: Option<NaiveDate>,

    pub thumb_url: ThumbnailStore,

    pub info: Option<String>,

    #[serde(serialize_with = "serialize_datetime", deserialize_with = "deserialize_datetime")]
    pub updated_at: DateTime<Utc>,
    #[serde(serialize_with = "serialize_datetime", deserialize_with = "deserialize_datetime")]
    pub created_at: DateTime<Utc>,
}

impl Person {
    pub fn get_thumb_url(&self) -> String {
        if self.thumb_url != ThumbnailStore::None {
            format!("/api/v1/person/{}/thumbnail", self.id)
        } else {
            String::from("/images/missingperson.jpg")
        }
    }
}


// Used for Library View

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DisplayItem {
    pub id: BookId,

    pub title: String,
    pub cached: MetadataItemCached,
    pub has_thumbnail: bool,
}

impl DisplayItem {
    pub fn get_thumb_url(&self) -> String {
        if self.has_thumbnail {
            format!("/api/v1/book/{}/thumbnail", self.id)
        } else {
            String::from("/images/missingthumbnail.jpg")
        }
    }
}

impl PartialEq for DisplayItem {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl From<DisplayMetaItem> for DisplayItem {
    fn from(val: DisplayMetaItem) -> Self {
        DisplayItem {
            id: val.id,
            title: val.title.or(val.clean_title).unwrap_or_default(),
            cached: val.cached,
            has_thumbnail: val.thumb_path.is_some()
        }
    }
}


// Used for Media View

/// Clone of Model.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct DisplayMetaItem {
    pub id: BookId,

    pub title: Option<String>,
    pub clean_title: Option<String>,
    pub description: Option<String>,
    pub rating: f64,

    pub thumb_path: ThumbnailStore,

    // TODO: Make table for all tags. Include publisher in it. Remove country.
    pub cached: MetadataItemCached,

    pub isbn_10: Option<String>,
    pub isbn_13: Option<String>,

    pub is_public: bool,
    pub edition_count: usize,

    #[serde(serialize_with = "serialize_naivedate_opt", deserialize_with = "deserialize_naivedate_opt")]
    pub available_at: Option<NaiveDate>,
    pub language: u16,

    #[serde(serialize_with = "serialize_datetime", deserialize_with = "deserialize_datetime")]
    pub created_at: DateTime<Utc>,
    #[serde(serialize_with = "serialize_datetime", deserialize_with = "deserialize_datetime")]
    pub updated_at: DateTime<Utc>,
    #[serde(serialize_with = "serialize_datetime_opt", deserialize_with = "deserialize_datetime_opt")]
    pub deleted_at: Option<DateTime<Utc>>,
}

impl DisplayMetaItem {
    pub fn get_thumb_url(&self) -> String {
        if let ThumbnailStore::Path(path) = &self.thumb_path {
            format!("/api/v1/image/{path}")
        } else {
            String::from("/images/missingthumbnail.jpg")
        }
    }

    pub fn get_title(&self) -> String {
        self.title.as_ref().or(self.clean_title.as_ref()).cloned().unwrap_or_else(|| String::from("No Title"))
    }
}

impl Default for DisplayMetaItem {
    fn default() -> Self {
        Self {
            id: Default::default(),
            title: Default::default(),
            clean_title: Default::default(),
            description: Default::default(),
            rating: Default::default(),
            thumb_path: ThumbnailStore::None,
            cached: Default::default(),
            isbn_10: Default::default(),
            isbn_13: Default::default(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: Default::default(),
            available_at: Default::default(),
            language: Default::default(),
            is_public: Default::default(),
            edition_count: Default::default(),
        }
    }
}

impl From<DisplayMetaItem> for BookEdit {
    fn from(value: DisplayMetaItem) -> Self {
        Self {
            title: value.title,
            description: value.description,
            rating: Some(value.rating),
            isbn_10: value.isbn_10,
            isbn_13: value.isbn_13,
            available_at: value.available_at.map(|v| v.and_hms(0, 0, 0).timestamp()),
            language: Some(value.language),

            .. Self::default()
        }
    }
}


// Used for Reader

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MediaItem {
    pub id: BookId,

    pub path: String,

    pub file_name: String,
    pub file_type: String,
    pub file_size: i64,

    pub library_id: usize,
    pub metadata_id: Option<usize>,
    pub chapter_count: usize,

    pub identifier: Option<String>,

    pub modified_at: i64,
    pub accessed_at: i64,
    pub created_at: i64,
}

impl PartialEq for MediaItem {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}


#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum Progression {
    Ebook {
        chapter: i64,
        char_pos: i64,
        page: i64,
    },

    AudioBook {
        chapter: i64,
        seek_pos: i64,
    },

    Complete
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Chapter {
    pub file_path: PathBuf,
    pub value: usize,
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LibraryColl {
    pub id: usize,
    pub name: String,

    pub scanned_at: i64,
    pub created_at: i64,
    pub updated_at: i64,

    pub directories: Vec<String>
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BasicLibrary {
    pub id: Option<usize>,
    pub name: Option<String>
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BasicDirectory {
    pub library_id: usize,
    pub path: String
}



#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MetadataItemCached {
    pub author_id: Option<PersonId>,
    pub author: Option<String>,
    pub publisher: Option<String>,
}

impl MetadataItemCached {
    pub fn as_string(&self) -> String {
        serde_urlencoded::to_string(&self).unwrap()
    }

    /// Returns `None` if string is empty.
    pub fn as_string_optional(&self) -> Option<String> {
        Some(self.as_string()).filter(|v| !v.is_empty())
    }

    pub fn from_string<V: AsRef<str>>(value: V) -> Self {
        serde_urlencoded::from_str(value.as_ref()).unwrap()
    }

    pub fn overwrite_with(&mut self, value: Self) {
        if value.author.is_some() {
            self.author = value.author;
        }

        if value.publisher.is_some() {
            self.publisher = value.publisher;
        }
    }

    pub fn author(mut self, value: String) -> Self {
        self.author = Some(value);
        self
    }

    pub fn author_id(mut self, value: PersonId) -> Self {
        self.author_id = Some(value);
        self
    }

    pub fn publisher(mut self, value: String) -> Self {
        self.publisher = Some(value);
        self
    }

    pub fn author_optional(mut self, value: Option<String>) -> Self {
        if value.is_some() {
            self.author = value;
        }

        self
    }

    pub fn publisher_optional(mut self, value: Option<String>) -> Self {
        if value.is_some() {
            self.publisher = value;
        }

        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SearchType {
    Book,
    Person
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SearchFor {
    Book(SearchForBooksBy),
    Person,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SearchForBooksBy {
    Query,
    Title,
    AuthorName,
    Contents,
}



// Image

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Poster {
    pub id: Option<ImageId>,

    pub selected: bool,

    pub path: String,

    #[serde(serialize_with = "serialize_datetime", deserialize_with = "deserialize_datetime")]
    pub created_at: DateTime<Utc>,
}