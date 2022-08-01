use std::ops::Neg;

use actix_web::{web, get, post};
use common::api::{QueryListResponse, WrappingResponse};
use common_local::{api, EditId, item::edit::*};

use crate::{database::Database, WebResult, model::{EditModel, BookModel, MemberModel, EditVoteModel, NewEditVoteModel}, http::{MemberCookie, JsonResponse}};


// Get List Of Edits
#[get("/edits")]
pub async fn load_edit_list(
	db: web::Data<Database>,
	this_member: Option<MemberCookie>,
	query: web::Query<api::SimpleListQuery>,
) -> WebResult<JsonResponse<api::GetEditListResponse>> {
	let offset = query.offset.unwrap_or(0);
	let limit = query.offset.unwrap_or(25);

	let mut existing_books: Vec<BookModel> = Vec::new();
	let mut existing_members: Vec<MemberModel> = Vec::new();

	let mut items = Vec::new();

	for item in EditModel::find_by_status(offset, limit, None, None, &db).await? {
		let member = if let Some(v) = existing_members.iter().find(|v| v.id == item.member_id).cloned() {
			Some(v)
		} else if let Some(v) = MemberModel::get_by_id(item.member_id, &db).await? {
			existing_members.push(v.clone());
			Some(v)
		} else {
			None
		};


		let mut item = item.into_shared_edit(member)?;

		let my_vote = if let Some(this_member) = this_member.as_ref() {
			// If we've voted, return our vote in there.
			EditVoteModel::find_one(item.id, this_member.member_id(), &db).await?
				.map(|v| vec![SharedEditVoteModel::from(v)])
				.unwrap_or_default()
		} else {
			Vec::new()
		};

		// Total amount of votes casted.
		item.votes = Some(QueryListResponse {
			offset: 0,
			limit: 1,
			items: my_vote,
			total: EditVoteModel::count_by_edit_id(item.id, &db).await?,
		});

		// Only add `current` field if status is pending.
		if item.status.is_pending() {
			// Attempt to get the Book Model ID we're editing.
			if let Some(ModelIdGroup::Book(book_id)) = item.get_model_id() {
				if let EditData::Book(book_data) = &mut item.data {
					// If we've already queried the database for this book id, clone it.
					if let Some(book_model) = existing_books.iter().find(|v| v.id == book_id).cloned() {
						book_data.current = Some(book_model.into());
					}
					// Query database for book id.
					else if let Some(book_model) = BookModel::get_by_id(book_id, &db).await? {
						existing_books.push(book_model.clone());
						book_data.current = Some(book_model.into());
					}
				}
			}
		}

		items.push(item);
	}

	Ok(web::Json(WrappingResponse::okay(api::GetEditListResponse {
		offset,
		limit,
		total: EditModel::get_count(&db).await?,
		items
	})))
}


// Edit
#[get("/edit/{id}")]
async fn load_edit(edit_id: web::Path<EditId>, db: web::Data<Database>) -> WebResult<JsonResponse<api::GetEditResponse>> {
	let model = EditModel::get_by_id(*edit_id, &db).await?.unwrap();

	let member = MemberModel::get_by_id(model.member_id, &db).await?;

	Ok(web::Json(WrappingResponse::okay(api::GetEditResponse {
		model: model.into_shared_edit(member)?
	})))
}


#[post("/edit/{id}")]
async fn update_edit(
	edit_id: web::Path<EditId>,
	json: web::Json<UpdateEditModel>,
	member: MemberCookie,
	db: web::Data<Database>
) -> WebResult<JsonResponse<api::PostEditResponse>> {
	let mut update = json.into_inner();

	update.ended_at = None;
	update.expires_at = None;
	update.is_applied = None;


	let mut edit_model = match EditModel::get_by_id(*edit_id, &db).await? {
		Some(value) => value,
		_ => return Ok(web::Json(WrappingResponse::error("Unable to find Edit Model."))),
	};


	// Check that we're pending.
	// TODO: Add Admin Override.
	if !edit_model.status.is_pending() {
		return Ok(web::Json(WrappingResponse::error("Edit Model is not currently pending!")));
	}


	let member = MemberModel::get_by_id(member.member_id(), &db).await?.unwrap();

	let member_is_admin = member.permissions.is_admin();

	// Only an Admin can change the status.
	if let Some(new_status) = update.status {
		if !member_is_admin {
			return Ok(web::Json(WrappingResponse::error("You cannot do this! Not Admin!")));
		}

		edit_model.process_status_change(new_status, &db).await?;
	}

	// Has Voting Or Admin Perms.
	let vote_model = if let Some(vote_amount) = update.vote.as_mut() {
		match *vote_amount {
			0 => return Ok(web::Json(WrappingResponse::error("You cannot do this! Invalid Vote!"))),

			i32::MIN..=-1 => {
				*vote_amount = -1;
			}

			1..=i32::MAX => {
				*vote_amount = 1;
			}
		}

		if !member.permissions.has_voting_perms() {
			return Ok(web::Json(WrappingResponse::error("You cannot do this! No Permissions!")));
		}

		if let Some(mut vote_model) = EditVoteModel::find_one(*edit_id, member.id, &db).await? {
			let model_vote_as_num = if vote_model.vote { 1 } else { -1 };

			// Remove Vote.
			if model_vote_as_num == *vote_amount {
				EditVoteModel::remove(
					*edit_id,
					member.id,
					&db
				).await?;

				// Opposite vote_amount value
				*vote_amount = vote_amount.neg();
			}

			// Double the value since we're switching ie: Total Votes = 10, Going from true -> false which means we have to go minus 2 votes.
			else {
				vote_model.vote = *vote_amount == 1;
				vote_model.update(&db).await?;

				*vote_amount *= 2;
			}

			Some(vote_model)
		} else {
			let vote_model = NewEditVoteModel::create(
				*edit_id,
				member.id,
				*vote_amount == 1,
			);

			Some(vote_model.insert(&db).await?)
		}
	} else {
		None
	};

	EditModel::update_by_id(*edit_id, update, &db).await?;

	Ok(web::Json(WrappingResponse::okay(api::PostEditResponse {
		edit_model: Some(edit_model.into_shared_edit(Some(member))?),

		vote: vote_model.map(|v| {
			let mut shared = SharedEditVoteModel::from(v);

			if !member_is_admin {
				shared.member_id = None;
			}

			shared
		}),
	})))
}