use chrono::{DateTime, Utc};
use common::api::QueryListResponse;
use serde::{Serialize, Deserialize};

use crate::{
	serialize_datetime, deserialize_datetime,
	serialize_datetime_opt, deserialize_datetime_opt,
	MetadataItemCached,
};



// Public Search
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetSearchQuery {
	pub query: String,
	pub offset: Option<usize>,
	pub limit: Option<usize>,
	#[serde(default)]
	pub view_private: bool,
}

pub type BookSearchResponse = QueryListResponse<PublicBook>;




#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct PublicBook {
	pub id: usize,

	pub title: Option<String>,
	pub clean_title: Option<String>,

	pub description: Option<String>,
	pub rating: f64,

	pub thumb_url: String,

	// TODO: Make table for all tags. Include publisher in it. Remove country.
	pub cached: MetadataItemCached,

	pub isbn_10: Option<String>,
	pub isbn_13: Option<String>,

	pub is_public: bool,
	pub edition_count: usize,

	pub available_at: Option<String>,
	pub language: Option<u16>,

	#[serde(serialize_with = "serialize_datetime", deserialize_with = "deserialize_datetime")]
	pub created_at: DateTime<Utc>,
	#[serde(serialize_with = "serialize_datetime", deserialize_with = "deserialize_datetime")]
	pub updated_at: DateTime<Utc>,
	#[serde(serialize_with = "serialize_datetime_opt", deserialize_with = "deserialize_datetime_opt")]
	pub deleted_at: Option<DateTime<Utc>>,
}