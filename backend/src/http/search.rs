use actix_web::{get, web};
use librarian_common::search::{self, BookSearchResponse};

use crate::{WebResult, database::Database};


// TODO: Author Search


#[get("/search")]
pub async fn public_search(
	query: web::Query<search::GetSearchQuery>,
	db: web::Data<Database>,
) -> WebResult<web::Json<BookSearchResponse>> {
	let offset = query.offset.unwrap_or(0);
	let limit = query.limit.unwrap_or(25);

	let total = db.count_search_book(
		Some(&query.query),
		!query.view_private,
		None
	)?;

	let items = db.search_book_list(
		Some(&query.query),
		offset,
		limit,
		!query.view_private,
		None
	)?;

	Ok(web::Json(BookSearchResponse {
		offset,
		limit,
		total,
		items: items.into_iter()
			.map(|v| v.into())
			.collect()
	}))
}