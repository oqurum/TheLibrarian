use actix_web::{web, get, HttpResponse};
use common::{PersonId, api::WrappingResponse};
use common_local::api;

use crate::{database::Database, WebResult, Error, model::PersonModel, http::JsonResponse};


// Get List Of People and Search For People
#[get("/people")]
pub async fn load_author_list(
	db: web::Data<Database>,
	query: web::Query<api::SimpleListQuery>,
) -> WebResult<JsonResponse<api::GetPeopleResponse>> {
	let offset = query.offset.unwrap_or(0);
	let limit = query.offset.unwrap_or(50);

	// Return Searched People
	if let Some(query) = query.query.as_deref() {
		let items = PersonModel::search(query, offset, limit, &db).await?
			.into_iter()
			.map(|v| v.into())
			.collect();

		Ok(web::Json(WrappingResponse::okay(api::GetPeopleResponse {
			offset,
			limit,
			total: 0, // TODO
			items
		})))
	}

	// Return All People
	else {
		let items = PersonModel::get_all(offset, limit, &db).await?
			.into_iter()
			.map(|v| v.into())
			.collect();

		Ok(web::Json(WrappingResponse::okay(api::GetPeopleResponse {
			offset,
			limit,
			total: PersonModel::get_count(&db).await?,
			items
		})))
	}
}


// Person
#[get("/person/{id}")]
async fn load_person(person_id: web::Path<PersonId>, db: web::Data<Database>) -> WebResult<JsonResponse<api::GetPersonResponse>> {
	let person = PersonModel::get_by_id(*person_id, &db).await?.unwrap();

	Ok(web::Json(WrappingResponse::okay(api::GetPersonResponse {
		person: person.into()
	})))
}


// Person Thumbnail
#[get("/person/{id}/thumbnail")]
async fn load_person_thumbnail(person_id: web::Path<PersonId>, db: web::Data<Database>) -> WebResult<HttpResponse> {
	let meta = PersonModel::get_by_id(*person_id, &db).await?;

	if let Some(loc) = meta.map(|v| v.thumb_url) {
		let path = crate::image::hash_to_path(loc.as_value());

		Ok(HttpResponse::Ok().body(std::fs::read(path).map_err(Error::from)?))
	} else {
		Ok(HttpResponse::NotFound().finish())
	}
}