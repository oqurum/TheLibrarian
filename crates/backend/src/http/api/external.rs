use actix_web::{get, web};
use common::{
    api::{ApiErrorResponse, WrappingResponse},
    Source,
};
use common_local::{api, SearchFor, SearchForBooksBy, SearchType};

use crate::{
    http::{JsonResponse, MemberCookie},
    metadata, WebResult,
};

#[get("/external/search")]
pub async fn get_external_search(
    body: web::Query<api::GetMetadataSearch>,
    member: MemberCookie,
    db: web::Data<tokio_postgres::Client>,
) -> WebResult<JsonResponse<api::ExternalSearchResponse>> {
    let member = member.fetch_or_error(&db).await?;

    if !member.permissions.has_editing_perms() {
        return Err(ApiErrorResponse::new("You cannot do this! No Permissions!").into());
    }

    let search = metadata::search_all_agents(
        &body.query,
        match body.search_type {
            // TODO: Allow for use in Query.
            SearchType::Book => SearchFor::Book(SearchForBooksBy::Query),
            SearchType::Person => SearchFor::Person,
        },
        &db,
    )
    .await?;

    Ok(web::Json(WrappingResponse::okay(
        api::ExternalSearchResponse {
            items: search
                .0
                .into_iter()
                .map(|(a, b)| {
                    (
                        a,
                        b.into_iter()
                            .map(|v| match v {
                                metadata::SearchItem::Book(book) => {
                                    api::SearchItem::Book(api::MetadataBookSearchItem {
                                        source: book.source,
                                        author: book.cached.author,
                                        description: book.description,
                                        name: book
                                            .title
                                            .unwrap_or_else(|| String::from("Unknown title")),
                                        thumbnail_url: book
                                            .thumb_locations
                                            .first()
                                            .and_then(|v| v.as_url_value())
                                            .map(|v| v.to_string())
                                            .unwrap_or_default(),
                                    })
                                }

                                metadata::SearchItem::Author(author) => {
                                    api::SearchItem::Person(api::MetadataPersonSearchItem {
                                        source: author.source,

                                        cover_image: author
                                            .cover_image_url
                                            .map(|v| v.as_api_path().into_owned()),

                                        name: author.name,
                                        other_names: author.other_names,
                                        description: author.description,

                                        birth_date: author.birth_date,
                                        death_date: author.death_date,
                                    })
                                }
                            })
                            .collect(),
                    )
                })
                .collect(),
        },
    )))
}

#[get("/external/{source}")]
pub async fn get_external_item(
    path: web::Path<Source>,
    member: MemberCookie,
    db: web::Data<tokio_postgres::Client>,
) -> WebResult<JsonResponse<api::ExternalSourceItemResponse>> {
    let member = member.fetch_or_error(&db).await?;

    if !member.permissions.has_editing_perms() {
        return Err(ApiErrorResponse::new("You cannot do this! No Permissions!").into());
    }

    if let Some(meta) = metadata::get_metadata_by_source(&*path, true, &db).await? {
        Ok(web::Json(WrappingResponse::okay(
            api::ExternalSourceItemResponse {
                item: Some(meta.meta.into()),
            },
        )))
    } else {
        Ok(web::Json(WrappingResponse::okay(
            api::ExternalSourceItemResponse { item: None },
        )))
    }
}
