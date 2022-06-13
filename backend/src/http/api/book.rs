use actix_web::{get, web, HttpResponse, post};

use librarian_common::item::edit::BookEdit;
use librarian_common::{api, DisplayItem, BookId, PersonId};

use crate::http::MemberCookie;
use crate::metadata::MetadataReturned;
use crate::model::{NewImageModel, BookPersonModel, BookModel, BookTagWithTagModel, PersonModel, NewEditModel};
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

		let mut db_book: BookModel = meta.into();

		db_book.cached = db_book.cached.publisher_optional(publisher).author_optional(main_author);

		db_book.add_or_update_book(&db).await?;

		for path in posters_to_add {
			NewImageModel::new_book(db_book.id, path).insert(&db).await?;
		}

		for person_id in author_ids {
			let model = BookPersonModel {
				book_id: db_book.id,
				person_id,
			};

			model.insert(&db).await?;
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

		let count = BookModel::count_search_book(search.query.as_deref(), false, query.person_id.map(PersonId::from), &db).await?;

		let items = if count == 0 {
			Vec::new()
		} else {
			BookModel::search_book_list(
				search.query.as_deref(),
				query.offset.unwrap_or(0),
				query.limit.unwrap_or(50),
				false,
				query.person_id.map(PersonId::from),
				&db
			).await?
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
		let count = BookModel::get_book_count(&db).await?;

		let items = BookModel::get_book_by(
			query.offset.unwrap_or(0),
			query.limit.unwrap_or(50),
			false,
			query.person_id.map(PersonId::from),
			&db,
		).await?
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
pub async fn get_book_info(book_id: web::Path<BookId>, db: web::Data<Database>) -> WebResult<web::Json<api::MediaViewResponse>> {
	let book = BookModel::get_by_id(*book_id, &db).await?.unwrap();
	let people = PersonModel::get_all_by_book_id(book.id, &db).await?;
	let tags = BookTagWithTagModel::get_by_book_id(book.id, &db).await?;

	Ok(web::Json(api::MediaViewResponse {
		metadata: book.into(),
		people: people.into_iter()
			.map(|p| p.into())
			.collect(),
		tags: tags.into_iter()
			.map(|t| t.into())
			.collect(),
	}))
}


#[post("/book/{id}")]
pub async fn update_book_id(
	book_id: web::Path<BookId>,
	body: web::Json<BookEdit>,
	member: MemberCookie,
	db: web::Data<Database>,
) -> WebResult<HttpResponse> {
	let body = body.into_inner();

	let current_book = BookModel::get_by_id(*book_id, &db).await?;

	if let Some((updated_book, current_book)) = Some(body).zip(current_book) {
		// Make sure we have something we're updating.
		if !updated_book.is_empty() {
			let model = NewEditModel::from_book_modify(member.member_id(), current_book, updated_book)?;

			if !model.data.is_empty() {
				model.insert(&db).await?;
			}
		}
	}

	Ok(HttpResponse::Ok().finish())
}




#[get("/book/{id}/thumbnail")]
async fn load_book_thumbnail(path: web::Path<BookId>, db: web::Data<Database>) -> WebResult<HttpResponse> {
	let book_id = path.into_inner();

	let meta = BookModel::get_by_id(book_id, &db).await?;

	if let Some(loc) = meta.map(|v| v.thumb_path) {
		let path = crate::image::hash_to_path(loc.as_value());

		Ok(HttpResponse::Ok().body(tokio::fs::read(path).await.map_err(Error::from)?))
	} else {
		Ok(HttpResponse::NotFound().finish())
	}
}