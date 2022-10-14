use actix_web::{delete, get, post, web};
use common::{
    api::{DeletionResponse, WrappingResponse},
    BookId, TagId,
};
use common_local::api::{self, NewTagBody};

use crate::{
    http::{JsonResponse, MemberCookie},
    model::{BookTagModel, BookTagWithTagModel, NewTagModel, TagModel},
    WebResult,
};

// Tags

#[get("/tag/{id}")]
async fn get_tag_by_id(
    tag_id: web::Path<TagId>,
    db: web::Data<tokio_postgres::Client>,
) -> WebResult<JsonResponse<api::GetTagResponse>> {
    Ok(web::Json(WrappingResponse::okay(api::GetTagResponse {
        value: TagModel::get_by_id(*tag_id, &db).await?.map(|v| v.into()),
    })))
}

#[post("/tag")]
async fn create_new_tag(
    body: web::Json<api::NewTagBody>,
    member: MemberCookie,
    db: web::Data<tokio_postgres::Client>,
) -> WebResult<JsonResponse<api::NewTagResponse>> {
    let member = member.fetch_or_error(&db).await?;

    if !member.permissions.has_editing_perms() {
        return Ok(web::Json(WrappingResponse::error(
            "You cannot do this! No Permissions!",
        )));
    }

    let NewTagBody { name, type_of } = body.into_inner();

    let model = NewTagModel { name, type_of };

    Ok(web::Json(WrappingResponse::okay(
        model.insert(&db).await?.into(),
    )))
}

#[get("/tags")]
async fn get_tags(
    db: web::Data<tokio_postgres::Client>,
) -> WebResult<JsonResponse<api::GetTagsResponse>> {
    Ok(web::Json(WrappingResponse::okay(api::GetTagsResponse {
        items: TagModel::get_all(&db)
            .await?
            .into_iter()
            .map(|v| v.into())
            .collect(),
    })))
}

// Book Tags

#[get("/tag/{tag_id}/book/{book_id}")]
async fn get_book_tag(
    id: web::Path<(TagId, BookId)>,
    db: web::Data<tokio_postgres::Client>,
) -> WebResult<JsonResponse<api::GetBookTagResponse>> {
    Ok(web::Json(WrappingResponse::okay(api::GetBookTagResponse {
        value: BookTagWithTagModel::get_by_book_id_and_tag_id(id.1, id.0, &db)
            .await?
            .map(|v| v.into()),
    })))
}

#[delete("/tag/{tag_id}/book/{book_id}")]
async fn delete_book_tag(
    id: web::Path<(TagId, BookId)>,
    member: MemberCookie,
    db: web::Data<tokio_postgres::Client>,
) -> WebResult<JsonResponse<DeletionResponse>> {
    let member = member.fetch_or_error(&db).await?;

    if !member.permissions.has_editing_perms() {
        return Ok(web::Json(WrappingResponse::error(
            "You cannot do this! No Permissions!",
        )));
    }

    Ok(web::Json(WrappingResponse::okay(DeletionResponse {
        total: BookTagModel::remove(id.1, id.0, &db).await? as usize,
    })))
}

#[get("/tags/book/{id}")]
async fn get_tags_for_book_id(
    book_id: web::Path<BookId>,
    db: web::Data<tokio_postgres::Client>,
) -> WebResult<JsonResponse<api::GetBookTagsResponse>> {
    Ok(web::Json(WrappingResponse::okay(
        api::GetBookTagsResponse {
            items: BookTagWithTagModel::get_by_book_id(*book_id, &db)
                .await?
                .into_iter()
                .map(|v| v.into())
                .collect(),
        },
    )))
}

#[post("/tag/book/{id}")]
async fn add_book_tag(
    book_id: web::Path<BookId>,
    member: MemberCookie,
    body: web::Json<api::NewBookTagBody>,
    db: web::Data<tokio_postgres::Client>,
) -> WebResult<JsonResponse<api::NewBookTagResponse>> {
    let member = member.fetch_or_error(&db).await?;

    if !member.permissions.has_editing_perms() {
        return Ok(web::Json(WrappingResponse::error(
            "You cannot do this! No Permissions!",
        )));
    }

    Ok(web::Json(WrappingResponse::okay(api::NewBookTagResponse {
        id: BookTagModel::insert(*book_id, body.tag_id, body.index, &db)
            .await?
            .id,
    })))
}
