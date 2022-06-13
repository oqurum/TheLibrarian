use std::io::Write;

use actix_files::NamedFile;
use actix_web::{get, post, web, HttpResponse, Responder};
use futures::TryStreamExt;
use librarian_common::{Poster, api, Either, ImageIdType, ImageType, BookId};

use crate::{WebResult, Error, store_image, database::Database, model::{NewImageModel, BookModel, ImageModel}};



#[get("/image/{id}")]
async fn get_local_image(id: web::Path<String>) -> impl Responder {
	let path = crate::image::hash_to_path(&id);

	NamedFile::open_async(path).await
}



#[get("/posters/{id}")]
async fn get_poster_list(
	image: web::Path<ImageIdType>,
	db: web::Data<Database>
) -> WebResult<web::Json<api::GetPostersResponse>> {
	let items: Vec<Poster> = match image.type_of {
		ImageType::Book => {
			let meta = BookModel::get_by_id(BookId::from(image.id), &db).await?.unwrap();

			ImageModel::get_by_linked_id(BookId::from(image.id), &db).await?
				.into_iter()
				.map(|poster| Poster {
					id: Some(poster.id),

					selected: poster.path == meta.thumb_path,

					path: poster.path.as_url(),

					created_at: poster.created_at,
				})
				.collect()
		}

		ImageType::Person => {
			// TODO
			Vec::new()
		}
	};

	Ok(web::Json(api::GetPostersResponse {
		items
	}))
}


#[post("/posters/{id}")]
async fn post_change_poster(
	image: web::Path<ImageIdType>,
	body: web::Json<api::ChangePosterBody>,
	db: web::Data<Database>
) -> WebResult<HttpResponse> {
	match image.type_of {
		ImageType::Book => {
			let mut meta = BookModel::get_by_id(BookId::from(image.id), &db).await?.unwrap();

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
		}

		ImageType::Person => {
			// TODO
		}
	}

	Ok(HttpResponse::Ok().finish())
}


#[post("/posters/{id}/upload")]
async fn post_upload_poster(
	image: web::Path<ImageIdType>,
	mut body: web::Payload,
	db: web::Data<Database>,
) -> WebResult<HttpResponse> {
	match image.type_of {
		ImageType::Book => {
			let book = BookModel::get_by_id(BookId::from(image.id), &db).await?.unwrap();

			let mut file = std::io::Cursor::new(Vec::new());

			while let Some(item) = body.try_next().await? {
				file.write_all(&item).map_err(Error::from)?;
			}

			let hash = store_image(file.into_inner()).await?;

			NewImageModel::new_book(book.id, hash)
				.insert(&db).await?;
		}

		ImageType::Person => {
			// TODO
		}
	}

	Ok(HttpResponse::Ok().finish())
}