use actix_web::{web, get};
use librarian_common::{api, EditId, item::edit::*};

use crate::{database::{Database}, WebResult, model::{EditModel, BookModel, MemberModel}};


// Get List Of Edits
#[get("/edits")]
pub async fn load_edit_list(
	db: web::Data<Database>,
	query: web::Query<api::SimpleListQuery>,
) -> WebResult<web::Json<api::GetEditListResponse>> {
	let offset = query.offset.unwrap_or(0);
	let limit = query.offset.unwrap_or(25);

	let mut existing_books: Vec<BookModel> = Vec::new();
	let mut existing_members: Vec<MemberModel> = Vec::new();

	let mut items = Vec::new();

	for item in EditModel::get_all(offset, limit, &db).await? {
		let member = if let Some(v) = existing_members.iter().find(|v| v.id == item.member_id).cloned() {
			Some(v)
		} else if let Some(v) = MemberModel::get_by_id(item.member_id, &db).await? {
			existing_members.push(v.clone());
			Some(v)
		} else {
			None
		};


		let mut item: SharedEditModel = item.into_shared_edit(member)?;

		// Attempt to get the Book Model ID we're editing.
		if let Some(ModelIdGroup::Book(book_id)) = item.get_model_id() {
			if let EditData::Book(book_data) = &mut item.data {
				// If we've already queried the database for this book id, clone it.
				if let Some(book_model) = existing_books.iter().find(|v| v.id == book_id).cloned() {
					book_data.existing = Some(book_model.into());
				}
				// Query database for book id.
				else if let Some(book_model) = BookModel::get_by_id(book_id, &db).await? {
					existing_books.push(book_model.clone());
					book_data.existing = Some(book_model.into());
				}
			}
		}

		items.push(item);
	}

	Ok(web::Json(api::GetEditListResponse {
		offset,
		limit,
		total: EditModel::get_count(&db).await?,
		items
	}))
}


// Edit
#[get("/edit/{id}")]
async fn load_edit(edit_id: web::Path<EditId>, db: web::Data<Database>) -> WebResult<web::Json<api::GetEditResponse>> {
	let model = EditModel::get_by_id(*edit_id, &db).await?.unwrap();

	let member = MemberModel::get_by_id(model.member_id, &db).await?;

	Ok(web::Json(api::GetEditResponse {
		model: model.into_shared_edit(member)?
	}))
}