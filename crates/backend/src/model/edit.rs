use chrono::{DateTime, Utc, Duration};
use common::{BookId, PersonId, TagId, MemberId};
use common_local::{edit::*, EditId, item::edit::*};


mod edit_comment;
mod edit_vote;

pub use edit_comment::*;
pub use edit_vote::*;

use crate::{Result, edit_translate};

use super::{BookModel, MemberModel, BookTagModel, BookPersonModel, TagModel, PersonModel, ImageLinkModel, TableRow, AdvRow, row_int_to_usize, row_bigint_to_usize};


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


impl TableRow for EditModel {
    fn create(row: &mut AdvRow) -> Result<Self> {
        Ok(Self {
            id: row.next()?,

            type_of: row.next()?,
            operation: row.next()?,
            status: row.next()?,

            member_id: MemberId::from(row.next::<i32>()? as usize),
            model_id: row.next::<Option<i32>>()?.map(|v| v as usize),

            is_applied: row.next()?,

            vote_count: row.next::<i16>()? as usize,

            data: row.next()?,

            ended_at: row.next_opt()?,
            expires_at: row.next_opt()?,

            created_at: row.next()?,
            updated_at: row.next()?,
        })
    }
}


impl NewEditModel {
    pub async fn from_book_modify(member_id: MemberId, current: BookModel, updated: BookEdit, db: &tokio_postgres::Client) -> Result<Self> {
        let now = Utc::now();

        Ok(Self {
            type_of: EditType::Book,
            operation: EditOperation::Modify,
            status: EditStatus::Pending,
            member_id,
            model_id: Some(*current.id),
            is_applied: false,
            vote_count: 0,
            data: convert_data_to_string(EditType::Book, &new_edit_data_from_book(current, updated, db).await?)?,
            ended_at: None,
            expires_at: Some(now + Duration::days(7)),
            created_at: now,
            updated_at: now,
        })
    }

    pub async fn insert(self, db: &tokio_postgres::Client) -> Result<EditModel> {
        let row = db.query_one(r#"
            INSERT INTO edit (
                type_of, operation, status,
                member_id, model_id, is_applied, vote_count, data,
                ended_at, expires_at, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12) RETURNING id"#,
            params![
                self.type_of, self.operation, self.status,
                *self.member_id as i32, self.model_id.map(|v| v as i32), self.is_applied, self.vote_count as i16, &self.data,
                self.ended_at, self.expires_at,
                self.created_at, self.updated_at,
            ]
        ).await?;

        Ok(EditModel {
            id: EditId::from(row_int_to_usize(row)?),

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
    pub async fn get_all(offset: usize, limit: usize, db: &tokio_postgres::Client) -> Result<Vec<Self>> {
        let values = db.query(
            "SELECT * FROM edit LIMIT $1 OFFSET $2",
            params![ limit as i64, offset as i64 ]
        ).await?;

        values.into_iter().map(Self::from_row).collect()
    }

    pub async fn get_by_id(id: EditId, db: &tokio_postgres::Client) -> Result<Option<Self>> {
        db.query_opt(
            "SELECT * FROM edit WHERE id = $1",
            params![ id ],
        ).await?.map(Self::from_row).transpose()
    }

    pub async fn find_by_status(offset: usize, limit: usize, status: Option<EditStatus>, is_expired: Option<bool>, db: &tokio_postgres::Client) -> Result<Vec<Self>> {
        let mut expired_str = String::new();

        if let Some(expired) = is_expired {
            let now = Utc::now();

            if expired {
                expired_str = format!("AND expires_at < {now}");
            } else {
                expired_str = format!("AND expires_at > {now}");
            }
        }

        if let Some(status) = status {
            let values = db.query(
                &format!("SELECT * FROM edit WHERE status = $1 {expired_str} ORDER BY id DESC LIMIT $2 OFFSET $3"),
                params![ status, limit as i64, offset as i64 ]
            ).await?;

            values.into_iter().map(Self::from_row).collect()
        } else {
            if !expired_str.is_empty() {
                expired_str.insert_str(0, "WHERE ");
            }

            let values = db.query(
                &format!("SELECT * FROM edit {expired_str} ORDER BY id DESC LIMIT $1 OFFSET $2"),
                params![ limit as i64, offset as i64 ]
            ).await?;

            values.into_iter().map(Self::from_row).collect()
        }
    }

    pub async fn get_count(db: &tokio_postgres::Client) -> Result<usize> {
        row_bigint_to_usize(db.query_one("SELECT COUNT(*) FROM edit", &[]).await?)
    }

    pub async fn update_by_id(id: EditId, edit: UpdateEditModel, db: &tokio_postgres::Client) -> Result<u64> {
        let mut items = Vec::new();
        // We have to Box because DateTime doesn't return a borrow.
        let mut values = vec![
            Box::new(id) as Box<dyn tokio_postgres::types::ToSql + Sync>
        ];

        if let Some(value) = edit.vote {
            items.push("vote_count = vote_count +");
            values.push(Box::new(value) as Box<dyn tokio_postgres::types::ToSql + Sync>);
        }

        if let Some(value) = edit.status {
            items.push("status");
            values.push(Box::new(value) as Box<dyn tokio_postgres::types::ToSql + Sync>);
        }

        if let Some(value) = edit.is_applied {
            items.push("is_applied");
            values.push(Box::new(value) as Box<dyn tokio_postgres::types::ToSql + Sync>);
        }

        if let Some(value) = edit.ended_at {
            items.push("ended_at");
            values.push(Box::new(value) as Box<dyn tokio_postgres::types::ToSql + Sync>);
        }

        if let Some(value) = edit.expires_at {
            items.push("expires_at");
            values.push(Box::new(value) as Box<dyn tokio_postgres::types::ToSql + Sync>);
        }


        if items.is_empty() {
            return Ok(0);
        }

        Ok(db.execute(
            &format!(
                "UPDATE edit SET {} WHERE id = $1",
                items.iter()
                    .enumerate()
                    .map(|(i, v)| if v.contains('=') { format!("{v} ${}", 2 + i) } else { format!("{v} = ${}", 2 + i) })
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            &super::boxed_to_dyn_vec(&values)
        ).await?)
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
            EditType::Person => EditData::Person(serde_json::from_str(&self.data)?),
            EditType::Tag => EditData::Tag,
            EditType::Collection => EditData::Collection,
        })
    }

    pub async fn update_end_data_and_status(&mut self, value: Option<EditData>, db: &tokio_postgres::Client) -> Result<()> {
        if let Some(value) = value {
            match (self.type_of, value) {
                (EditType::Book, EditData::Book(v)) => self.data = serde_json::to_string(&v)?,
                (EditType::Person, EditData::Person(v)) => self.data = serde_json::to_string(&v)?,
                (EditType::Tag, EditData::Tag) => (),
                (EditType::Collection, EditData::Collection) => (),

                _ => panic!("save_data"),
            }

            db.execute(
                "UPDATE edit SET data = $2, status = $3, ended_at = $4 WHERE id = $1",
                params![ self.id, &self.data, self.status, self.ended_at ]
            ).await?;
        } else {
            db.execute(
                "UPDATE edit SET status = $2, ended_at = $3 WHERE id = $1",
                params![ self.id, self.status, self.ended_at ]
            ).await?;
        }

        Ok(())
    }

    pub fn into_shared_edit(self, member: Option<MemberModel>) -> Result<SharedEditModel> {
        let data = self.parse_data()?;

        Ok(SharedEditModel {
            id: self.id,
            type_of: self.type_of,
            operation: self.operation,
            status: self.status,
            member: member.map(|v| v.into()),
            votes: None,
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


    pub async fn process_status_change(&mut self, new_status: EditStatus, db: &tokio_postgres::Client) -> Result<()> {
        self.status = new_status;

        if !self.status.is_pending() {
            self.ended_at = Some(Utc::now());
        }

        if new_status.is_accepted() {
            match self.parse_data()? {
                EditData::Book(mut book_data) => {
                    match self.operation {
                        EditOperation::Modify => {
                            if let Some(book_model) = BookModel::get_by_id(BookId::from(self.model_id.unwrap()), db).await? {
                                accept_register_book_data_overwrites(book_model, &mut book_data, db).await?;

                                self.update_end_data_and_status(Some(EditData::Book(book_data)), db).await?;
                            }
                        }

                        EditOperation::Create => (),
                        EditOperation::Delete => (),
                        EditOperation::Merge => (),
                    }
                }

                EditData::Person(_) => todo!(),
                EditData::Tag => todo!(),
                EditData::Collection => todo!(),
            }
        } else {
            self.update_end_data_and_status(None, db).await?;
        }

        Ok(())
    }
}


pub async fn new_edit_data_from_book(current: BookModel, updated: BookEdit, db: &tokio_postgres::Client) -> Result<EditData> {
    // TODO: Cleaner, less complicated way?

    let current_people = if updated.added_people.is_some() {
        Some(BookPersonModel::get_all_by_book_id(current.id, db).await?
            .into_iter()
            .map(|v| v.person_id)
            .collect())
    } else {
        None
    };

    let (title_old, title) = edit_translate::cmp_opt_string(current.title, updated.title);
    let (clean_title_old, clean_title) = edit_translate::cmp_opt_string(current.clean_title, updated.clean_title);
    let (description_old, description) = edit_translate::cmp_opt_string(current.description, updated.description);
    let (rating_old, rating) = edit_translate::cmp_opt_partial_eq(Some(current.rating), updated.rating);
    let (isbn_10_old, isbn_10) = edit_translate::cmp_opt_string(current.isbn_10, updated.isbn_10);
    let (isbn_13_old, isbn_13) = edit_translate::cmp_opt_string(current.isbn_13, updated.isbn_13);
    let (is_public_old, is_public) = edit_translate::cmp_opt_bool(Some(current.is_public), updated.is_public);
    let (available_at_old, available_at) = edit_translate::cmp_opt_partial_eq(current.available_at.map(|v| v.and_hms(0, 0, 0).timestamp()), updated.available_at);
    let (language_old, language) = edit_translate::cmp_opt_partial_eq(Some(current.language), updated.language);
    let (added_people_old, added_people) = edit_translate::cmp_opt_partial_eq(current_people, updated.added_people);

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
        added_people,
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
        added_people: added_people_old,
        removed_people: None,
        added_tags: None,
        removed_tags: None,
        added_images: None,
        removed_images: None,
    };

    Ok(EditData::Book(BookEditData {
        current: None,
        new: Some(new).filter(|v| !v.is_empty()),
        old: Some(old).filter(|v| !v.is_empty()),
        updated: None,
    }))
}



// We use EditType to double check that we're using the correct EditData.
pub fn convert_data_to_string(type_of: EditType, value: &EditData) -> Result<String> {
    Ok(match (type_of, value) {
        (EditType::Book, EditData::Book(book)) => serde_json::to_string(&book)?,

        v => todo!("convert_data_to_string: {:?}", v),
    })
}



/// Update: BookModel
///
/// Link: Tags, People, Images
pub async fn accept_register_book_data_overwrites(
    mut book_model: BookModel,
    edit: &mut BookEditData,
    db: &tokio_postgres::Client
) -> Result<()> {
    let (old, new) = match (edit.old.clone().unwrap_or_default(), edit.new.clone()) {
        (a, Some(b)) => (a, b),
        _ => return Ok(())
    };

    let mut book_edits = UpdatedBookEdit::default();

    // Update Book
    cmp_opt_old_and_new_return(&mut book_edits.title, &mut book_model.title, old.title, new.title);
    cmp_opt_old_and_new_return(&mut book_edits.clean_title, &mut book_model.clean_title, old.clean_title, new.clean_title);
    cmp_opt_old_and_new_return(&mut book_edits.description, &mut book_model.description, old.description, new.description);
    cmp_opt_old_and_new_return(&mut book_edits.isbn_10, &mut book_model.isbn_10, old.isbn_10, new.isbn_10);
    cmp_opt_old_and_new_return(&mut book_edits.isbn_13, &mut book_model.isbn_13, old.isbn_13, new.isbn_13);
    cmp_opt_old_and_new_return(&mut book_edits.available_at, &mut book_model.available_at.map(|v| v.and_hms(0, 0, 0).timestamp()), old.available_at, new.available_at);
    cmp_old_and_new_return(&mut book_edits.language, &mut book_model.language, old.language, new.language);
    cmp_old_and_new_return(&mut book_edits.rating, &mut book_model.rating, old.rating, new.rating);
    cmp_old_and_new_return(&mut book_edits.is_public, &mut book_model.is_public, old.is_public, new.is_public);
    // TODO: publisher

    edit.updated = Some(book_edits).filter(|v| !v.is_empty());

    book_model.update_book(db).await?;


    // Tags
    if let Some(values) = new.added_tags {
        for tag_id in values {
            if TagModel::get_by_id(tag_id, db).await?.is_some() {
                BookTagModel::insert(book_model.id, tag_id, None, db).await?;
            }
        }
    }

    if let Some(values) = new.removed_tags {
        for tag_id in values {
            BookTagModel::remove(book_model.id, tag_id, db).await?;
        }
    }


    // Images
    if let Some(values) = new.added_images {
        for id_or_url in values {
            let image_id = match id_or_url {
                NewOrCachedImage::Id(v) => v,
                NewOrCachedImage::Url(url) => {
                    let resp = reqwest::get(url)
                        .await?
                        .bytes()
                        .await?;

                    let model = crate::store_image(resp.to_vec(), db).await?;

                    model.id
                }
            };

            ImageLinkModel::new_book(image_id, book_model.id)
                .insert(db).await?;
        }
    }

    if let Some(values) = new.removed_images {
        for image_id in values {
            ImageLinkModel::new_book(image_id, book_model.id)
                .remove(db).await?;
        }
    }


    // People
    if let Some(values) = new.added_people {
        for person_id in values {
            if PersonModel::get_by_id(person_id, db).await?.is_some() {
                BookPersonModel::new(book_model.id, person_id)
                    .insert(db).await?;
            }
        }
    }

    if let Some(values) = new.removed_people {
        for person_id in values {
            BookPersonModel::new(book_model.id, person_id)
                .remove(db).await?;
        }
    }

    Ok(())
}

/// Returns the new value if current and old are equal.
fn cmp_old_and_new_return<V: PartialEq + Default>(edited: &mut bool, current: &mut V, old: Option<V>, new: Option<V>) {
    match (old, new) {
        // If we have an old value and new value.
        (Some(old), Some(new)) => {
            if *current == old {
                *current = new;
                *edited = true;
            }
        }

        // If we are just inserting a new value.
        (None, Some(new)) => {
            *current = new;
            *edited = true;
        }

        // If we're unsetting a value
        (Some(old), None) => {
            if *current == old {
                // TODO: Determine if we should keep.
                *current = V::default();
                *edited = true;
            }
        }

        _ => ()
    }
}

/// Returns the new value if current and old are equal.
fn cmp_opt_old_and_new_return<V: PartialEq>(edited: &mut bool, current: &mut Option<V>, old: Option<V>, new: Option<V>) {
    if (old.is_some() || new.is_some()) && *current == old {
        *current = new;
        *edited = true;
    }
}