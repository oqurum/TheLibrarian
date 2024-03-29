use std::io::{Cursor, Write};

use actix_web::{get, post, web, HttpRequest, HttpResponse};
use chrono::Utc;
use common::{api::WrappingResponse, BookId, Either, ImageIdType, ImageType, PersonId};
use common_local::{api, Poster};
use futures::TryStreamExt;

use crate::{
    http::{JsonResponse, MemberCookie},
    model::{BookModel, ImageLinkModel, PersonModel, UploadedImageModel},
    storage::get_storage,
    store_image, Error, InternalError, WebResult,
};

#[get("/image/{id}")]
async fn get_local_image(id: web::Path<String>, req: HttpRequest) -> WebResult<HttpResponse> {
    Ok(get_storage().await.get_http_response(&id, &req).await?)
}

#[get("/posters/{id}")]
async fn get_poster_list(
    query: web::Query<api::GetPostersQuery>,
    image: web::Path<ImageIdType>,
    member: MemberCookie,
    db: web::Data<tokio_postgres::Client>,
) -> WebResult<JsonResponse<api::GetPostersResponse>> {
    let member = member.fetch_or_error(&db).await?;

    let mut items = Vec::new();

    let current_thumb = match image.type_of {
        ImageType::Book => {
            let book = BookModel::get_by_id(BookId::from(image.id), &db)
                .await?
                .ok_or_else(|| Error::from(InternalError::ItemMissing))?;

            if query.search_metadata {
                if !member.permissions.has_editing_perms() {
                    return Ok(web::Json(WrappingResponse::error(
                        "You cannot do this! No Permissions!",
                    )));
                }

                let search = crate::metadata::search_all_agents(
                    &format!(
                        "{} {}",
                        book.title
                            .as_deref()
                            .or(book.title.as_deref())
                            .unwrap_or_default(),
                        book.cached.author.as_deref().unwrap_or_default(),
                    ),
                    common_local::SearchFor::Book(common_local::SearchForBooksBy::Query),
                    &db,
                )
                .await?;

                for item in search.0.into_values().flatten() {
                    if let crate::metadata::SearchItem::Book(item) = item {
                        for path in item
                            .thumb_locations
                            .into_iter()
                            .filter_map(|v| v.into_url_value())
                        {
                            items.push(Poster {
                                id: None,

                                selected: false,
                                path,

                                created_at: Utc::now(),
                            });
                        }
                    }
                }
            }

            book.thumb_path
        }
        ImageType::Person => {
            PersonModel::get_by_id(PersonId::from(image.id), &db)
                .await?
                .ok_or_else(|| Error::from(InternalError::ItemMissing))?
                .thumb_url
        }
    };

    let mut stored = ImageLinkModel::find_by_linked_id_w_image(image.id, image.type_of, &db)
        .await?
        .into_iter()
        .map(|poster| Poster {
            id: Some(poster.image_id),

            selected: poster.path == current_thumb,

            path: format!("/api/v1/image/{}", poster.path.into_value().unwrap()),

            created_at: poster.created_at,
        })
        .collect::<Vec<_>>();

    stored.append(&mut items);

    Ok(web::Json(WrappingResponse::okay(api::GetPostersResponse {
        items: stored,
    })))
}

#[post("/posters/{id}")]
async fn post_change_poster(
    image: web::Path<ImageIdType>,
    body: web::Json<api::ChangePosterBody>,
    member: MemberCookie,
    db: web::Data<tokio_postgres::Client>,
) -> WebResult<JsonResponse<&'static str>> {
    let member = member.fetch_or_error(&db).await?;

    if !member.permissions.has_editing_perms() {
        return Ok(web::Json(WrappingResponse::error(
            "You cannot do this! No Permissions!",
        )));
    }

    match image.type_of {
        ImageType::Book => {
            let mut book = BookModel::get_by_id(BookId::from(image.id), &db)
                .await?
                .ok_or_else(|| Error::from(InternalError::ItemMissing))?;

            match body.into_inner().url_or_id {
                Either::Left(url) => {
                    let resp = reqwest::get(url)
                        .await
                        .map_err(Error::from)?
                        .bytes()
                        .await
                        .map_err(Error::from)?;

                    let image_model = store_image(resp.to_vec(), &db).await?;

                    book.thumb_path = image_model.path;

                    ImageLinkModel::new_book(image_model.id, book.id)
                        .insert(&db)
                        .await?;
                }

                Either::Right(id) => {
                    let poster = UploadedImageModel::get_by_id(id, &db)
                        .await?
                        .ok_or_else(|| Error::from(InternalError::ItemMissing))?;

                    if book.thumb_path == poster.path {
                        return Ok(web::Json(WrappingResponse::okay("poster already set")));
                    }

                    book.thumb_path = poster.path;
                }
            }

            book.update_book(&db).await?;
        }

        ImageType::Person => {
            let mut person = PersonModel::get_by_id(PersonId::from(image.id), &db)
                .await?
                .ok_or_else(|| Error::from(InternalError::ItemMissing))?;

            match body.into_inner().url_or_id {
                Either::Left(url) => {
                    let resp = reqwest::get(url)
                        .await
                        .map_err(Error::from)?
                        .bytes()
                        .await
                        .map_err(Error::from)?;

                    let image_model = store_image(resp.to_vec(), &db).await?;

                    person.thumb_url = image_model.path;

                    ImageLinkModel::new_person(image_model.id, person.id)
                        .insert(&db)
                        .await?;
                }

                Either::Right(id) => {
                    let poster = UploadedImageModel::get_by_id(id, &db)
                        .await?
                        .ok_or_else(|| Error::from(InternalError::ItemMissing))?;

                    if person.thumb_url == poster.path {
                        return Ok(web::Json(WrappingResponse::okay("poster already set")));
                    }

                    person.thumb_url = poster.path;
                }
            }

            person.update(&db).await?;
        }
    }

    Ok(web::Json(WrappingResponse::okay("success")))
}

#[post("/posters/{id}/upload")]
async fn post_upload_poster(
    image: web::Path<ImageIdType>,
    mut body: web::Payload,
    member: MemberCookie,
    db: web::Data<tokio_postgres::Client>,
) -> WebResult<JsonResponse<&'static str>> {
    let member = member.fetch_or_error(&db).await?;

    if !member.permissions.has_editing_perms() {
        return Ok(web::Json(WrappingResponse::error(
            "You cannot do this! No Permissions!",
        )));
    }

    match image.type_of {
        ImageType::Book => {
            let book = BookModel::get_by_id(BookId::from(image.id), &db)
                .await?
                .ok_or_else(|| Error::from(InternalError::ItemMissing))?;

            let mut file = Cursor::new(Vec::new());

            while let Some(item) = body.try_next().await? {
                file.write_all(&item).map_err(Error::from)?;
            }

            let image_model = store_image(file.into_inner(), &db).await?;

            ImageLinkModel::new_book(image_model.id, book.id)
                .insert(&db)
                .await?;
        }

        ImageType::Person => {
            let person = PersonModel::get_by_id(PersonId::from(image.id), &db)
                .await?
                .ok_or_else(|| Error::from(InternalError::ItemMissing))?;

            let mut file = Cursor::new(Vec::new());

            while let Some(item) = body.try_next().await? {
                file.write_all(&item).map_err(Error::from)?;
            }

            let image_model = store_image(file.into_inner(), &db).await?;

            ImageLinkModel::new_person(image_model.id, person.id)
                .insert(&db)
                .await?;
        }
    }

    Ok(web::Json(WrappingResponse::okay("success")))
}
