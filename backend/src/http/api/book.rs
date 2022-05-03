use actix_web::{get, web, HttpResponse, post};

use chrono::Utc;
use librarian_common::{api, DisplayItem};

use crate::database::table::{BookModel, self};
use crate::metadata::MetadataReturned;
use crate::{WebResult, metadata, Error};
use crate::database::Database;



#[post("/book")]
pub async fn add_new_book(
	body: web::Json<api::NewBookBody>,
	db: web::Data<Database>,
) -> WebResult<HttpResponse> {
	if let Some(mut meta) = metadata::get_metadata_by_source(&body.source, true).await? {
		let (main_author, author_ids) = meta.add_or_ignore_authors_into_database(&db).await?;

		let MetadataReturned { mut meta, publisher, .. } = meta;
		let mut posters_to_add = Vec::new();

		for item in meta.thumb_locations.iter_mut().filter(|v| v.is_url()) {
			item.download().await?;

			if let Some(v) = item.as_local_value().cloned() {
				posters_to_add.push(v);
			}
		}

		let mut meta: BookModel = meta.into();

		meta.cached = meta.cached.publisher_optional(publisher).author_optional(main_author);

		let db_book = db.add_or_update_metadata(&meta)?;

		for path in posters_to_add {
			db.add_poster(&table::NewPosterModel {
				link_id: db_book.id,
				path,
				created_at: Utc::now(),
			})?;
		}

		for person_id in author_ids {
			db.add_meta_person(&table::BookPersonModel {
				book_id: db_book.id,
				person_id,
			})?;
		}
	}

	Ok(HttpResponse::Ok().finish())
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
						title: meta.title.or(meta.clean_title).unwrap_or_default(),
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
					title: meta.title.or(meta.clean_title).unwrap_or_default(),
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



#[get("/book/{id}")]
pub async fn get_book_info(meta_id: web::Path<usize>, db: web::Data<Database>) -> WebResult<web::Json<api::MediaViewResponse>> {
	let meta = db.get_metadata_by_id(*meta_id)?.unwrap();
	let people = db.get_person_list_by_meta_id(meta.id)?;

	Ok(web::Json(api::MediaViewResponse {
		metadata: meta.into(),
		people: people.into_iter()
			.map(|p| p.into())
			.collect(),
	}))
}




#[get("/book/{id}/thumbnail")]
async fn load_book_thumbnail(path: web::Path<usize>, db: web::Data<Database>) -> WebResult<HttpResponse> {
	let book_id = path.into_inner();

	let meta = db.get_metadata_by_id(book_id)?;

	if let Some(loc) = meta.map(|v| v.thumb_path) {
		let path = crate::image::prefixhash_to_path(loc.as_type(), loc.as_value());

		Ok(HttpResponse::Ok().body(tokio::fs::read(path).await.map_err(Error::from)?))
	} else {
		Ok(HttpResponse::NotFound().finish())
	}
}