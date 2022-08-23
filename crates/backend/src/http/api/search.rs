use actix_web::{get, web};
use common::api::{WrappingResponse, QueryListResponse};
use common_local::{api, SearchGroup};

use crate::{http::{MemberCookie, JsonResponse}, WebResult, model::SearchGroupModel, Database};




#[get("/searches")]
pub async fn get_searches(
    query: web::Query<api::SimpleListQuery>,
    db: web::Data<Database>,
    member: MemberCookie,
) -> WebResult<JsonResponse<QueryListResponse<SearchGroup>>> {
    let member = member.fetch_or_error(&db).await?;

    if !member.permissions.is_admin() {
        return Ok(web::Json(WrappingResponse::error("Not Admin")));
    }

    let offset = query.offset.unwrap_or_default();
    let limit = query.limit.unwrap_or(50);

    let total = SearchGroupModel::get_count(&db).await?;
    let items = SearchGroupModel::find_all(offset, limit, &db).await?
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