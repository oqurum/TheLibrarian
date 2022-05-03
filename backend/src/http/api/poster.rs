use actix_files::NamedFile;
use actix_web::{get, post, put, web, HttpResponse, Responder};
use chrono::Utc;
use librarian_common::{Poster, api, ThumbnailStoreType, Either};

use crate::{WebResult, Error, store_image, database::{table::NewPosterModel, Database}};



#[get("/image/{type}/{id}")]
async fn get_local_image(path: web::Path<(String, String)>) -> impl Responder {
	let (type_of, id) = path.into_inner();

	let path = crate::image::prefixhash_to_path(
		ThumbnailStoreType::from(type_of.as_str()),
		&id
	);

	NamedFile::open_async(path).await
}



#[get("/posters/{meta_id}")]
async fn get_poster_list(
	path: web::Path<usize>,
	db: web::Data<Database>
) -> WebResult<web::Json<api::GetPostersResponse>> {
	let meta = db.get_metadata_by_id(*path)?.unwrap();

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
	let mut meta = db.get_metadata_by_id(*metadata_id)?.unwrap();

	match body.into_inner().url_or_id {
		Either::Left(url) => {
			let resp = reqwest::get(url)
				.await.map_err(Error::from)?
				.bytes()
				.await.map_err(Error::from)?;

			let hash = store_image(ThumbnailStoreType::Metadata, resp.to_vec()).await?;


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

	db.update_metadata(&meta)?;

	Ok(HttpResponse::Ok().finish())
}


#[put("/posters/{meta_id}")]
async fn put_upload_poster(
	path: web::Path<usize>,
	// body: web::Payload,
	db: web::Data<Database>
) -> HttpResponse {
	//

	HttpResponse::Ok().finish()
}