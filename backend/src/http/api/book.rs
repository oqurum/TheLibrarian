use actix_web::{get, web, HttpResponse, post};

use librarian_common::{api, DisplayItem};

use crate::WebResult;
use crate::database::Database;



#[post("/book")]
pub async fn add_new_book(
	body: web::Json<api::NewBookBody>,
	db: web::Data<Database>,
) -> HttpResponse {
	//

	HttpResponse::Ok().finish()
}



#[get("/books")]
pub async fn load_book_list(
	query: web::Query<api::BookListQuery>,
	db: web::Data<Database>,
) -> WebResult<web::Json<api::GetBookListResponse>> {
	let (items, count) = if let Some(search) = query.search_query() {
		let search = search?;

		let count = db.count_search_metadata(&search)?;

		let items = if count == 0 {
			Vec::new()
		} else {
			db.search_metadata_list(
				&search,
				query.offset.unwrap_or(0),
				query.limit.unwrap_or(50),
			)?
				.into_iter()
				.map(|meta| {
					DisplayItem {
						id: meta.id,
						title: meta.title.or(meta.original_title).unwrap_or_default(),
						cached: meta.cached,
						has_thumbnail: meta.thumb_path.is_some()
					}
				})
				.collect()
		};

		(items, count)
	} else {
		let count = db.get_metadata_count()?;

		let items = db.get_metadata_by(
			query.offset.unwrap_or(0),
			query.limit.unwrap_or(50),
		)?
			.into_iter()
			.map(|meta| {
				DisplayItem {
					id: meta.id,
					title: meta.title.or(meta.original_title).unwrap_or_default(),
					cached: meta.cached,
					has_thumbnail: meta.thumb_path.is_some()
				}
			})
			.collect();

		(items, count)
	};

	Ok(web::Json(api::GetBookListResponse {
		items,
		count,
	}))
}