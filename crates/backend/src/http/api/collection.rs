use actix_web::{get, post, web};
use common::api::WrappingResponse;
use common_local::{api, util::parse_num_description_string, CollectionId, DisplayItem};

use crate::{
    http::{JsonResponse, MemberCookie},
    model::{CollectionModel, NewCollectionModel},
    WebResult,
};

#[get("/collection/{id}")]
async fn get_collection_by_id(
    coll_id: web::Path<String>,
    db: web::Data<tokio_postgres::Client>,
) -> WebResult<JsonResponse<api::GetCollectionResponse>> {
    let coll_id =
        parse_num_description_string::<CollectionId>(&coll_id).map_err(crate::Error::from)?;

    Ok(web::Json(WrappingResponse::okay(
        api::GetCollectionResponse {
            value: CollectionModel::find_by_id(coll_id, &db)
                .await?
                .map(|v| v.into()),
        },
    )))
}

#[post("/collection/{id}")]
async fn update_collection_by_id(
    coll_id: web::Path<String>,
    body: web::Json<api::UpdateCollectionModel>,
    member: MemberCookie,
    db: web::Data<tokio_postgres::Client>,
) -> WebResult<JsonResponse<&'static str>> {
    let coll_id =
        parse_num_description_string::<CollectionId>(&coll_id).map_err(crate::Error::from)?;

    let member = member.fetch_or_error(&db).await?;

    if !member.permissions.has_editing_perms() {
        return Ok(web::Json(WrappingResponse::error(
            "You cannot do this! No Permissions!",
        )));
    }

    CollectionModel::update_by_id(coll_id, body.into_inner(), &db).await?;

    Ok(web::Json(WrappingResponse::okay("ok")))
}

#[get("/collection/{id}/books")]
async fn get_collection_books_by_id(
    coll_id: web::Path<String>,
    db: web::Data<tokio_postgres::Client>,
) -> WebResult<JsonResponse<api::GetBookListResponse>> {
    let coll_id =
        parse_num_description_string::<CollectionId>(&coll_id).map_err(crate::Error::from)?;

    let books = CollectionModel::find_books_by_id(coll_id, &db).await?;

    Ok(web::Json(WrappingResponse::okay(
        api::GetBookListResponse {
            count: books.len(),
            items: books
                .into_iter()
                .map(|meta| DisplayItem {
                    id: meta.id,
                    title: meta.title.or(meta.clean_title).unwrap_or_default(),
                    cached: meta.cached,
                    has_thumbnail: meta.thumb_path.is_some(),
                })
                .collect(),
        },
    )))
}

#[get("/collections")]
async fn get_collection_list(
    query: web::Query<api::SimpleListQuery>,
    db: web::Data<tokio_postgres::Client>,
) -> WebResult<JsonResponse<api::GetCollectionListResponse>> {
    let limit = query.limit.unwrap_or(50);
    let offset = query.offset.unwrap_or(0);

    let total = CollectionModel::count(None, None, &db).await?;

    let items = CollectionModel::search(query.query.as_deref(), offset, limit, None, &db).await?;

    Ok(web::Json(WrappingResponse::okay(
        api::GetCollectionListResponse {
            offset: 0,
            limit: 0,
            total,
            items: items.into_iter().map(|v| v.into()).collect(),
        },
    )))
}

#[post("/collection")]
async fn create_new_collection(
    body: web::Json<api::NewCollectionBody>,
    member: MemberCookie,
    db: web::Data<tokio_postgres::Client>,
) -> WebResult<JsonResponse<api::NewCollectionResponse>> {
    let member = member.fetch_or_error(&db).await?;

    if !member.permissions.has_editing_perms() {
        return Ok(web::Json(WrappingResponse::error(
            "You cannot do this! No Permissions!",
        )));
    }

    let api::NewCollectionBody {
        name,
        description,
        type_of,
    } = body.into_inner();

    let model = NewCollectionModel {
        name,
        description,
        type_of,
    };

    Ok(web::Json(WrappingResponse::okay(
        model.insert(&db).await?.into(),
    )))
}
