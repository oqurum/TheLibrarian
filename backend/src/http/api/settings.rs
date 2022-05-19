use actix_web::{web, get};
use librarian_common::api;

use crate::{database::Database, WebResult};



#[get("/settings")]
async fn get_settings(_db: web::Data<Database>) -> WebResult<web::Json<api::GetSettingsResponse>> {
	Ok(web::Json(api::GetSettingsResponse {
		//
	}))
}
