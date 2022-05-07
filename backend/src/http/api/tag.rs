use actix_web::{get, web, post};
use librarian_common::api;

use crate::{database::Database, WebResult};




// Tags

#[get("/tag/{id}")]
async fn get_tag_by_id(
	tag_id: web::Path<usize>,
	db: web::Data<Database>
) -> WebResult<web::Json<api::GetTagResponse>> {
	Ok(web::Json(api::GetTagResponse {
		value: db.get_tag_by_id(*tag_id)?.unwrap().into(),
	}))
}

#[post("/tag")]
async fn create_new_tag(
	body: web::Json<api::NewTagBody>,
	db: web::Data<Database>
) -> WebResult<web::Json<api::NewTagResponse>> {
	Ok(web::Json(api::NewTagResponse {
		id: db.add_tag(&body.name, body.type_of)?,
	}))
}






// Book Tags

// #[get("/tags/book/{id}")]
// async fn get_tags_for_book_id(
// 	book_id: web::Path<usize>,
// 	db: web::Data<Database>
// ) -> WebResult<web::Json<api::GetBookTagsResponse>> {
// 	Ok(web::Json(api::GetBookTagsResponse {
// 		items: db.get_book_tags(*book_id)?
// 			.into_iter()
// 			.map(|v| v.into())
// 			.collect(),
// 	}))
// }

// #[post("/tags/book/{id}")]
// async fn add_tag_for_book(
// 	body: web::Json<api::NewBookTagBody>,
// 	db: web::Data<Database>
// ) -> WebResult<web::Json<api::NewBookTagResponse>> {
// 	let tag_id = db.add_tag(&body.name, body.type_of)?;

// 	Ok(web::Json(api::GetTagResponse {
// 		value: db.get_tag_by_id(tag_id)?.unwrap().into(),
// 	}))
// }


