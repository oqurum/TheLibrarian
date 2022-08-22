use actix_web::{get, web};
use common::api::{WrappingResponse, QueryListResponse};
use common_local::{api, Member};

use crate::{database::Database, http::{JsonResponse, MemberCookie}, WebResult, model::MemberModel};



// TODO: Add body requests for specifics
#[get("/member")]
pub async fn load_member_self(
    db: web::Data<Database>,
    member: MemberCookie,
) -> WebResult<JsonResponse<api::GetMemberSelfResponse>> {
    let member = member.fetch_or_error(&db).await?;

    Ok(web::Json(WrappingResponse::okay(api::GetMemberSelfResponse {
        member: Some(member.into())
    })))
}


#[get("/members")]
pub async fn get_members(
    query: web::Query<api::SimpleListQuery>,
    db: web::Data<Database>,
    member: MemberCookie,
) -> WebResult<JsonResponse<QueryListResponse<Member>>> {
    let member = member.fetch_or_error(&db).await?;

    if !member.permissions.is_admin() {
        return Ok(web::Json(WrappingResponse::error("Not Admin")));
    }

    let offset = query.offset.unwrap_or_default();
    let limit = query.limit.unwrap_or(50);

    let total = MemberModel::get_count(&db).await?;
    let items = MemberModel::find_all(offset, limit, &db).await?
        .into_iter()
        .map(|v| v.into())
        .collect();

    Ok(web::Json(WrappingResponse::okay(QueryListResponse {
        offset,
        limit,
        total,
        items,
    })))
}