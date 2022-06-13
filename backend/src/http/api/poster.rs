use std::io::Write;

use actix_files::NamedFile;
use actix_web::{get, post, web, HttpResponse, Responder};
use futures::TryStreamExt;
use librarian_common::{Poster, api, Either, BookId};

use crate::{WebResult, Error, store_image, database::Database, model::{NewImageModel, BookModel, ImageModel}};



#[get("/image/{id}")]
async fn get_local_image(id: web::Path<String>) -> impl Responder {
	let path = crate::image::hash_to_path(&id);

	NamedFile::open_async(path).await
}



#[get("/posters/{meta_id}")]
async fn get_poster_list(
	path: web::Path<BookId>,
	db: web::Data<Database>
) -> WebResult<web::Json<api::GetPostersResponse>> {
	let meta = BookModel::get_by_id(*path, &db).await?.unwrap();

	let items: Vec<Poster> = ImageModel::get_by_linked_id(*path, &db).await?
		.into_iter()
		.map(|poster| Poster {
			id: Some(poster.id),

			selected: poster.path == meta.thumb_path,

			path: poster.path.as_url(),

			created_at: poster.created_at,
		})
		.collect();

	Ok(web::Json(api::GetPostersResponse {
		items
	}))
}


#[post("/posters/{meta_id}")]
async fn post_change_poster(
	book_id: web::Path<BookId>,
	body: web::Json<api::ChangePosterBody>,
	db: web::Data<Database>
) -> WebResult<HttpResponse> {
	let mut meta = BookModel::get_by_id(*book_id, &db).await?.unwrap();

	match body.into_inner().url_or_id {
		Either::Left(url) => {
			let resp = reqwest::get(url)
				.await.map_err(Error::from)?
				.bytes()
				.await.map_err(Error::from)?;

			let hash = store_image(resp.to_vec()).await?;


			meta.thumb_path = hash;

			NewImageModel::new_book(meta.id, meta.thumb_path.clone())
				.insert(&db).await?;
		}

		Either::Right(id) => {
			let poster = ImageModel::get_by_id(id, &db).await?.unwrap();

			if meta.thumb_path == poster.path {
				return Ok(HttpResponse::Ok().finish());
			}

			meta.thumb_path = poster.path;
		}
	}

	meta.update_book(&db).await?;

	Ok(HttpResponse::Ok().finish())
}


#[post("/posters/{book_id}/upload")]
async fn post_upload_poster(
	book_id: web::Path<BookId>,
	mut body: web::Payload,
	db: web::Data<Database>,
) -> WebResult<HttpResponse> {
	let book = BookModel::get_by_id(*book_id, &db).await?.unwrap();

	let mut file = std::io::Cursor::new(Vec::new());

	while let Some(item) = body.try_next().await? {
		file.write_all(&item).map_err(Error::from)?;
	}

	let hash = store_image(file.into_inner()).await?;

	NewImageModel::new_book(book.id, hash)
		.insert(&db).await?;

	Ok(HttpResponse::Ok().finish())
}