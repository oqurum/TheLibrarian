use actix_web::{web, get, HttpResponse, HttpRequest, post};
use chrono::{Utc, NaiveDate};
use common::{PersonId, api::WrappingResponse, Source, ThumbnailStore};
use common_local::api;
use tokio_postgres::Client;

use crate::{WebResult, model::{PersonModel, NewPersonModel, PersonAltModel}, http::{JsonResponse, MemberCookie}, storage::get_storage, metadata, InternalError, Error};


// Get List Of People and Search For People
#[get("/people")]
pub async fn load_author_list(
    db: web::Data<tokio_postgres::Client>,
    query: web::Query<api::SimpleListQuery>,
) -> WebResult<JsonResponse<api::GetPeopleResponse>> {
    let offset = query.offset.unwrap_or(0);
    let limit = query.offset.unwrap_or(50);

    // Return Searched People
    if let Some(query) = query.query.as_deref() {
        let items = PersonModel::search(query, offset, limit, &db).await?
            .into_iter()
            .map(|v| v.into())
            .collect();

        Ok(web::Json(WrappingResponse::okay(api::GetPeopleResponse {
            offset,
            limit,
            total: 0, // TODO
            items
        })))
    }

    // Return All People
    else {
        let items = PersonModel::get_all(offset, limit, &db).await?
            .into_iter()
            .map(|v| v.into())
            .collect();

        Ok(web::Json(WrappingResponse::okay(api::GetPeopleResponse {
            offset,
            limit,
            total: PersonModel::get_count(&db).await?,
            items
        })))
    }
}


#[post("/person")]
pub async fn add_new_person(
    source: web::Json<Source>,
    member: MemberCookie,
    db: web::Data<Client>,
) -> WebResult<JsonResponse<&'static str>> {
    let member = member.fetch_or_error(&db).await?;

    if !member.permissions.has_editing_perms() {
        return Ok(web::Json(WrappingResponse::error("You cannot do this! No Permissions!")));
    }

    if let Some(author) = metadata::get_person_by_source(&source, &db).await? {
        // Download thumbnail
        let thumb_url = if let Some(mut item) = author.cover_image_url {
            item.download(&db).await?;

            item.as_local_value().cloned().unwrap_or(ThumbnailStore::None)
        } else {
            ThumbnailStore::None
        };

        // Insert Person
        let person = NewPersonModel {
            source: source.into_inner(),
            thumb_url,
            name: author.name,
            description: author.description,
            birth_date: author.birth_date.and_then(|v| v.parse::<NaiveDate>().ok()),
            updated_at: Utc::now(),
            created_at: Utc::now(),
        }.insert(&db).await?;

        // Insert Person Alt Names
        if let Some(names) = author.other_names {
            for name in names {
                let _ = PersonAltModel {
                    person_id: person.id,
                    name,
                }.insert(&db).await;
            }
        }

        Ok(web::Json(WrappingResponse::okay("ok")))
    } else {
        Ok(web::Json(WrappingResponse::error("Unable to find person from source")))
    }

}





// Person
#[get("/person/{id}")]
async fn load_person(person_id: web::Path<PersonId>, db: web::Data<tokio_postgres::Client>) -> WebResult<JsonResponse<api::GetPersonResponse>> {
    let person = PersonModel::get_by_id(*person_id, &db).await?.ok_or_else(|| Error::from(InternalError::ItemMissing))?;

    Ok(web::Json(WrappingResponse::okay(api::GetPersonResponse {
        person: person.into()
    })))
}


// Person Thumbnail
#[get("/person/{id}/thumbnail")]
async fn load_person_thumbnail(person_id: web::Path<PersonId>, req: HttpRequest, db: web::Data<tokio_postgres::Client>) -> WebResult<HttpResponse> {
    let meta = PersonModel::get_by_id(*person_id, &db).await?;

    if let Some(file_name) = meta.as_ref().and_then(|v| v.thumb_url.as_value()) {
        Ok(get_storage().await.get_http_response(file_name, &req).await?)
    } else {
        Ok(HttpResponse::NotFound().finish())
    }
}