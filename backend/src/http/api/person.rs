use actix_web::{web, get, HttpResponse};
use librarian_common::api;

use crate::{database::{Database}, WebResult, Error};


// Get List Of People and Search For People
#[get("/people")]
pub async fn load_author_list(
	db: web::Data<Database>,
	query: web::Query<api::SimpleListQuery>,
) -> WebResult<web::Json<api::GetPeopleResponse>> {
	let offset = query.offset.unwrap_or(0);
	let limit = query.offset.unwrap_or(50);

	// Return Searched People
	if let Some(query) = query.query.as_deref() {
		let items = db.search_person_list(query, offset, limit)?
			.into_iter()
			.map(|v| v.into())
			.collect();

		Ok(web::Json(api::GetPeopleResponse {
			offset,
			limit,
			total: 0, // TODO
			items
		}))
	}

	// Return All People
	else {
		let items = db.get_person_list(offset, limit)?
			.into_iter()
			.map(|v| v.into())
			.collect();

		Ok(web::Json(api::GetPeopleResponse {
			offset,
			limit,
			total: db.get_person_count()?,
			items
		}))
	}
}


// Person
#[get("/person/{id}")]
async fn load_person(person_id: web::Path<usize>, db: web::Data<Database>) -> WebResult<web::Json<api::GetPersonResponse>> {
	let person = db.get_person_by_id(*person_id)?.unwrap();

	Ok(web::Json(api::GetPersonResponse {
		person: person.into()
	}))
}


// Person Thumbnail
#[get("/person/{id}/thumbnail")]
async fn load_person_thumbnail(person_id: web::Path<usize>, db: web::Data<Database>) -> WebResult<HttpResponse> {
	let meta = db.get_person_by_id(*person_id)?;

	if let Some(loc) = meta.map(|v| v.thumb_url) {
		let path = crate::image::prefixhash_to_path(loc.as_type(), loc.as_value());

		Ok(HttpResponse::Ok().body(std::fs::read(path).map_err(Error::from)?))
	} else {
		Ok(HttpResponse::NotFound().finish())
	}
}