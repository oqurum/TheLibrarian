use chrono::{DateTime, Utc, TimeZone, Duration};
use librarian_common::{edit::*, EditId, MemberId, PersonId, TagId, ImageId};
use rusqlite::{Row, params};
use serde::{Serialize, Deserialize};


mod edit_comment;
mod edit_vote;

pub use edit_comment::*;
pub use edit_vote::*;

use crate::{Result, Database, edit_translate::{self, Output}};

use super::BookModel;


#[derive(Debug)]
pub struct NewEditModel {
	pub type_of: EditType,
	pub operation: EditOperation,
	pub status: EditStatus,

	pub member_id: MemberId,

	pub is_applied: bool,
	pub vote_count: usize,

	pub data: String,

	pub ended_at: Option<DateTime<Utc>>,
	pub expires_at: Option<DateTime<Utc>>,

	pub created_at: DateTime<Utc>,
	pub updated_at: DateTime<Utc>,
}


#[derive(Debug, Clone)]
pub struct EditModel {
	pub id: EditId,

	pub type_of: EditType,
	pub operation: EditOperation,
	pub status: EditStatus,

	pub member_id: MemberId,

	pub is_applied: bool,
	pub vote_count: usize,

	pub data: String,

	pub ended_at: Option<DateTime<Utc>>,
	pub expires_at: Option<DateTime<Utc>>,

	pub created_at: DateTime<Utc>,
	pub updated_at: DateTime<Utc>,
}


#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub enum EditData {
	Book(BookEditData),
	Person,
	Tag,
	Collection,
}



#[derive(Debug, Serialize, Deserialize)]
pub struct BookEditData {
	#[serde(skip_serializing_if = "Option::is_none")]
	new: Option<BookEdit>,
	#[serde(skip_serializing_if = "Option::is_none")]
	old: Option<BookEdit>, // Based off of current Model. If field is different than current Model once it's updating it'll ignore it.
}


// TODO: Move to common. Use as the Book Editing struct.
#[derive(Debug, Serialize, Deserialize)]
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
	pub added_images: Option<Vec<ImageId>>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub removed_images: Option<Vec<ImageId>>,
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


impl<'a> TryFrom<&Row<'a>> for EditModel {
	type Error = rusqlite::Error;

	fn try_from(value: &Row<'a>) -> std::result::Result<Self, Self::Error> {
		Ok(Self {
			id: value.get(0)?,

			type_of: value.get(1)?,
			operation: value.get(2)?,
			status: value.get(3)?,

			member_id: value.get(4)?,

			is_applied: value.get(5)?,

			vote_count: value.get(6)?,

			data: value.get(7)?,

			ended_at: value.get::<_, Option<_>>(8)?.map(|v| Utc.timestamp_millis(v)),
			expires_at: value.get::<_, Option<_>>(9)?.map(|v| Utc.timestamp_millis(v)),

			created_at: Utc.timestamp_millis(value.get(10)?),
			updated_at: Utc.timestamp_millis(value.get(11)?),
		})
	}
}


impl NewEditModel {
	pub fn from_book_modify(member_id: MemberId, current: BookModel, updated: BookModel) -> Result<Self> {
		let now = Utc::now();

		Ok(Self {
			type_of: EditType::Book,
			operation: EditOperation::Modify,
			status: EditStatus::Pending,
			member_id,
			is_applied: false,
			vote_count: 0,
			data: convert_data_to_string(EditType::Book, &EditData::from_book(current, updated))?,
			ended_at: None,
			expires_at: Some(now + Duration::days(7)),
			created_at: now,
			updated_at: now,
		})
	}

	pub async fn insert(self, db: &Database) -> Result<EditModel> {
		let lock = db.write().await;

		lock.execute(r#"
			INSERT INTO edit (
				type_of, operation, status,
				member_id, is_applied, vote_count, data,
				ended_at, expires_at, created_at, updated_at
			)
			VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)"#,
			params![
				self.type_of, self.operation, self.status,
				self.member_id, self.is_applied, self.vote_count, &self.data,
				self.ended_at.map(|v| v.timestamp_millis()), self.expires_at.map(|v| v.timestamp_millis()),
				self.created_at.timestamp_millis(), self.updated_at.timestamp_millis(),
			]
		)?;

		Ok(EditModel {
			id: EditId::from(lock.last_insert_rowid() as usize),

			type_of: self.type_of,
			operation: self.operation,
			status: self.status,

			member_id: self.member_id,

			is_applied: self.is_applied,
			vote_count: self.vote_count,

			data: self.data,

			ended_at: self.ended_at,
			expires_at: self.expires_at,
			created_at: self.created_at,
			updated_at: self.updated_at,
		})
	}
}


impl EditModel {
	pub fn parse_data(&self) -> Result<EditData> {
		Ok(match self.type_of {
			EditType::Book => EditData::Book(serde_json::from_str(&self.data)?),
			EditType::Person => EditData::Person,
			EditType::Tag => EditData::Tag,
			EditType::Collection => EditData::Collection,
		})
	}
}


impl EditData {
	pub fn from_book(current: BookModel, updated: BookModel) -> Self {
		// TODO: Cleaner, less complicated way?

		let Output { new_value: title, old_value: title_old } = edit_translate::cmp_opt_string(
			current.title, updated.title);
		let Output { new_value: clean_title, old_value: clean_title_old } = edit_translate::cmp_opt_string(
			current.clean_title, updated.clean_title);
		let Output { new_value: description, old_value: description_old } = edit_translate::cmp_opt_string(
			current.description, updated.description);
		let Output { new_value: rating, old_value: rating_old } = edit_translate::cmp_opt_number(
			Some(current.rating), Some(updated.rating));
		let Output { new_value: isbn_10, old_value: isbn_10_old } = edit_translate::cmp_opt_string(
			current.isbn_10, updated.isbn_10);
		let Output { new_value: isbn_13, old_value: isbn_13_old } = edit_translate::cmp_opt_string(
			current.isbn_13, updated.isbn_13);
		let Output { new_value: is_public, old_value: is_public_old } = edit_translate::cmp_opt_bool(
			Some(current.is_public), Some(updated.is_public));
		let Output { new_value: available_at, old_value: available_at_old } = edit_translate::cmp_opt_string(
			current.available_at, updated.available_at);
		let Output { new_value: language, old_value: language_old } = edit_translate::cmp_opt_number(
			current.language, updated.language);

		let new = BookEdit {
			title,
			clean_title,
			description,
			rating,
			isbn_10,
			isbn_13,
			is_public,
			available_at,
			language,
			publisher: None, // TODO
			added_people: None,
			removed_people: None,
			added_tags: None,
			removed_tags: None,
			added_images: None,
			removed_images: None,
		};

		let old = BookEdit {
			title: title_old,
			clean_title: clean_title_old,
			description: description_old,
			rating: rating_old,
			isbn_10: isbn_10_old,
			isbn_13: isbn_13_old,
			is_public: is_public_old,
			available_at: available_at_old,
			language: language_old,
			publisher: None,
			added_people: None,
			removed_people: None,
			added_tags: None,
			removed_tags: None,
			added_images: None,
			removed_images: None,
		};

		Self::Book(BookEditData {
			new: Some(new).filter(|v| !v.is_empty()),
			old: Some(old).filter(|v| !v.is_empty()),
		})
	}
}



// We use EditType to double check that we're using the correct EditData.
pub fn convert_data_to_string(type_of: EditType, value: &EditData) -> Result<String> {
	Ok(match (type_of, value) {
		(EditType::Book, EditData::Book(book)) => serde_json::to_string(&book)?,

		v => todo!("convert_data_to_string: {:?}", v),
	})
}