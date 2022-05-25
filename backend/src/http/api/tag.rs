use actix_web::{get, web, post, delete};
use librarian_common::api::{self, NewTagBody};

use crate::{database::Database, WebResult, model::{TagModel, NewTagModel, BookTagWithTagModel, BookTagModel}};




// Tags

#[get("/tag/{id}")]
async fn get_tag_by_id(
	tag_id: web::Path<usize>,
	db: web::Data<Database>
) -> WebResult<web::Json<api::GetTagResponse>> {
	Ok(web::Json(api::GetTagResponse {
		value: TagModel::get_by_id(*tag_id, &db).await?.map(|v| v.into()),
	}))
}

#[post("/tag")]
async fn create_new_tag(
	body: web::Json<api::NewTagBody>,
	db: web::Data<Database>
) -> WebResult<web::Json<api::NewTagResponse>> {
	let NewTagBody { name, type_of } = body.into_inner();

	let model = NewTagModel { name, type_of };

	Ok(web::Json(model.insert(&db).await?.into()))
}

#[get("/tags")]
async fn get_tags(db: web::Data<Database>) -> WebResult<web::Json<api::GetTagsResponse>> {
	Ok(web::Json(api::GetTagsResponse {
		items: TagModel::get_all(&db).await?
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
		value: BookTagWithTagModel::get_by_book_id_and_tag_id(id.1, id.0, &db).await?.map(|v| v.into()),
	}))
}

#[delete("/tag/{tag_id}/book/{book_id}")]
async fn delete_book_tag(
	id: web::Path<(usize, usize)>,
	db: web::Data<Database>
) -> WebResult<web::Json<api::DeletionResponse>> {
	Ok(web::Json(api::DeletionResponse {
		amount: BookTagModel::remove(id.1, id.0, &db).await?,
	}))
}


#[get("/tags/book/{id}")]
async fn get_tags_for_book_id(
	book_id: web::Path<usize>,
	db: web::Data<Database>
) -> WebResult<web::Json<api::GetBookTagsResponse>> {
	Ok(web::Json(api::GetBookTagsResponse {
		items: BookTagWithTagModel::get_by_book_id(*book_id, &db).await?
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
		id: BookTagModel::insert(*book_id, body.tag_id, body.index, &db).await?.id,
	}))
}