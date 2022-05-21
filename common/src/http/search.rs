use serde::{Serialize, Deserialize};

use crate::{api::QueryListResponse, DisplayMetaItem};



// Public Search
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetSearchQuery {
	pub query: String,
	pub offset: Option<usize>,
	pub limit: Option<usize>,
}

pub type BookSearchResponse = QueryListResponse<DisplayMetaItem>;