use actix_web::{web, get};
use librarian_common::{api, EditId, item::edit::*};

use crate::{database::{Database}, WebResult, model::{EditModel, BookModel}};


// Get List Of Edits
#[get("/edits")]
pub async fn load_edit_list(
	db: web::Data<Database>,
	query: web::Query<api::SimpleListQuery>,
) -> WebResult<web::Json<api::GetEditListResponse>> {
	let offset = query.offset.unwrap_or(0);
	let limit = query.offset.unwrap_or(25);

	let mut existing_models: Vec<BookModel> = Vec::new();

	let mut items = Vec::new();

	for item in EditModel::get_all(offset, limit, &db).await? {
		let mut item: SharedEditModel = item.try_into()?;

		// Attempt to get the Book Model ID we're editing.
		if let Some(ModelIdGroup::Book(book_id)) = item.get_model_id() {
			if let EditData::Book(book_data) = &mut item.data {
				// If we've already queried the database for this book id, clone it.
				if let Some(book_model) = existing_models.iter().find(|v| v.id == book_id).cloned() {
					book_data.existing = Some(book_model.into());
				}
				// Query database for book id.
				else if let Some(book_model) = BookModel::get_by_id(book_id, &db).await? {
					existing_models.push(book_model.clone());
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

	Ok(web::Json(api::GetEditResponse {
		model: model.try_into()?
	}))
}