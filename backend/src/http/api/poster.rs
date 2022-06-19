use std::io::Write;

use actix_files::NamedFile;
use actix_web::{get, post, web, HttpResponse, Responder};
use futures::TryStreamExt;
use librarian_common::{Poster, api, Either, ImageIdType, ImageType, BookId};

use crate::{WebResult, Error, store_image, database::Database, model::{BookModel, ImageLinkModel, UploadedImageModel}, http::JsonResponse};



#[get("/image/{id}")]
async fn get_local_image(id: web::Path<String>) -> impl Responder {
	let path = crate::image::hash_to_path(&id);

	NamedFile::open_async(path).await
}



#[get("/posters/{id}")]
async fn get_poster_list(
	image: web::Path<ImageIdType>,
	db: web::Data<Database>
) -> WebResult<JsonResponse<api::GetPostersResponse>> {
	let items: Vec<Poster> = match image.type_of {
		ImageType::Book => {
			let book = BookModel::get_by_id(BookId::from(image.id), &db).await?.unwrap();

			ImageLinkModel::get_by_linked_id(image.id, image.type_of, &db).await?
				.into_iter()
				.map(|poster| Poster {
					id: Some(poster.image_id),

					selected: poster.path == book.thumb_path,

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

	Ok(web::Json(api::WrappingResponse::new(api::GetPostersResponse {
		items
	})))
}


#[post("/posters/{id}")]
async fn post_change_poster(
	image: web::Path<ImageIdType>,
	body: web::Json<api::ChangePosterBody>,
	db: web::Data<Database>
) -> WebResult<HttpResponse> {
	match image.type_of {
		ImageType::Book => {
			let mut book = BookModel::get_by_id(BookId::from(image.id), &db).await?.unwrap();

			match body.into_inner().url_or_id {
				Either::Left(url) => {
					let resp = reqwest::get(url)
						.await.map_err(Error::from)?
						.bytes()
						.await.map_err(Error::from)?;

					let image_model = store_image(resp.to_vec(), &db).await?;

					book.thumb_path = image_model.path;

					ImageLinkModel::new_book(image_model.id, book.id)
						.insert(&db).await?;
				}

				Either::Right(id) => {
					let poster = UploadedImageModel::get_by_id(id, &db).await?.unwrap();

					if book.thumb_path == poster.path {
						return Ok(HttpResponse::Ok().finish());
					}

					book.thumb_path = poster.path;
				}
			}

			book.update_book(&db).await?;
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

			let image_model = store_image(file.into_inner(), &db).await?;

			ImageLinkModel::new_book(image_model.id, book.id)
				.insert(&db).await?;
		}

		ImageType::Person => {
			// TODO
		}
	}

	Ok(HttpResponse::Ok().finish())
}