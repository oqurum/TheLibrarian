use actix_web::{web, get, post};
use common::api::{WrappingResponse, ApiErrorResponse};
use common_local::{api, update::OptionsUpdate};

use crate::{database::Database, WebResult, http::{JsonResponse, MemberCookie}, config};



#[get("/settings")]
async fn get_settings(member: MemberCookie, db: web::Data<Database>) -> WebResult<JsonResponse<api::GetSettingsResponse>> {
	let member = member.fetch_or_error(&db).await?;

	if !member.permissions.is_admin() {
		return Err(ApiErrorResponse::new("Admin perms needed").into());
	}

	let mut config = config::get_config();

	config.auth.auth_key.clear();
	config.email = None;


	Ok(web::Json(WrappingResponse::okay(api::GetSettingsResponse {
		config,
	})))
}




#[post("/settings")]
async fn update_settings(modify: web::Json<OptionsUpdate>, member: MemberCookie, db: web::Data<Database>) -> WebResult<JsonResponse<String>> {
	let update = modify.into_inner();

	let member = member.fetch_or_error(&db).await?;

	if !member.permissions.is_admin() {
		return Err(ApiErrorResponse::new("Admin perms needed").into());
	}


	let mut config = config::get_config();

	if let Some(value) = update.server_name {
		config.server.name = value;
	}

	if let Some(value) = update.user_signup {
		config.auth.new_users = value
	}

	config::update_config(move |v| { *v = config; Ok(()) })?;
	config::save_config().await?;

	Ok(web::Json(WrappingResponse::okay(String::from("ok"))))
}
