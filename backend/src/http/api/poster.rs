use std::io::{Write, Cursor};

use actix_files::NamedFile;
use actix_web::{get, post, web, HttpResponse, Responder};
use common::{Either, ImageIdType, ImageType, BookId, PersonId};
use futures::TryStreamExt;
use librarian_common::{Poster, api};

use crate::{WebResult, Error, store_image, database::Database, model::{BookModel, ImageLinkModel, UploadedImageModel, PersonModel}, http::{JsonResponse, MemberCookie}};



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
	let current_thumb = match image.type_of {
		ImageType::Book => BookModel::get_by_id(BookId::from(image.id), &db).await?.unwrap().thumb_path,
		ImageType::Person => PersonModel::get_by_id(PersonId::from(image.id), &db).await?.unwrap().thumb_url,
	};

	let items = ImageLinkModel::get_by_linked_id(image.id, image.type_of, &db).await?
		.into_iter()
		.map(|poster| Poster {
			id: Some(poster.image_id),

			selected: poster.path == current_thumb,

			path: poster.path.as_url(),

			created_at: poster.created_at,
		})
		.collect();

	Ok(web::Json(api::WrappingResponse::new(api::GetPostersResponse {
		items
	})))
}


#[post("/posters/{id}")]
async fn post_change_poster(
	image: web::Path<ImageIdType>,
	body: web::Json<api::ChangePosterBody>,
	member: MemberCookie,
	db: web::Data<Database>
) -> WebResult<HttpResponse> {
	let member = member.fetch(&db).await?.unwrap();

	if !member.permissions.has_editing_perms() {
		return Ok(HttpResponse::InternalServerError().json(api::WrappingResponse::<()>::error("You cannot do this! No Permissions!")));
	}

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
			let mut person = PersonModel::get_by_id(PersonId::from(image.id), &db).await?.unwrap();

			match body.into_inner().url_or_id {
				Either::Left(url) => {
					let resp = reqwest::get(url)
						.await.map_err(Error::from)?
						.bytes()
						.await.map_err(Error::from)?;

					let image_model = store_image(resp.to_vec(), &db).await?;

					person.thumb_url = image_model.path;

					ImageLinkModel::new_person(image_model.id, person.id)
						.insert(&db).await?;
				}

				Either::Right(id) => {
					let poster = UploadedImageModel::get_by_id(id, &db).await?.unwrap();

					if person.thumb_url == poster.path {
						return Ok(HttpResponse::Ok().finish());
					}

					person.thumb_url = poster.path;
				}
			}

			person.update(&db).await?;
		}
	}

	Ok(HttpResponse::Ok().finish())
}


#[post("/posters/{id}/upload")]
async fn post_upload_poster(
	image: web::Path<ImageIdType>,
	mut body: web::Payload,
	member: MemberCookie,
	db: web::Data<Database>,
) -> WebResult<HttpResponse> {
	let member = member.fetch(&db).await?.unwrap();

	if !member.permissions.has_editing_perms() {
		return Ok(HttpResponse::InternalServerError().json(api::WrappingResponse::<()>::error("You cannot do this! No Permissions!")));
	}

	match image.type_of {
		ImageType::Book => {
			let book = BookModel::get_by_id(BookId::from(image.id), &db).await?.unwrap();

			let mut file = Cursor::new(Vec::new());

			while let Some(item) = body.try_next().await? {
				file.write_all(&item).map_err(Error::from)?;
			}

			let image_model = store_image(file.into_inner(), &db).await?;

			ImageLinkModel::new_book(image_model.id, book.id)
				.insert(&db).await?;
		}

		ImageType::Person => {
			let person = PersonModel::get_by_id(PersonId::from(image.id), &db).await?.unwrap();

			let mut file = Cursor::new(Vec::new());

			while let Some(item) = body.try_next().await? {
				file.write_all(&item).map_err(Error::from)?;
			}

			let image_model = store_image(file.into_inner(), &db).await?;

			ImageLinkModel::new_person(image_model.id, person.id)
				.insert(&db).await?;
		}
	}

	Ok(HttpResponse::Ok().finish())
}