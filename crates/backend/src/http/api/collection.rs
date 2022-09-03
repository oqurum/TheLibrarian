use actix_web::{get, post, web};
use common::api::WrappingResponse;
use common_local::{api, util::parse_num_description_string, CollectionId};

use crate::{WebResult, http::{JsonResponse, MemberCookie}, Database, model::{CollectionModel, NewCollectionModel}};


#[get("/collection/{id}")]
async fn get_collection_by_id(
    coll_id: web::Path<String>,
    db: web::Data<Database>
) -> WebResult<JsonResponse<api::GetCollectionResponse>> {
    let coll_id = parse_num_description_string::<CollectionId>(&coll_id).map_err(crate::Error::from)?;

    Ok(web::Json(WrappingResponse::okay(api::GetCollectionResponse {
        value: CollectionModel::find_by_id(coll_id, &db).await?.map(|v| v.into()),
    })))
}


#[post("/collection/{id}")]
async fn update_collection_by_id(
    coll_id: web::Path<String>,
    body: web::Json<api::UpdateCollectionModel>,
    member: MemberCookie,
    db: web::Data<Database>
) -> WebResult<JsonResponse<&'static str>> {
    let coll_id = parse_num_description_string::<CollectionId>(&coll_id).map_err(crate::Error::from)?;

    let member = member.fetch(&db).await?.unwrap();

    if !member.permissions.has_editing_perms() {
        return Ok(web::Json(WrappingResponse::error("You cannot do this! No Permissions!")));
    }

    CollectionModel::update_by_id(coll_id, body.into_inner(), &db).await?;

    Ok(web::Json(WrappingResponse::okay("ok")))
}


#[post("/collection")]
async fn create_new_collection(
    body: web::Json<api::NewCollectionBody>,
    member: MemberCookie,
    db: web::Data<Database>
) -> WebResult<JsonResponse<api::NewCollectionResponse>> {
    let member = member.fetch(&db).await?.unwrap();

    if !member.permissions.has_editing_perms() {
        return Ok(web::Json(WrappingResponse::error("You cannot do this! No Permissions!")));
    }

    let api::NewCollectionBody { name, description, type_of } = body.into_inner();

    let model = NewCollectionModel { name, description, type_of };

    Ok(web::Json(WrappingResponse::okay(model.insert(&db).await?.into())))
}