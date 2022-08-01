use actix_identity::Identity;
use actix_web::{get, web};
use common::api::WrappingResponse;
use librarian_common::api;

use crate::{database::Database, http::{get_auth_value, JsonResponse}, WebResult, model::MemberModel};



// TODO: Add body requests for specifics
#[get("/member")]
pub async fn load_member_self(
	db: web::Data<Database>,
	identity: Identity,
) -> WebResult<JsonResponse<api::GetMemberSelfResponse>> {
	if let Some(cookie) = get_auth_value(&identity) {
		if let Some(member) = MemberModel::get_by_id(cookie.member_id, &db).await? {
			return Ok(web::Json(WrappingResponse::okay(api::GetMemberSelfResponse {
				member: Some(member.into())
			})));
		}
	}

	Ok(web::Json(WrappingResponse::okay(api::GetMemberSelfResponse {
		member: None
	})))
}