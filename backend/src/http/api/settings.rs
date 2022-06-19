use actix_web::{web, get};
use librarian_common::api;

use crate::{database::Database, WebResult, http::JsonResponse};



#[get("/settings")]
async fn get_settings(_db: web::Data<Database>) -> WebResult<JsonResponse<api::GetSettingsResponse>> {
	Ok(web::Json(api::WrappingResponse::new(api::GetSettingsResponse {
		//
	})))
}
