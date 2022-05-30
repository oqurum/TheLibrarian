use chrono::{DateTime, Utc, TimeZone};
use librarian_common::{edit::*, EditId, MemberId, PersonId, TagId, ImageId};
use rusqlite::{Row, params};
use serde::{Serialize, Deserialize};

use crate::{Result, Database};


pub struct NewEditModel {
	pub type_of: EditType,
	pub operation: EditOperation,
	pub status: EditStatus,

	pub member_id: MemberId,

	pub is_applied: bool,
	pub vote_count: usize,

	pub data: String,

	pub ended_at: DateTime<Utc>,
	pub expires_at: DateTime<Utc>,

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

	pub ended_at: DateTime<Utc>,
	pub expires_at: DateTime<Utc>,

	pub created_at: DateTime<Utc>,
	pub updated_at: DateTime<Utc>,
}


#[allow(clippy::large_enum_variant)]
pub enum EditData {
	Book(BookEditData),
	Person,
	Tag,
	Collection,
}



#[derive(Debug, Serialize, Deserialize)]
pub struct BookEditData {
	new: Option<BookEdit>,
	old: Option<BookEdit>, // Based off of current Model. If field is different than current Model once it's updating it'll ignore it.
}


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

			ended_at: Utc.timestamp_millis(value.get(8)?),
			expires_at: Utc.timestamp_millis(value.get(9)?),

			created_at: Utc.timestamp_millis(value.get(10)?),
			updated_at: Utc.timestamp_millis(value.get(11)?),
		})
	}
}


impl NewEditModel {
	pub async fn insert(self, db: &Database) -> Result<EditModel> {
		let lock = db.write().await;

		lock.execute(r#"
			INSERT INTO edit (
				type_of, operation, status
				member_id, is_applied, vote_count, data
				ended_at, expires_at, created_at, updated_at
			)
			VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)"#,
			params![
				self.type_of, self.operation, self.status,
				self.member_id, self.is_applied, self.vote_count, &self.data,
				self.ended_at.timestamp_millis(), self.expires_at.timestamp_millis(),
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
	pub fn parse_data(&self) -> crate::Result<EditData> {
		Ok(match self.type_of {
			EditType::Book => EditData::Book(serde_json::from_str(&self.data)?),
			EditType::Person => EditData::Person,
			EditType::Tag => EditData::Tag,
			EditType::Collection => EditData::Collection,
		})
	}

	pub fn save_data(&mut self, value: EditData) -> crate::Result<EditData> {
		Ok(match self.type_of {
			EditType::Book => todo!(),
			EditType::Person => todo!(),
			EditType::Tag => todo!(),
			EditType::Collection => todo!(),
		})
	}
}