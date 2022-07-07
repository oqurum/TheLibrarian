use chrono::{DateTime, Utc};
use common::{MemberId, BookId, PersonId, TagId};
use serde::{Serialize, Deserialize};

use crate::{EditId, edit::*, util::*, Member, api::QueryListResponse, EditVoteId};


pub use book_edit::*;
pub use person_edit::*;



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



#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EditData {
	Book(BookEditData),
	Person(PersonEditData),
	Tag,
	Collection,
}


#[derive(Debug, Clone, Copy)]
pub enum ModelIdGroup {
	Book(BookId),
	Person(PersonId),
	Tag(TagId),
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InnerEditData<Curr, Cached, Update: Clone + Default> {
	#[serde(skip_serializing_if = "Option::is_none")]
	pub current: Option<Curr>,

	#[serde(skip_serializing_if = "Option::is_none")]
	pub new: Option<Cached>,

	#[serde(skip_serializing_if = "Option::is_none")]
	pub old: Option<Cached>, // Based off of current Model. If field is different than current Model once it's updating it'll ignore it.

	#[serde(skip_serializing_if = "Option::is_none")]
	pub updated: Option<Update>,
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


fn is_false(value: &bool) -> bool {
	!*value
}


mod book_edit {
	use std::borrow::Cow;

	use common::ImageId;

	use crate::DisplayMetaItem;
	use super::*;


	#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
	pub enum NewOrCachedImage {
		Url(String),
		Id(ImageId),
	}

	impl NewOrCachedImage {
		pub fn as_url(&self) -> Cow<str> {
			match self {
				Self::Url(v) => Cow::Borrowed(v.as_str()),
				Self::Id(v) => Cow::Owned(format!("/api/v1/book/{v}/thumbnail")),
			}
		}
	}


	pub type BookEditData = InnerEditData<DisplayMetaItem, BookEdit, UpdatedBookEdit>;

	// TODO: Option<Option<_>> Values. Allows for only updating specific values.
	#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
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
		pub added_images: Option<Vec<NewOrCachedImage>>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub removed_images: Option<Vec<ImageId>>,
	}

	#[derive(Debug, Clone, Default, Serialize, Deserialize)]
	pub struct UpdatedBookEdit {
		#[serde(default, skip_serializing_if = "is_false")]
		pub title: bool,
		#[serde(default, skip_serializing_if = "is_false")]
		pub clean_title: bool,
		#[serde(default, skip_serializing_if = "is_false")]
		pub description: bool,

		#[serde(default, skip_serializing_if = "is_false")]
		pub rating: bool,

		#[serde(default, skip_serializing_if = "is_false")]
		pub isbn_10: bool,
		#[serde(default, skip_serializing_if = "is_false")]
		pub isbn_13: bool,

		#[serde(default, skip_serializing_if = "is_false")]
		pub is_public: bool,

		#[serde(default, skip_serializing_if = "is_false")]
		pub available_at: bool,
		#[serde(default, skip_serializing_if = "is_false")]
		pub language: bool,

		#[serde(default, skip_serializing_if = "is_false")]
		pub publisher: bool,
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

		pub fn insert_added_tag(&mut self, value: TagId) {
			self.added_tags.get_or_insert_with(Default::default).push(value);
		}

		pub fn insert_removed_tag(&mut self, value: TagId) {
			self.removed_tags.get_or_insert_with(Default::default).push(value);
		}

		pub fn remove_tag(&mut self, value: TagId) {
			if let Some(list) = self.added_tags.as_mut() {
				if let Some(index) = list.iter().position(|&id| value == id) {
					list.remove(index);

					if list.is_empty() {
						self.added_tags = None;
					}

					return;
				}
			}

			if let Some(list) = self.removed_tags.as_mut() {
				if let Some(index) = list.iter().position(|&id| value == id) {
					list.remove(index);

					if list.is_empty() {
						self.removed_tags = None;
					}
				}
			}
		}
	}


	#[cfg(feature = "frontend")]
	mod _bookedit_frontend {
		use std::collections::HashMap;

		use common::component::popup::compare::{Comparable, CompareContainer, CompareDisplay, MapContainer, morph_map_value};

		use super::*;

		impl Comparable for BookEdit {
			fn create_comparison_with(&self, other: &Self) -> serde_json::Result<CompareContainer> {
				Ok(CompareContainer::create(
					vec![
						("title", "Title", CompareDisplay::Text),
						("clean_title", "Clean Title", CompareDisplay::Text),
						("description", "Description", CompareDisplay::Text),
						("rating", "Rating", CompareDisplay::Text),
						("isbn_10", "ISBN 10", CompareDisplay::Text),
						("isbn_13", "ISBN 13", CompareDisplay::Text),
						("is_public", "Is Public", CompareDisplay::Text),
						("available_at", "Available At", CompareDisplay::Text),
						("language", "Language", CompareDisplay::Text),
						("publisher", "Publisher", CompareDisplay::Text),

						("added_people", "Added People", CompareDisplay::Text),
						("removed_people", "Removed People", CompareDisplay::Text),
						("added_tags", "Added Tags", CompareDisplay::Text),
						("removed_tags", "Removed Tags", CompareDisplay::Text),
						("added_images", "Added Images", CompareDisplay::Image),
						("removed_images", "Removed Images", CompareDisplay::Image),
					],
					self.create_map()?,
					other.create_map()?
				))
			}

			fn create_from_comparison(mut map: HashMap<&'static str, serde_json::Value>) -> serde_json::Result<Self> where Self: Sized {
				Ok(Self {
					title: map.remove("title").map(serde_json::from_value).transpose()?,
					clean_title: map.remove("clean_title").map(serde_json::from_value).transpose()?,
					description: map.remove("description").map(serde_json::from_value).transpose()?,
					rating: map.remove("rating").map(serde_json::from_value).transpose()?,
					isbn_10: map.remove("isbn_10").map(serde_json::from_value).transpose()?,
					isbn_13: map.remove("isbn_13").map(serde_json::from_value).transpose()?,
					is_public: map.remove("is_public").map(serde_json::from_value).transpose()?,
					available_at: map.remove("available_at").map(serde_json::from_value).transpose()?,
					language: map.remove("language").map(serde_json::from_value).transpose()?,
					publisher: map.remove("publisher").map(serde_json::from_value).transpose()?,

					added_people: map.remove("added_people").map(serde_json::from_value).transpose()?,
					removed_people: map.remove("removed_people").map(serde_json::from_value).transpose()?,
					added_tags: map.remove("added_tags").map(serde_json::from_value).transpose()?,
					removed_tags: map.remove("removed_tags").map(serde_json::from_value).transpose()?,
					added_images: map.remove("added_images").map(serde_json::from_value).transpose()?,
					removed_images: map.remove("removed_images").map(serde_json::from_value).transpose()?,
				})
			}

			fn create_map(&self) -> serde_json::Result<MapContainer> {
				let mut map = MapContainer::with_capacity(16);

				self.title.clone().map(|v| Ok(map.insert("title", morph_map_value(v)?))).transpose()?;
				self.clean_title.clone().map(|v| Ok(map.insert("clean_title", morph_map_value(v)?))).transpose()?;
				self.description.clone().map(|v| Ok(map.insert("description", morph_map_value(v)?))).transpose()?;
				self.rating.map(|v| Ok(map.insert("rating", morph_map_value(v)?))).transpose()?;
				self.isbn_10.clone().map(|v| Ok(map.insert("isbn_10", morph_map_value(v)?))).transpose()?;
				self.isbn_13.clone().map(|v| Ok(map.insert("isbn_13", morph_map_value(v)?))).transpose()?;
				self.is_public.map(|v| Ok(map.insert("is_public", morph_map_value(v)?))).transpose()?;
				self.available_at.clone().map(|v| Ok(map.insert("available_at", morph_map_value(v)?))).transpose()?;
				self.language.map(|v| Ok(map.insert("language", morph_map_value(v)?))).transpose()?;
				self.publisher.clone().map(|v| Ok(map.insert("publisher", morph_map_value(v)?))).transpose()?;

				// TODO:
				// self.added_people.clone().map(|v| Ok(map.insert("added_people", morph_map_value(v)?))).transpose()?;
				// self.removed_people.clone().map(|v| Ok(map.insert("removed_people", morph_map_value(v)?))).transpose()?;
				// self.added_tags.clone().map(|v| Ok(map.insert("added_tags", morph_map_value(v)?))).transpose()?;
				// self.removed_tags.clone().map(|v| Ok(map.insert("removed_tags", morph_map_value(v)?))).transpose()?;
				self.added_images.as_deref()
					.map(|v| Ok(map.insert("added_images", morph_map_value(v.iter().map(|v| v.as_url().into_owned()).collect::<Vec<_>>())?)))
					.transpose()?;
				self.removed_images.as_deref()
					.map(|v| Ok(map.insert("removed_images", morph_map_value(v.iter().map(|v| format!("/api/v1/book/{v}/thumbnail")).collect::<Vec<_>>())?)))
					.transpose()?;

				Ok(map)
			}
		}
	}
}

mod person_edit {
	use common::ThumbnailStore;

	use crate::Person;
	use super::*;

	pub type PersonEditData = InnerEditData<Person, PersonEdit, UpdatedPersonEdit>;

	#[derive(Debug, Clone, Default, Serialize, Deserialize)]
	pub struct PersonEdit {
		#[serde(skip_serializing_if = "Option::is_none")]
		pub name: Option<String>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub description: Option<String>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub birth_date: Option<String>,

		#[serde(skip_serializing_if = "Option::is_none")]
		pub added_images: Option<Vec<ThumbnailStore>>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub removed_images: Option<Vec<ThumbnailStore>>,
	}

	#[derive(Debug, Clone, Default, Serialize, Deserialize)]
	pub struct UpdatedPersonEdit {
		#[serde(default, skip_serializing_if = "is_false")]
		pub name: bool,
		#[serde(default, skip_serializing_if = "is_false")]
		pub description: bool,
		#[serde(default, skip_serializing_if = "is_false")]
		pub birth_date: bool,
	}


	impl PersonEdit {
		pub fn is_empty(&self) -> bool {
			self.name.is_none() &&
			self.description.is_none() &&
			self.birth_date.is_none() &&
			self.added_images.is_none() &&
			self.removed_images.is_none()
		}
	}

	impl UpdatedPersonEdit {
		pub fn is_empty(&self) -> bool {
			!self.name &&
			!self.description &&
			!self.birth_date
		}
	}
}