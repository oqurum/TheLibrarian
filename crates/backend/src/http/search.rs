use actix_web::{get, web, HttpRequest};
use common::api::WrappingResponse;
use common_local::search::{self, BookSearchResponse, PublicBook};

use crate::{WebResult, database::Database, model::{BookModel, ServerLinkModel, NewSearchGroupModel, NewSearchItemServerModel}, http::JsonResponse};


// TODO: Author Search


#[get("/search")]
pub async fn public_search(
    req: HttpRequest,
    query: web::Query<search::GetSearchQuery>,
    db: web::Data<Database>,
) -> WebResult<JsonResponse<BookSearchResponse>> {
    let sever_link_model = match ServerLinkModel::get_by_server_id(&query.server_id, &db).await? {
        Some(v) => v,
        None => return Ok(web::Json(WrappingResponse::error("Invalid Server ID"))),
    };

    let offset = query.offset.unwrap_or(0);
    let limit = query.limit.unwrap_or(25);

    let total = BookModel::count_search_book(
        Some(&query.query),
        !query.view_private,
        None,
        &db,
    ).await?;

    let search_item_updated = NewSearchItemServerModel::new(sever_link_model.id, query.query.clone()).insert_or_inc(&db).await?;

    if search_item_updated {
        // Only insert/update if previous one was inserted/updated.
        NewSearchGroupModel::new(query.query.clone(), total).insert_or_inc(&db).await?;
    }

    let items = BookModel::search_book_list(
        Some(&query.query),
        offset,
        limit,
        !query.view_private,
        None,
        &db,
    ).await?;

    let host = format!(
        "//{}",
        req.headers().get("host").unwrap().to_str().unwrap()
    );

    Ok(web::Json(WrappingResponse::okay(BookSearchResponse {
        offset,
        limit,
        total,
        items: items.into_iter()
            .map(|v| {
                let id = v.thumb_path.as_value().unwrap().to_string();

                let mut book: PublicBook = v.into();

                book.thumb_url = format!(
                    "{}/api/v1/image/{}",
                    &host,
                    id
                );

                book
            })
            .collect()
    })))
}