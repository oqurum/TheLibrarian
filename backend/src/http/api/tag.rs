use actix_web::{get, web, post, delete};
use librarian_common::api;

use crate::{database::Database, WebResult};




// Tags

#[get("/tag/{id}")]
async fn get_tag_by_id(
	tag_id: web::Path<usize>,
	db: web::Data<Database>
) -> WebResult<web::Json<api::GetTagResponse>> {
	Ok(web::Json(api::GetTagResponse {
		value: db.get_tag_by_id(*tag_id)?.map(|v| v.into()),
	}))
}

#[post("/tag")]
async fn create_new_tag(
	body: web::Json<api::NewTagBody>,
	db: web::Data<Database>
) -> WebResult<web::Json<api::NewTagResponse>> {
	let body = body.into_inner();

	Ok(web::Json(api::NewTagResponse {
		id: db.add_tag(&body.name, body.type_of)?,
	}))
}

#[get("/tags")]
async fn get_tags(db: web::Data<Database>) -> WebResult<web::Json<api::GetTagsResponse>> {
	Ok(web::Json(api::GetTagsResponse {
		items: db.get_tags()?
			.into_iter()
			.map(|v| v.into())
			.collect(),
	}))
}





// Book Tags

#[get("/tag/{tag_id}/book/{book_id}")]
async fn get_book_tag(
	id: web::Path<(usize, usize)>,
	db: web::Data<Database>
) -> WebResult<web::Json<api::GetBookTagResponse>> {
	Ok(web::Json(api::GetBookTagResponse {
		value: db.get_book_tag_info_by_bid_tid(id.1, id.0)?.map(|v| v.into()),
	}))
}

#[delete("/tag/{tag_id}/book/{book_id}")]
async fn delete_book_tag(
	id: web::Path<(usize, usize)>,
	db: web::Data<Database>
) -> WebResult<web::Json<api::DeletionResponse>> {
	Ok(web::Json(api::DeletionResponse {
		amount: db.remove_book_tag(id.1, id.0)?,
	}))
}


#[get("/tags/book/{id}")]
async fn get_tags_for_book_id(
	book_id: web::Path<usize>,
	db: web::Data<Database>
) -> WebResult<web::Json<api::GetBookTagsResponse>> {
	Ok(web::Json(api::GetBookTagsResponse {
		items: db.get_book_tags_info(*book_id)?
			.into_iter()
			.map(|v| v.into())
			.collect(),
	}))
}


#[post("/tag/book/{id}")]
async fn add_book_tag(
	book_id: web::Path<usize>,
	body: web::Json<api::NewBookTagBody>,
	db: web::Data<Database>
) -> WebResult<web::Json<api::NewBookTagResponse>> {
	Ok(web::Json(api::NewBookTagResponse {
		id: db.add_book_tag(*book_id, body.tag_id, body.index)?,
	}))
}