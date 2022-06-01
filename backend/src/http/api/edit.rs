use actix_web::{web, get, post, HttpResponse};
use librarian_common::{api, EditId, item::edit::*, Permissions};

use crate::{database::{Database}, WebResult, model::{EditModel, BookModel, MemberModel}, http::MemberCookie};


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


#[post("/edit/{id}")]
async fn update_edit(
	edit_id: web::Path<EditId>,
	json: web::Json<UpdateEditModel>,
	member: MemberCookie,
	db: web::Data<Database>
) -> WebResult<HttpResponse> {
	let mut update = json.into_inner();

	update.ended_at = None;
	update.expires_at = None;
	update.is_applied = None;

	let member = MemberModel::get_by_id(member.member_id(), &db).await?.unwrap();

	// Only an Admin can change the status.
	if update.status.is_some() && !member.permissions.contains(Permissions::ADMIN) {
		return Ok(HttpResponse::InternalServerError().finish());
	}

	// Has Voting Or Admin Perms.
	if update.vote.is_some() && !member.permissions.intersects(Permissions::ADMIN | Permissions::VOTING) {
		return Ok(HttpResponse::InternalServerError().finish());
	}

	EditModel::update_by_id(*edit_id, update, &db).await?;

	Ok(HttpResponse::Ok().finish())
}