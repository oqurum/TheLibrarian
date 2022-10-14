use actix_web::{get, post, web, HttpRequest, HttpResponse};
use chrono::{NaiveDate, Utc};
use common::{
    api::{ApiErrorResponse, WrappingResponse},
    PersonId, Source, ThumbnailStore,
};
use common_local::api;
use tokio_postgres::Client;

use crate::{
    http::{JsonResponse, MemberCookie},
    metadata,
    model::{
        BookModel, BookPersonModel, NewEditModel, NewPersonModel, PersonAltModel, PersonModel,
    },
    storage::get_storage,
    Error, InternalError, WebResult,
};

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
        let items = PersonModel::search(query, offset, limit, &db)
            .await?
            .into_iter()
            .map(|v| v.into_public_person(None))
            .collect();

        Ok(web::Json(WrappingResponse::okay(api::GetPeopleResponse {
            offset,
            limit,
            total: 0, // TODO
            items,
        })))
    }
    // Return All People
    else {
        let items = PersonModel::get_all(offset, limit, &db)
            .await?
            .into_iter()
            .map(|v| v.into_public_person(None))
            .collect();

        Ok(web::Json(WrappingResponse::okay(api::GetPeopleResponse {
            offset,
            limit,
            total: PersonModel::get_count(&db).await?,
            items,
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
        return Ok(web::Json(WrappingResponse::error(
            "You cannot do this! No Permissions!",
        )));
    }

    if let Some(author) = metadata::get_person_by_source(&source, &db).await? {
        // Download thumbnail
        let thumb_url = if let Some(mut item) = author.cover_image_url {
            item.download(&db).await?;

            item.as_local_value()
                .cloned()
                .unwrap_or(ThumbnailStore::None)
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
        }
        .insert(&db)
        .await?;

        // Insert Person Alt Names
        if let Some(names) = author.other_names {
            for name in names {
                let _ = PersonAltModel {
                    person_id: person.id,
                    name,
                }
                .insert(&db)
                .await;
            }
        }

        Ok(web::Json(WrappingResponse::okay("ok")))
    } else {
        Ok(web::Json(WrappingResponse::error(
            "Unable to find person from source",
        )))
    }
}

// Person
#[get("/person/{id}")]
async fn load_person(
    person_id: web::Path<PersonId>,
    db: web::Data<tokio_postgres::Client>,
) -> WebResult<JsonResponse<api::GetPersonResponse>> {
    let person = PersonModel::get_by_id(*person_id, &db)
        .await?
        .ok_or_else(|| Error::from(InternalError::ItemMissing))?;
    let person_alts = PersonAltModel::find_all_by_person_id(*person_id, &db).await?;

    Ok(web::Json(WrappingResponse::okay(api::GetPersonResponse {
        person: person.into_public_person(None),
        other_names: person_alts.into_iter().map(|v| v.name).collect(),
    })))
}

// Person Thumbnail
#[get("/person/{id}/thumbnail")]
async fn load_person_thumbnail(
    person_id: web::Path<PersonId>,
    req: HttpRequest,
    db: web::Data<tokio_postgres::Client>,
) -> WebResult<HttpResponse> {
    let meta = PersonModel::get_by_id(*person_id, &db).await?;

    if let Some(file_name) = meta.as_ref().and_then(|v| v.thumb_url.as_value()) {
        Ok(get_storage()
            .await
            .get_http_response(file_name, &req)
            .await?)
    } else {
        Ok(HttpResponse::NotFound().finish())
    }
}

// Person Tasks - Update Person, Overwrite Person with another source.
#[post("/person/{id}")]
pub async fn update_person_data(
    person_id: web::Path<PersonId>,
    body: web::Json<api::PostPersonBody>,
    member: MemberCookie,
    db: web::Data<tokio_postgres::Client>,
) -> WebResult<JsonResponse<&'static str>> {
    let person_id = *person_id;

    let member = member.fetch_or_error(&db).await?;

    if !member.permissions.has_editing_perms() {
        return Err(ApiErrorResponse::new("You cannot do this! No Permissions!").into());
    }

    match body.into_inner() {
        api::PostPersonBody::AutoMatchById => (),
        api::PostPersonBody::UpdateBySource(_) => (),

        api::PostPersonBody::Edit(person_edit) => {
            let current_book = PersonModel::get_by_id(person_id, &db).await?;

            if let Some((updated_book, current_book)) = Some(person_edit).zip(current_book) {
                // Make sure we have something we're updating.
                if !updated_book.is_empty() {
                    let model =
                        NewEditModel::from_person_modify(member.id, current_book, updated_book)
                            .await?;

                    if !model.data.is_empty() {
                        model.insert(&db).await?;
                    }
                }
            }
        }

        api::PostPersonBody::CombinePersonWith(into_person_id) => {
            // TODO: Tests for this to ensure it's correct.

            if person_id == into_person_id {
                return Err(
                    ApiErrorResponse::new("You cannot join the same person into itself!").into(),
                );
            }

            let old_person = PersonModel::get_by_id(person_id, &db)
                .await?
                .ok_or_else(|| Error::from(InternalError::ItemMissing))?;
            let mut into_person = PersonModel::get_by_id(into_person_id, &db)
                .await?
                .ok_or_else(|| Error::from(InternalError::ItemMissing))?;

            // Attempt to transfer to other person
            let _ = PersonAltModel::transfer_by_person_id(old_person.id, into_person.id, &db).await;

            // Delete remaining Alt Names
            PersonAltModel::remove_by_person_id(old_person.id, &db).await?;

            // Make Old Person Name an Alt Name
            let _ = PersonAltModel {
                name: old_person.name,
                person_id: into_person.id,
            }
            .insert(&db)
            .await;

            // Transfer Old Person Book to New Person
            let trans_book_person_vec =
                BookPersonModel::find_by_person_id(old_person.id, &db).await?;
            for met_per in &trans_book_person_vec {
                let _ = BookPersonModel {
                    book_id: met_per.book_id,
                    person_id: into_person.id,
                    info: None,
                }
                .insert(&db)
                .await;
            }

            BookPersonModel::remove_by_person_id(old_person.id, &db).await?;

            if into_person.birth_date.is_none() {
                into_person.birth_date = old_person.birth_date;
            }

            if into_person.description.is_none() {
                into_person.description = old_person.description;
            }

            if into_person.thumb_url.is_none() {
                into_person.thumb_url = old_person.thumb_url;
            }

            into_person.updated_at = Utc::now();

            // Update New Person
            into_person.update(&db).await?;

            // Delete Old Person
            PersonModel::remove_by_id(old_person.id, &db).await?;

            // Update book cache author name cache
            for met_per in trans_book_person_vec {
                let person = PersonModel::get_by_id(into_person_id, &db).await?;
                let book = BookModel::get_by_id(met_per.book_id, &db).await?;

                if let Some((person, mut book)) = person.zip(book) {
                    book.cached.author = Some(person.name);
                    book.cached.author_id = Some(person.id);
                    book.update_book(&db).await?;
                }
            }
        }
    }

    Ok(web::Json(WrappingResponse::okay("success")))
}
