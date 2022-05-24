use actix_web::{get, web, HttpRequest};
use librarian_common::search::{self, BookSearchResponse, PublicBook};

use crate::{WebResult, database::Database, model::BookModel};


// TODO: Author Search


#[get("/search")]
pub async fn public_search(
	req: HttpRequest,
	query: web::Query<search::GetSearchQuery>,
	db: web::Data<Database>,
) -> WebResult<web::Json<BookSearchResponse>> {
	let offset = query.offset.unwrap_or(0);
	let limit = query.limit.unwrap_or(25);

	let total = BookModel::count_search_book(
		Some(&query.query),
		!query.view_private,
		None,
		&db,
	)?;

	let items = BookModel::search_book_list(
		Some(&query.query),
		offset,
		limit,
		!query.view_private,
		None,
		&db,
	)?;

	let host = format!(
		"//{}",
		req.headers().get("host").unwrap().to_str().unwrap()
	);

	Ok(web::Json(BookSearchResponse {
		offset,
		limit,
		total,
		items: items.into_iter()
			.map(|v| {
				let id = v.thumb_path.as_value().to_string();

				let mut book: PublicBook = v.into();

				book.thumb_url = format!(
					"{}/api/v1/image/{}",
					&host,
					id
				);

				book
			})
			.collect()
	}))
}