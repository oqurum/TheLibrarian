use actix_web::{get, web, HttpRequest};
use common::{
    api::{
        librarian::{GetSearchQuery, PublicSearchResponse, PublicSearchType},
        QueryListResponse, WrappingResponse,
    },
    BookId, PersonId,
};
use common_local::api::{OrderBy, QueryType};

use crate::{
    model::{
        BookModel, NewSearchGroupModel, NewSearchItemServerModel, PersonAltModel, PersonModel,
        ServerLinkModel,
    },
    Error, WebResult,
};

#[get("/search/book")]
pub async fn public_search_book(
    req: HttpRequest,
    query: web::Query<GetSearchQuery>,
    db: web::Data<tokio_postgres::Client>,
) -> WebResult<web::Json<PublicSearchResponse>> {
    const ID_CHECK: &str = "id:";

    let GetSearchQuery {
        query,
        offset,
        limit,
        view_private,
        server_id,
    } = query.into_inner();

    let sever_link_model = match ServerLinkModel::get_by_server_id(&server_id, &db).await? {
        Some(v) => v,
        None => return Ok(web::Json(WrappingResponse::error("Invalid Server ID"))),
    };

    let host = format!("//{}", req.headers().get("host").unwrap().to_str().unwrap());

    if query.starts_with(ID_CHECK) && query.len() > 3 {
        let book_id = BookId::from(
            query[ID_CHECK.len()..]
                .parse::<usize>()
                .map_err(Error::from)?,
        );

        if let Some(model) = BookModel::get_by_id(book_id, &db).await? {
            let author_ids = PersonModel::get_all_by_book_id(book_id, &db)
                .await?
                .into_iter()
                .map(|v| *v.id)
                .collect();

            Ok(web::Json(WrappingResponse::okay(
                PublicSearchType::BookItem(Some(model.into_public_book(&host, author_ids))),
            )))
        } else {
            Ok(web::Json(WrappingResponse::okay(
                PublicSearchType::BookItem(None),
            )))
        }
    } else {
        let offset = offset.unwrap_or(0);
        let limit = limit.unwrap_or(25);

        let total =
            BookModel::count_search_book(&QueryType::Query(query.clone()), !view_private, &db)
                .await?;

        // Only update if offset is 0.
        if offset == 0 {
            let search_item_updated =
                NewSearchItemServerModel::new(sever_link_model.id, query.clone())
                    .insert_or_inc(&db)
                    .await?;

            if search_item_updated {
                // Only insert/update if previous one was inserted/updated.
                NewSearchGroupModel::new(query.clone(), total)
                    .insert_or_inc(&db)
                    .await?;
            }
        }

        // Only search if our offset is less than the total amount we have.
        let items = if offset < total {
            BookModel::search_book_list(
                &QueryType::Query(query),
                offset,
                limit,
                OrderBy::Asc,
                !view_private,
                &db,
            )
            .await?
        } else {
            Vec::new()
        };

        // TODO: If we only found 1 item we'll use the singular item response.

        Ok(web::Json(WrappingResponse::okay(
            PublicSearchType::BookList(QueryListResponse {
                offset,
                limit,
                total,
                items: items
                    .into_iter()
                    .map(|v| v.into_partial_book(&host))
                    .collect(),
            }),
        )))
    }
}

#[get("/search/author")]
pub async fn public_search_author(
    req: HttpRequest,
    query: web::Query<GetSearchQuery>,
    db: web::Data<tokio_postgres::Client>,
) -> WebResult<web::Json<PublicSearchResponse>> {
    const ID_CHECK: &str = "id:";

    let sever_link_model = match ServerLinkModel::get_by_server_id(&query.server_id, &db).await? {
        Some(v) => v,
        None => return Ok(web::Json(WrappingResponse::error("Invalid Server ID"))),
    };

    let host = format!("//{}", req.headers().get("host").unwrap().to_str().unwrap());

    if query.query.starts_with(ID_CHECK) && query.query.len() > 3 {
        let book_id = PersonId::from(
            query.query[ID_CHECK.len()..]
                .parse::<usize>()
                .map_err(Error::from)?,
        );

        if let Some(model) = PersonModel::get_by_id(book_id, &db).await? {
            let other_names = PersonAltModel::find_all_by_person_id(model.id, &db)
                .await?
                .into_iter()
                .map(|v| v.name)
                .collect();

            Ok(web::Json(WrappingResponse::okay(
                PublicSearchType::AuthorItem(Some(model.into_public_author(&host, other_names))),
            )))
        } else {
            Ok(web::Json(WrappingResponse::okay(
                PublicSearchType::AuthorItem(None),
            )))
        }
    } else {
        let offset = query.offset.unwrap_or(0);
        let limit = query.limit.unwrap_or(25);

        let total = PersonModel::search_count(&query.query, &db).await?;

        // Only update if offset is 0.
        if offset == 0 {
            let search_item_updated =
                NewSearchItemServerModel::new(sever_link_model.id, query.query.clone())
                    .insert_or_inc(&db)
                    .await?;

            if search_item_updated {
                // Only insert/update if previous one was inserted/updated.
                NewSearchGroupModel::new(query.query.clone(), total)
                    .insert_or_inc(&db)
                    .await?;
            }
        }

        // Only search if our offset is less than the total amount we have.
        let items = if offset < total {
            PersonModel::search(&query.query, offset, limit, &db).await?
        } else {
            Vec::new()
        };

        Ok(web::Json(WrappingResponse::okay(
            PublicSearchType::AuthorList(QueryListResponse {
                offset,
                limit,
                total,
                items: items
                    .into_iter()
                    .map(|v| v.into_public_author(&host, Vec::new()))
                    .collect(),
            }),
        )))
    }
}
