use std::collections::HashMap;

use serde::{Serialize, Deserialize};

use crate::{Either, MediaItem, Progression, LibraryColl, BasicLibrary, BasicDirectory, Chapter, DisplayItem, DisplayMetaItem, Person, SearchType, Source, Member, Poster, Result, TagFE, BookTag, TagType};

// General

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeletionResponse {
	pub amount: usize,
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

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct NewTagResponse {
	pub id: usize,
}



// Book Tags

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetBookTagsResponse {
	pub items: Vec<BookTag>,
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NewBookTagBody {
	pub tag_id: usize,
	pub index: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NewBookTagResponse {
	pub id: usize,
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
	pub url_or_id: Either<String, usize>,
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
pub struct NewBookBody {
	pub source: Source,
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
	pub search: Option<String>,
	pub person_id: Option<usize>,
}

impl BookListQuery {
	pub fn new(
		offset: Option<usize>,
		limit: Option<usize>,
		search: Option<SearchQuery>,
		person_id: Option<usize>,
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
	pub query: Option<String>,
	pub source: Option<String>,
}



pub type GetChaptersResponse = QueryListResponse<Chapter>;



// People

pub type GetPeopleResponse = QueryListResponse<Person>;


#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum PostPersonBody {
	AutoMatchById,

	UpdateBySource(Source),

	CombinePersonWith(usize),
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
	//
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

// Task

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RunTaskBody {
	#[serde(default)]
	pub run_search: bool,
	#[serde(default)]
	pub run_metadata: bool
}



#[derive(Deserialize)]
pub struct SimpleListQuery {
	pub offset: Option<usize>,
	pub limit: Option<usize>,
	pub query: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct QueryListResponse<V> {
	pub offset: usize,
	pub limit: usize,
	pub total: usize,
	pub items: Vec<V>
}