use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

use crate::{EditId, edit::*, util::*, TagId, PersonId, BookId, DisplayMetaItem, Member, MemberId, api::QueryListResponse, EditVoteId, ThumbnailStore};






#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedEditModel {
	pub id: EditId,

	pub type_of: EditType,
	pub operation: EditOperation,
	pub status: EditStatus,

	pub member: Option<Member>,

	pub model_id: Option<usize>,

	pub is_applied: bool,
	pub vote_count: usize,

	pub votes: Option<QueryListResponse<SharedEditVoteModel>>,

	pub data: EditData,

	#[serde(serialize_with = "serialize_datetime_opt", deserialize_with = "deserialize_datetime_opt")]
	pub ended_at: Option<DateTime<Utc>>,
	#[serde(serialize_with = "serialize_datetime_opt", deserialize_with = "deserialize_datetime_opt")]
	pub expires_at: Option<DateTime<Utc>>,

	#[serde(serialize_with = "serialize_datetime", deserialize_with = "deserialize_datetime")]
	pub created_at: DateTime<Utc>,
	#[serde(serialize_with = "serialize_datetime", deserialize_with = "deserialize_datetime")]
	pub updated_at: DateTime<Utc>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedEditVoteModel {
	pub id: EditVoteId,

	pub edit_id: EditId,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub member_id: Option<MemberId>,

	pub vote: bool,

	#[serde(serialize_with = "serialize_datetime", deserialize_with = "deserialize_datetime")]
	pub created_at: DateTime<Utc>,
}



#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdateEditModel {
	pub status: Option<EditStatus>,

	pub is_applied: Option<bool>,

	pub vote: Option<i32>,

	#[serde(serialize_with = "serialize_datetime_opt_opt", deserialize_with = "deserialize_datetime_opt_opt")]
	pub ended_at: Option<Option<DateTime<Utc>>>,
	#[serde(serialize_with = "serialize_datetime_opt_opt", deserialize_with = "deserialize_datetime_opt_opt")]
	pub expires_at: Option<Option<DateTime<Utc>>>,
}



// EditModel data field

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EditData {
	Book(BookEditData),
	Person,
	Tag,
	Collection,
}



#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookEditData {
	#[serde(skip_serializing_if = "Option::is_none")]
	pub current: Option<DisplayMetaItem>,

	#[serde(skip_serializing_if = "Option::is_none")]
	pub new: Option<BookEdit>,

	#[serde(skip_serializing_if = "Option::is_none")]
	pub old: Option<BookEdit>, // Based off of current Model. If field is different than current Model once it's updating it'll ignore it.

	#[serde(skip_serializing_if = "Option::is_none")]
	pub updated: Option<UpdatedBookEdit>,
}


#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdatedBookEdit {
	#[serde(skip_serializing_if = "is_false")]
	pub title: bool,
	#[serde(skip_serializing_if = "is_false")]
	pub clean_title: bool,
	#[serde(skip_serializing_if = "is_false")]
	pub description: bool,

	#[serde(skip_serializing_if = "is_false")]
	pub rating: bool,

	#[serde(skip_serializing_if = "is_false")]
	pub isbn_10: bool,
	#[serde(skip_serializing_if = "is_false")]
	pub isbn_13: bool,

	#[serde(skip_serializing_if = "is_false")]
	pub is_public: bool,

	#[serde(skip_serializing_if = "is_false")]
	pub available_at: bool,
	#[serde(skip_serializing_if = "is_false")]
	pub language: bool,

	#[serde(skip_serializing_if = "is_false")]
	pub publisher: bool,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookEdit {
	#[serde(skip_serializing_if = "Option::is_none")]
	pub title: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub clean_title: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub description: Option<String>,

	#[serde(skip_serializing_if = "Option::is_none")]
	pub rating: Option<f64>,

	#[serde(skip_serializing_if = "Option::is_none")]
	pub isbn_10: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub isbn_13: Option<String>,

	#[serde(skip_serializing_if = "Option::is_none")]
	pub is_public: Option<bool>,

	#[serde(skip_serializing_if = "Option::is_none")]
	pub available_at: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub language: Option<u16>,

	#[serde(skip_serializing_if = "Option::is_none")]
	pub publisher: Option<String>,

	#[serde(skip_serializing_if = "Option::is_none")]
	pub added_people: Option<Vec<PersonId>>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub removed_people: Option<Vec<PersonId>>,

	#[serde(skip_serializing_if = "Option::is_none")]
	pub added_tags: Option<Vec<TagId>>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub removed_tags: Option<Vec<TagId>>,

	#[serde(skip_serializing_if = "Option::is_none")]
	pub added_images: Option<Vec<ThumbnailStore>>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub removed_images: Option<Vec<ThumbnailStore>>,
}

#[derive(Debug, Clone, Copy)]
pub enum ModelIdGroup {
	Book(BookId),
	Person(PersonId),
	Tag(TagId),
}


impl SharedEditModel {
	pub fn get_model_id(&self) -> Option<ModelIdGroup> {
		self.model_id.map(|id| match self.type_of {
			EditType::Book => ModelIdGroup::Book(BookId::from(id)),
			EditType::Person => ModelIdGroup::Person(PersonId::from(id)),
			EditType::Tag => ModelIdGroup::Tag(TagId::from(id)),
			EditType::Collection => todo!(),
		})
	}
}



impl UpdatedBookEdit {
	pub fn is_empty(&self) -> bool {
		!self.title &&
		!self.clean_title &&
		!self.description &&
		!self.rating &&
		!self.isbn_10 &&
		!self.isbn_13 &&
		!self.is_public &&
		!self.available_at &&
		!self.language &&
		!self.publisher
	}
}



impl BookEdit {
	pub fn is_empty(&self) -> bool {
		self.title.is_none() &&
		self.clean_title.is_none() &&
		self.description.is_none() &&
		self.rating.is_none() &&
		self.isbn_10.is_none() &&
		self.isbn_13.is_none() &&
		self.is_public.is_none() &&
		self.available_at.is_none() &&
		self.language.is_none() &&
		self.publisher.is_none() &&
		self.added_people.is_none() &&
		self.removed_people.is_none() &&
		self.added_tags.is_none() &&
		self.removed_tags.is_none() &&
		self.added_images.is_none() &&
		self.removed_images.is_none()
	}
}

fn is_false(value: &bool) -> bool {
	!*value
}