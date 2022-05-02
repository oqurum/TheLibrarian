use actix_web::{get, web};

use librarian_common::{api, DisplayItem};

use crate::WebResult;
use crate::database::Database;




// TODO: Add body requests for specifics
#[get("/book/{id}")]
pub async fn load_book(file_id: web::Path<usize>, db: web::Data<Database>) -> WebResult<web::Json<Option<api::GetBookIdResponse>>> {
	Ok(web::Json(if let Some(file) = db.find_file_by_id(*file_id)? {
		Some(api::GetBookIdResponse {
			progress: db.get_progress(0, *file_id)?.map(|v| v.into()),

			media: file.into()
		})
	} else {
		None
	}))
}


#[get("/books")]
pub async fn load_book_list(
	query: web::Query<api::BookListQuery>,
	db: web::Data<Database>,
) -> WebResult<web::Json<api::GetBookListResponse>> {
	let (items, count) = if let Some(search) = query.search_query() {
		let search = search?;

		let count = db.count_search_metadata(&search, query.library)?;

		let items = if count == 0 {
			Vec::new()
		} else {
			db.search_metadata_list(
				&search,
				query.library,
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
		let count = db.get_file_count()?;

		let items = db.get_metadata_by(
			query.library,
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