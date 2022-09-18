use actix_web::{get, web, HttpRequest};
use common::{api::{WrappingResponse, QueryListResponse, librarian::{GetSearchQuery, BookSearchResponse, PublicBook}}, BookId, Either};
use common_local::api::OrderBy;

use crate::{WebResult, model::{BookModel, ServerLinkModel, NewSearchGroupModel, NewSearchItemServerModel}, Error};


// TODO: Author Search


#[get("/search")]
pub async fn public_search(
    req: HttpRequest,
    query: web::Query<GetSearchQuery>,
    db: web::Data<tokio_postgres::Client>,
) -> WebResult<web::Json<BookSearchResponse>> {
    let sever_link_model = match ServerLinkModel::get_by_server_id(&query.server_id, &db).await? {
        Some(v) => v,
        None => return Ok(web::Json(WrappingResponse::error("Invalid Server ID"))),
    };

    let host = format!("//{}", req.headers().get("host").unwrap().to_str().unwrap());


    if query.query.starts_with("id:") && query.query.len() > 3 {
        let book_id = BookId::from(query.query[3..].parse::<usize>().map_err(Error::from)?);

        let model = BookModel::get_by_id(book_id, &db).await?;

        Ok(web::Json(WrappingResponse::okay(
            Either::Right(model.map(|v| {
                let id = v.thumb_path.as_value().unwrap().to_string();

                let mut book: PublicBook = v.into();

                book.thumb_url = format!(
                    "{}/api/v1/image/{}",
                    &host,
                    id
                );

                book
            }))
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

        Ok(web::Json(WrappingResponse::okay(Either::Left(QueryListResponse {
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
        }))))
    }
}