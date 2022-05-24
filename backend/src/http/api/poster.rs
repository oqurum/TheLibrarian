use std::io::Write;

use actix_files::NamedFile;
use actix_web::{get, post, web, HttpResponse, Responder};
use chrono::Utc;
use futures::TryStreamExt;
use librarian_common::{Poster, api, Either};

use crate::{WebResult, Error, store_image, database::Database, model::{NewPosterModel, BookModel}};



#[get("/image/{id}")]
async fn get_local_image(id: web::Path<String>) -> impl Responder {
	let path = crate::image::hash_to_path(&id);

	NamedFile::open_async(path).await
}



#[get("/posters/{meta_id}")]
async fn get_poster_list(
	path: web::Path<usize>,
	db: web::Data<Database>
) -> WebResult<web::Json<api::GetPostersResponse>> {
	let meta = BookModel::get_by_id(*path, &db)?.unwrap();

	let items: Vec<Poster> = db.get_posters_by_linked_id(*path)?
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
	metadata_id: web::Path<usize>,
	body: web::Json<api::ChangePosterBody>,
	db: web::Data<Database>
) -> WebResult<HttpResponse> {
	let mut meta = BookModel::get_by_id(*metadata_id, &db)?.unwrap();

	match body.into_inner().url_or_id {
		Either::Left(url) => {
			let resp = reqwest::get(url)
				.await.map_err(Error::from)?
				.bytes()
				.await.map_err(Error::from)?;

			let hash = store_image(resp.to_vec()).await?;


			meta.thumb_path = hash;

			db.add_poster(&NewPosterModel {
				link_id: meta.id,
				path: meta.thumb_path.clone(),
				created_at: Utc::now(),
			})?;
		}

		Either::Right(id) => {
			let poster = db.get_poster_by_id(id)?.unwrap();

			if meta.thumb_path == poster.path {
				return Ok(HttpResponse::Ok().finish());
			}

			meta.thumb_path = poster.path;
		}
	}

	meta.update_book(&db)?;

	Ok(HttpResponse::Ok().finish())
}


#[post("/posters/{book_id}/upload")]
async fn post_upload_poster(
	book_id: web::Path<usize>,
	mut body: web::Payload,
	db: web::Data<Database>,
) -> WebResult<HttpResponse> {
	let book = BookModel::get_by_id(*book_id, &db)?.unwrap();

	let mut file = std::io::Cursor::new(Vec::new());

	while let Some(item) = body.try_next().await? {
		file.write_all(&item).map_err(Error::from)?;
	}

	let hash = store_image(file.into_inner()).await?;

	db.add_poster(&NewPosterModel {
		link_id: book.id,
		path: hash,
		created_at: Utc::now(),
	})?;

	Ok(HttpResponse::Ok().finish())
}