use std::collections::HashMap;

use chrono::NaiveDate;
use common::{
    api::QueryListResponse, BookId, BookTagId, Either, ImageId, ImageIdType, PersonId, Source,
    TagId,
};
use serde::{Deserialize, Serialize};

use crate::{
    edit::ModifyValuesBy,
    item::edit::{BookEdit, NewOrCachedImage, PersonEdit, SharedEditModel, SharedEditVoteModel},
    util::{deserialize_naivedate_opt, serialize_naivedate_opt},
    BasicDirectory, BasicLibrary, BookTag, Chapter, Collection, CollectionType, DisplayItem,
    DisplayMetaItem, LibraryColl, MediaItem, Member, MetadataItemCached, Person, Poster,
    Progression, SearchType, SharedConfig, TagFE, TagType,
};

#[derive(Serialize, Deserialize, Clone, Copy)]
pub enum OrderBy {
    Asc,
    Desc,
}

impl OrderBy {
    pub fn into_string(self) -> &'static str {
        match self {
            OrderBy::Asc => "ASC",
            OrderBy::Desc => "DESC",
        }
    }

    pub fn from_string(value: &str) -> Option<Self> {
        match value.to_lowercase().as_str() {
            "asc" => Some(Self::Asc),
            "desc" => Some(Self::Desc),
            _ => None,
        }
    }
}

// Searches

// POST /search/{id}
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct PostUpdateSearchIdBody {
    pub update_id: Option<Option<ImageIdType>>,
}

// Collection
pub type GetCollectionListResponse = QueryListResponse<Collection>;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetCollectionResponse {
    pub value: Option<Collection>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NewCollectionBody {
    pub name: String,
    pub description: Option<String>,
    pub type_of: CollectionType,
}

pub type NewCollectionResponse = Collection;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateCollectionModel {
    pub name: Option<String>,
    pub description: Option<Option<String>>,

    pub added_books: Option<Vec<BookId>>,
}

// Edits
// GET /edits
pub type GetEditListResponse = QueryListResponse<SharedEditModel>;

// GET /edit/{id}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetEditResponse {
    pub model: SharedEditModel,
}

// POST /edit/{id}
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct PostEditResponse {
    pub edit_model: Option<SharedEditModel>,
    pub vote: Option<SharedEditVoteModel>,
}

// Tags

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetTagResponse {
    pub value: Option<TagFE>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetTagsResponse {
    pub items: Vec<TagFE>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NewTagBody {
    pub name: String,
    pub type_of: TagType,
}

pub type NewTagResponse = TagFE;

// Book Tags

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetBookTagsResponse {
    pub items: Vec<BookTag>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NewBookTagBody {
    pub tag_id: TagId,
    pub index: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NewBookTagResponse {
    pub id: BookTagId,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetBookTagResponse {
    pub value: Option<BookTag>,
}

// Images

#[derive(Serialize, Deserialize)]
pub struct GetPostersQuery {
    #[serde(default)]
    pub search_metadata: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetPostersResponse {
    pub items: Vec<Poster>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChangePosterBody {
    pub url_or_id: Either<String, ImageId>,
}

// Members

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct GetMemberSelfResponse {
    pub member: Option<Member>,
}

// Libraries

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetLibrariesResponse {
    pub items: Vec<LibraryColl>,
}

// Book

#[derive(Default, Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
pub struct MassEditBooks {
    pub book_ids: Vec<BookId>,

    // People
    pub people_list: Vec<PersonId>,
    pub people_list_mod: ModifyValuesBy,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum NewBookBody {
    FindAndAdd(String),
    Value(Box<Either<Source, BookEdit>>),

    UpdateMultiple(MassEditBooks),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateBookBody {
    pub metadata: Option<DisplayMetaItem>,
    pub people: Option<Vec<Person>>,
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct LoadResourceQuery {
    #[serde(default)]
    pub configure_pages: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetBookIdResponse {
    pub media: MediaItem,
    pub progress: Option<Progression>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetBookListResponse {
    pub count: usize,
    pub items: Vec<DisplayItem>,
}

#[derive(Default, Serialize, Deserialize)]
pub struct BookListQuery {
    pub search: Option<QueryType>,

    pub offset: Option<usize>,
    pub limit: Option<usize>,

    pub order: Option<OrderBy>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "t", content = "v")]
pub enum QueryType {
    Query(String),
    Person(PersonId),

    HasPerson(bool),
}

pub type GetChaptersResponse = QueryListResponse<Chapter>;

// People

pub type GetPeopleResponse = QueryListResponse<Person>;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum PostPersonBody {
    AutoMatchById,

    UpdateBySource(Source),

    CombinePersonWith(PersonId),

    Edit(PersonEdit),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetPeopleSearch {
    pub query: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetPersonResponse {
    pub person: Person,
    pub other_names: Vec<String>,
}

// Options

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetSettingsResponse {
    pub config: SharedConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModifyOptionsBody {
    pub library: Option<BasicLibrary>,
    pub directory: Option<BasicDirectory>,
}

// Metadata

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct MediaViewResponse {
    pub metadata: DisplayMetaItem,
    pub people: Vec<Person>,
    pub tags: Vec<BookTag>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum PostMetadataBody {
    AutoMatchMetaIdBySource,
    AutoMatchMetaIdByFiles,

    UpdateMetaBySource(Source),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetMetadataSearch {
    pub query: String,
    pub search_type: SearchType,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExternalSearchResponse {
    pub items: HashMap<String, Vec<SearchItem>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExternalSourceItemResponse {
    pub item: Option<MetadataBookItem>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum SearchItem {
    Book(MetadataBookSearchItem),
    Person(MetadataPersonSearchItem),
}

impl SearchItem {
    pub fn as_book(&self) -> &MetadataBookSearchItem {
        match self {
            Self::Book(v) => v,
            _ => unreachable!(),
        }
    }

    pub fn as_person(&self) -> &MetadataPersonSearchItem {
        match self {
            Self::Person(v) => v,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MetadataPersonSearchItem {
    pub source: Source,

    pub cover_image: Option<String>,

    pub name: String,
    pub other_names: Option<Vec<String>>,
    pub description: Option<String>,

    pub birth_date: Option<String>,
    pub death_date: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MetadataBookSearchItem {
    pub source: Source,
    pub author: Option<String>,
    pub thumbnail_url: String,
    pub description: Option<String>,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct MetadataBookItem {
    pub source: Source,
    pub title: Option<String>,
    pub description: Option<String>,
    pub rating: f64,

    pub thumbnails: Vec<String>,

    // TODO: Make table for all tags. Include publisher in it. Remove country.
    pub cached: MetadataItemCached,

    pub isbn_10: Option<String>,
    pub isbn_13: Option<String>,

    #[serde(
        serialize_with = "serialize_naivedate_opt",
        deserialize_with = "deserialize_naivedate_opt"
    )]
    pub available_at: Option<NaiveDate>,
    pub language: Option<u16>,
}

impl From<MetadataBookItem> for BookEdit {
    fn from(value: MetadataBookItem) -> Self {
        Self {
            title: value.title,
            description: value.description,
            rating: Some(value.rating).filter(|v| *v != 0.0),
            available_at: value.available_at.map(|v| v.and_hms(0, 0, 0).timestamp()),
            language: value.language,

            added_images: Some(
                value
                    .thumbnails
                    .into_iter()
                    .map(NewOrCachedImage::Url)
                    .collect::<Vec<_>>(),
            )
            .filter(|v| !v.is_empty()),

            ..Self::default()
        }
    }
}

// Task

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RunTaskBody {
    #[serde(default)]
    pub run_search: bool,
    #[serde(default)]
    pub run_metadata: bool,
}

#[derive(Default, Deserialize)]
pub struct SimpleListQuery {
    pub offset: Option<usize>,
    pub limit: Option<usize>,
    pub query: Option<String>,
}

impl SimpleListQuery {
    pub fn limit() -> usize {
        25
    }

    #[cfg(feature = "frontend")]
    pub fn from_url_search_params() -> Self {
        fn create() -> Option<SimpleListQuery> {
            let search_params = web_sys::UrlSearchParams::new_with_str(
                &web_sys::window().unwrap().location().search().ok()?,
            )
            .ok()?;

            let limit = search_params
                .get("limit")
                .and_then(|v| v.parse::<usize>().ok())
                .unwrap_or_else(SimpleListQuery::limit);

            let offset = match search_params
                .get("page")
                .and_then(|v| v.parse::<usize>().ok())
            {
                Some(page) => page * limit,
                None => search_params
                    .get("offset")
                    .and_then(|v| v.parse::<usize>().ok())
                    .unwrap_or_default(),
            };

            Some(SimpleListQuery {
                offset: Some(offset).filter(|v| *v != 0),
                limit: Some(limit),
                query: None,
            })
        }

        create().unwrap_or_default()
    }

    pub fn get_page(&self) -> usize {
        self.offset.unwrap_or_default() / self.limit.unwrap_or_else(Self::limit)
    }

    pub fn set_page(&mut self, value: usize) {
        if value == 0 {
            self.offset = None;
        }

        if let Some(limit) = self.limit {
            self.offset = Some(value * limit);
        } else {
            self.offset = Some(value * Self::limit());
        }
    }

    pub fn to_query(&self) -> String {
        let mut query = format!("page={}", self.get_page());

        if let Some(limit) = self.limit {
            query += "&limit=";
            query += &limit.to_string();
        }

        query
    }
}
