use actix_web::{get, web, HttpRequest};
use common::{api::{WrappingResponse, QueryListResponse, librarian::{GetSearchQuery, PublicSearchResponse, PublicSearchType}}, BookId};
use common_local::api::OrderBy;

use crate::{WebResult, model::{BookModel, ServerLinkModel, NewSearchGroupModel, NewSearchItemServerModel}, Error};



#[get("/search/book")]
pub async fn public_search(
    req: HttpRequest,
    query: web::Query<GetSearchQuery>,
    db: web::Data<tokio_postgres::Client>,
) -> WebResult<web::Json<PublicSearchResponse>> {
    const BOOK_ID_CHECK: &str = "id:";

    let sever_link_model = match ServerLinkModel::get_by_server_id(&query.server_id, &db).await? {
        Some(v) => v,
        None => return Ok(web::Json(WrappingResponse::error("Invalid Server ID"))),
    };

    let host = format!("//{}", req.headers().get("host").unwrap().to_str().unwrap());

    if query.query.starts_with(BOOK_ID_CHECK) && query.query.len() > 3 {
        let book_id = BookId::from(query.query[BOOK_ID_CHECK.len()..].parse::<usize>().map_err(Error::from)?);

        let model = BookModel::get_by_id(book_id, &db).await?;

        Ok(web::Json(WrappingResponse::okay(
            PublicSearchType::BookItem(model.map(|v| v.into_public_book(&host)))
        )))
    } else {
        let offset = query.offset.unwrap_or(0);
        let limit = query.limit.unwrap_or(25);

        let total = BookModel::count_search_book(
            Some(&query.query),
            !query.view_private,
            None,
            &db,
        ).await?;

        // Only update if offset is 0.
        if offset == 0 {
            let search_item_updated = NewSearchItemServerModel::new(
                sever_link_model.id,
                query.query.clone()
            ).insert_or_inc(&db).await?;

            if search_item_updated {
                // Only insert/update if previous one was inserted/updated.
                NewSearchGroupModel::new(query.query.clone(), total).insert_or_inc(&db).await?;
            }
        }

        // Only search if our offset is less than the total amount we have.
        let items = if offset < total {
            BookModel::search_book_list(
                Some(&query.query),
                offset,
                limit,
                OrderBy::Asc,
                !query.view_private,
                None,
                &db,
            ).await?
        } else {
            Vec::new()
        };

        Ok(web::Json(WrappingResponse::okay(PublicSearchType::BookList(QueryListResponse {
            offset,
            limit,
            total,
            items: items.into_iter()
                .map(|v| v.into_partial_book(&host))
                .collect()
        }))))
    }
}