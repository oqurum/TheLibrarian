use actix_web::{get, post, web};
use common::api::{ApiErrorResponse, WrappingResponse};
use common_local::{api, update::OptionsUpdate};

use crate::{
    config,
    http::{JsonResponse, MemberCookie},
    WebResult,
};

#[get("/settings")]
async fn get_settings(
    member: MemberCookie,
    db: web::Data<tokio_postgres::Client>,
) -> WebResult<JsonResponse<api::GetSettingsResponse>> {
    let member = member.fetch_or_error(&db).await?;

    if !member.permissions.is_admin() {
        return Err(ApiErrorResponse::new("Admin perms needed").into());
    }

    let mut config = config::get_config();

    config.auth.auth_key.clear();
    config.email = None;

    Ok(web::Json(WrappingResponse::okay(
        api::GetSettingsResponse {
            config: config.into(),
        },
    )))
}

#[post("/settings")]
async fn update_settings(
    modify: web::Json<OptionsUpdate>,
    member: MemberCookie,
    db: web::Data<tokio_postgres::Client>,
) -> WebResult<JsonResponse<&'static str>> {
    let update = modify.into_inner();

    let mut member = member.fetch_or_error(&db).await?;

    // Member Changes
    if let Some(settings) = update.member {
        member.set_settings(settings)?;
        member.update(&db).await?;

        return Ok(web::Json(WrappingResponse::okay("ok")));
    }

    // Admin Changes
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

    config::update_config(move |v| {
        *v = config;
        Ok(())
    })?;
    config::save_config().await?;

    Ok(web::Json(WrappingResponse::okay("ok")))
}
