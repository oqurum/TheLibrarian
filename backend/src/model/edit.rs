use chrono::{DateTime, Utc, TimeZone, Duration};
use librarian_common::{edit::*, EditId, MemberId, item::edit::*, BookId, PersonId, TagId};
use rusqlite::{Row, params, OptionalExtension};


mod edit_comment;
mod edit_vote;

pub use edit_comment::*;
pub use edit_vote::*;

use crate::{Result, Database, edit_translate::{self, Output}};

use super::{BookModel, MemberModel};


#[derive(Debug)]
pub struct NewEditModel {
	pub type_of: EditType,
	pub operation: EditOperation,
	pub status: EditStatus,

	pub member_id: MemberId,
	/// Unset if Operation is Create, if unset, set after accepted
	pub model_id: Option<usize>, // TODO: Make ModelIdGroup

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
	// TODO: Add Model Id AFTER Operation::Create is accepted
	/// Unset if Operation is Create, if unset, set after accepted
	pub model_id: Option<usize>, // TODO: Make ModelIdGroup

	pub is_applied: bool,
	pub vote_count: usize,

	pub data: String,

	pub ended_at: Option<DateTime<Utc>>,
	pub expires_at: Option<DateTime<Utc>>,

	pub created_at: DateTime<Utc>,
	pub updated_at: DateTime<Utc>,
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
			model_id: value.get(5)?,

			is_applied: value.get(6)?,

			vote_count: value.get(7)?,

			data: value.get(8)?,

			ended_at: value.get::<_, Option<_>>(9)?.map(|v| Utc.timestamp_millis(v)),
			expires_at: value.get::<_, Option<_>>(10)?.map(|v| Utc.timestamp_millis(v)),

			created_at: Utc.timestamp_millis(value.get(11)?),
			updated_at: Utc.timestamp_millis(value.get(12)?),
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
			model_id: Some(*current.id),
			is_applied: false,
			vote_count: 0,
			data: convert_data_to_string(EditType::Book, &new_edit_data_from_book(current, updated))?,
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
				member_id, model_id, is_applied, vote_count, data,
				ended_at, expires_at, created_at, updated_at
			)
			VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)"#,
			params![
				self.type_of, self.operation, self.status,
				self.member_id, self.model_id, self.is_applied, self.vote_count, &self.data,
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
			model_id: self.model_id,

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
	pub async fn get_all(offset: usize, limit: usize, db: &Database) -> Result<Vec<Self>> {
		let this = db.read().await;

		let mut conn = this.prepare(r#"SELECT * FROM edit LIMIT ?1 OFFSET ?2"#)?;

		let map = conn.query_map([limit, offset], |v| Self::try_from(v))?;

		Ok(map.collect::<std::result::Result<Vec<_>, _>>()?)
	}

	pub async fn get_by_id(id: EditId, db: &Database) -> Result<Option<Self>> {
		Ok(db.read().await.query_row(
			r#"SELECT * FROM edit WHERE id = ?1 LIMIT 1"#,
			params![id],
			|v| Self::try_from(v)
		).optional()?)
	}

	pub async fn get_count(db: &Database) -> Result<usize> {
		Ok(db.read().await.query_row(r#"SELECT COUNT(*) FROM edit"#, [], |v| v.get(0))?)
	}

	pub async fn update_by_id(id: EditId, edit: UpdateEditModel, db: &Database) -> Result<usize> {
		let mut items = Vec::new();
		// We have to Box because DateTime doesn't return a borrow.
		let mut values = vec![
			Box::new(id) as Box<dyn rusqlite::ToSql>
		];

		if let Some(value) = edit.vote {
			let value = if value { 1 } else { -1 };

			items.push("vote_count");
			values.push(Box::new(format!("vote_count + {value}")) as Box<dyn rusqlite::ToSql>);
		}

		if let Some(value) = edit.status {
			items.push("status");
			values.push(Box::new(value) as Box<dyn rusqlite::ToSql>);
		}

		if let Some(value) = edit.is_applied {
			items.push("is_applied");
			values.push(Box::new(value) as Box<dyn rusqlite::ToSql>);
		}

		if let Some(value) = edit.ended_at {
			items.push("ended_at");
			values.push(Box::new(value.map(|v| v.timestamp_millis())) as Box<dyn rusqlite::ToSql>);
		}

		if let Some(value) = edit.expires_at {
			items.push("expires_at");
			values.push(Box::new(value.map(|v| v.timestamp_millis())) as Box<dyn rusqlite::ToSql>);
		}


		if items.is_empty() {
			return Ok(0);
		}

		Ok(db.write().await
		.execute(
			&format!(
				"UPDATE edit SET {} WHERE id = ?1",
				items.iter()
					.enumerate()
					.map(|(i, v)| format!("{v} = ?{}", 2 + i))
					.collect::<Vec<_>>()
					.join(", ")
			),
			rusqlite::params_from_iter(values.iter().map(|v| &*v))
		)?)
	}

	pub fn get_model_id(&self) -> Option<ModelIdGroup> {
		self.model_id.map(|id| match self.type_of {
			EditType::Book => ModelIdGroup::Book(BookId::from(id)),
			EditType::Person => ModelIdGroup::Person(PersonId::from(id)),
			EditType::Tag => ModelIdGroup::Tag(TagId::from(id)),
			EditType::Collection => todo!(),
		})
	}

	pub fn parse_data(&self) -> Result<EditData> {
		Ok(match self.type_of {
			EditType::Book => EditData::Book(serde_json::from_str(&self.data)?),
			EditType::Person => EditData::Person,
			EditType::Tag => EditData::Tag,
			EditType::Collection => EditData::Collection,
		})
	}

	pub fn into_shared_edit(self, member: Option<MemberModel>) -> Result<SharedEditModel> {
		let data = self.parse_data()?;

		Ok(SharedEditModel {
			id: self.id,
			type_of: self.type_of,
			operation: self.operation,
			status: self.status,
			member: member.map(|v| v.into()),
			model_id: self.model_id,
			is_applied: self.is_applied,
			vote_count: self.vote_count,
			data,
			ended_at: self.ended_at,
			expires_at: self.expires_at,
			created_at: self.created_at,
			updated_at: self.updated_at,
		})
	}
}


pub fn new_edit_data_from_book(current: BookModel, updated: BookModel) -> EditData {
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

	EditData::Book(BookEditData {
		existing: None,
		new: Some(new).filter(|v| !v.is_empty()),
		old: Some(old).filter(|v| !v.is_empty()),
	})
}



// We use EditType to double check that we're using the correct EditData.
pub fn convert_data_to_string(type_of: EditType, value: &EditData) -> Result<String> {
	Ok(match (type_of, value) {
		(EditType::Book, EditData::Book(book)) => serde_json::to_string(&book)?,

		v => todo!("convert_data_to_string: {:?}", v),
	})
}