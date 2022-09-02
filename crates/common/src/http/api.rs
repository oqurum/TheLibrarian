use std::collections::HashMap;

use common::{TagId, BookTagId, Either, ImageId, Source, PersonId, api::QueryListResponse, ImageIdType};
use serde::{Serialize, Deserialize};

use crate::{
    MediaItem, Progression, LibraryColl,
    BasicLibrary, BasicDirectory, Chapter,
    DisplayItem, DisplayMetaItem, Person,
    SearchType, Member, Poster,
    Result, TagFE, BookTag, TagType,
    MetadataItemCached,
    item::edit::{SharedEditModel, SharedEditVoteModel, BookEdit, NewOrCachedImage}, SharedConfig,
    des_if_opt_str_not_empty,
};


// Searches

// POST /search/{id}
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct PostUpdateSearchIdBody {
    pub update_id: Option<Option<ImageIdType>>,
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
    pub value: Option<TagFE>
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetPostersResponse {
    pub items: Vec<Poster>
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
    pub items: Vec<LibraryColl>
}



// Book

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum NewBookBody {
    FindAndAdd(String),
    Value(Box<Either<Source, BookEdit>>),
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
    pub progress: Option<Progression>
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetBookListResponse {
    pub count: usize,
    pub items: Vec<DisplayItem>
}

#[derive(Serialize, Deserialize)]
pub struct BookListQuery {
    pub offset: Option<usize>,
    pub limit: Option<usize>,
    /// `SearchQuery`
    #[serde(deserialize_with = "des_if_opt_str_not_empty")]
    pub search: Option<String>,
    pub person_id: Option<PersonId>,
}

impl BookListQuery {
    pub fn new(
        offset: Option<usize>,
        limit: Option<usize>,
        search: Option<SearchQuery>,
        person_id: Option<PersonId>,
    ) -> Result<Self> {
        let search = search.map(serde_urlencoded::to_string)
            .transpose()?;

        Ok(Self {
            offset,
            limit,
            search,
            person_id
        })
    }

    pub fn search_query(&self) -> Option<Result<SearchQuery>> {
        self.search.as_deref().map(|v| Ok(serde_urlencoded::from_str(v)?))
    }
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchQuery {
    #[serde(deserialize_with = "des_if_opt_str_not_empty")]
    #[serde(default)]
    pub query: Option<String>,
    #[serde(deserialize_with = "des_if_opt_str_not_empty")]
    #[serde(default)]
    pub source: Option<String>,
}



pub type GetChaptersResponse = QueryListResponse<Chapter>;



// People

pub type GetPeopleResponse = QueryListResponse<Person>;


#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum PostPersonBody {
    AutoMatchById,

    UpdateBySource(Source),

    CombinePersonWith(PersonId),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetPeopleSearch {
    pub query: Option<String>
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetPersonResponse {
    pub person: Person,
}



// Options

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetSettingsResponse {
    pub config: SharedConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModifyOptionsBody {
    pub library: Option<BasicLibrary>,
    pub directory: Option<BasicDirectory>
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

    UpdateMetaBySource(Source)
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetMetadataSearch {
    pub query: String,
    pub search_type: SearchType,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExternalSearchResponse {
    pub items: HashMap<String, Vec<SearchItem>>
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
            _ => unreachable!()
        }
    }

    pub fn as_person(&self) -> &MetadataPersonSearchItem {
        match self {
            Self::Person(v) => v,
            _ => unreachable!()
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

    pub available_at: Option<String>,
    pub language: Option<u16>
}

impl From<MetadataBookItem> for BookEdit {
    fn from(value: MetadataBookItem) -> Self {
        Self {
            title: value.title,
            description: value.description,
            rating: Some(value.rating).filter(|v|*v != 0.0),
            isbn_10: value.isbn_10,
            isbn_13: value.isbn_13,
            available_at: value.available_at,
            language: value.language,

            added_images: Some(value.thumbnails.into_iter().map(NewOrCachedImage::Url).collect::<Vec<_>>()).filter(|v| !v.is_empty()),

            .. Self::default()
        }
    }
}



// Task

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RunTaskBody {
    #[serde(default)]
    pub run_search: bool,
    #[serde(default)]
    pub run_metadata: bool
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
                &web_sys::window().unwrap().location().search().ok()?
            ).ok()?;

            let limit = search_params.get("limit")
                .and_then(|v| v.parse::<usize>().ok());

            let (offset, limit) = match search_params.get("page").and_then(|v| v.parse::<usize>().ok()) {
                Some(page) => (page * limit.unwrap_or_else(SimpleListQuery::limit), limit),
                None => (search_params.get("offset").and_then(|v| v.parse::<usize>().ok()).unwrap_or_default(), limit)
            };

            Some(SimpleListQuery {
                offset: Some(offset).filter(|v| *v != 0),
                limit,
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