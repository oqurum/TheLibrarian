use actix_web::{get, web, HttpResponse, post, HttpRequest};

use chrono::Utc;
use common::api::WrappingResponse;
use common::{Either, ThumbnailStore, BookId, PersonId};
use common_local::item::edit::{BookEdit, NewOrCachedImage};
use common_local::{api, DisplayItem, MetadataItemCached, DisplayMetaItem};

use crate::http::{MemberCookie, JsonResponse};
use crate::metadata::MetadataReturned;
use crate::model::{BookPersonModel, BookModel, BookTagWithTagModel, PersonModel, NewEditModel, ImageLinkModel, UploadedImageModel};
use crate::storage::get_storage;
use crate::{WebResult, metadata, Error};
use crate::database::Database;



#[post("/book")]
pub async fn add_new_book(
    body: web::Json<api::NewBookBody>,
    member: MemberCookie,
    db: web::Data<Database>,
) -> WebResult<JsonResponse<Option<DisplayMetaItem>>> {
    let member = member.fetch(&db).await?.unwrap();

    if !member.permissions.has_editing_perms() {
        return Ok(web::Json(WrappingResponse::error("You cannot do this! No Permissions!")));
    }

    let value = match body.into_inner() {
        // Used for the Search Item "Auto Find" Button
        api::NewBookBody::FindAndAdd(mut find_str) => {
            use metadata::{google_books, Metadata};

            // TODO: Check to see if we already have isbn: prefixed before the find_str
            // Check if we're searching by ISBN, if so check that we don't already have it in DB.
            if let Some(isbn) = common::parse_book_id(&find_str).into_possible_isbn_value() {
                if find_str.trim() == isbn {
                    if BookModel::exists_by_isbn(&isbn, &db).await? {
                        return Ok(web::Json(WrappingResponse::error("Book ISBN already exists!")));
                    } else {
                        // Add isbn: before the string to specify the book we want.
                        find_str = format!("isbn:{find_str}");
                    }
                }
            }

            let found = google_books::GoogleBooksMetadata.search(
                &find_str,
                common_local::SearchFor::Book(common_local::SearchForBooksBy::Query),
                &db
            ).await?;

            if let Some(item) = found.first().and_then(|v| v.as_book()) {
                Either::Left(item.source.clone())
            } else {
                return Ok(web::Json(WrappingResponse::error("Unable to find an item to add!")));
            }
        }

        api::NewBookBody::Value(v) => *v,
    };

    match value {
        Either::Left(source) => {
            if let Some(mut meta) = metadata::get_metadata_by_source(&source, true, &db).await? {
                let (main_author, author_ids) = meta.add_or_ignore_authors_into_database(&db).await?;

                let MetadataReturned { mut meta, publisher, .. } = meta;
                let mut posters_to_add = Vec::new();

                for item in meta.thumb_locations.iter_mut().filter(|v| v.is_url()) {
                    item.download(&db).await?;

                    if let Some(v) = item.as_local_value().cloned() {
                        posters_to_add.push(v);
                    }
                }

                let mut db_book: BookModel = meta.into();

                db_book.cached = db_book.cached.publisher_optional(publisher).author_optional(main_author);

                db_book.add_or_update_book(&db).await?;

                for path in posters_to_add {
                    if let Some(model) = UploadedImageModel::get_by_path(path.as_value().unwrap(), &db).await? {
                        ImageLinkModel::new_book(model.id, db_book.id).insert(&db).await?;
                    }
                }

                for person_id in author_ids {
                    let model = BookPersonModel {
                        book_id: db_book.id,
                        person_id,
                    };

                    model.insert(&db).await?;
                }

                return Ok(web::Json(WrappingResponse::okay(Some(db_book.into()))));
            }
        }

        Either::Right(book) => {
            let mut thumb_path = ThumbnailStore::None;

            // Used to link to the newly created book
            let mut uploaded_images = Vec::new();

            // Upload images and set the new book image.
            if let Some(added_images) = book.added_images.as_ref() {
                for id_or_url in added_images {
                    match id_or_url {
                        &NewOrCachedImage::Id(id) => {
                            uploaded_images.push(id);

                            if thumb_path.is_none() {
                                let image = UploadedImageModel::get_by_id(id, &db).await?.unwrap();

                                thumb_path = image.path;
                            }
                        }

                        NewOrCachedImage::Url(url) => {
                            let resp = reqwest::get(url)
                                .await.map_err(Error::from)?
                                .bytes()
                                .await.map_err(Error::from)?;

                            let model = crate::store_image(resp.to_vec(), &db).await?;

                            if thumb_path.is_none() {
                                thumb_path = model.path;
                            }

                            uploaded_images.push(model.id);
                        }
                    }
                }
            }

            let mut book_model = BookModel {
                id: BookId::none(),
                title: book.title,
                clean_title: book.clean_title,
                description: book.description,
                rating: book.rating.unwrap_or_default(),
                thumb_path: ThumbnailStore::None,
                cached: MetadataItemCached::default().publisher_optional(book.publisher),
                isbn_10: book.isbn_10,
                isbn_13: book.isbn_13,
                is_public: book.is_public.unwrap_or_default(),
                available_at: book.available_at,
                language: book.language,
                edition_count: 0,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                deleted_at: None,
            };

            book_model.add_or_update_book(&db).await?;

            for image_id in uploaded_images {
                ImageLinkModel::new_book(image_id, book_model.id)
                    .insert(&db).await?;
            }

            return Ok(web::Json(WrappingResponse::okay(Some(book_model.into()))));
        }
    }

    Ok(web::Json(WrappingResponse::okay(None)))
}



#[get("/books")]
pub async fn load_book_list(
    query: web::Query<api::BookListQuery>,
    db: web::Data<Database>,
) -> WebResult<JsonResponse<api::GetBookListResponse>> {
    let (items, count) = if let Some(search) = query.search_query().transpose()? {
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

    Ok(web::Json(WrappingResponse::okay(api::GetBookListResponse {
        items,
        count,
    })))
}



#[get("/book/{id}")]
pub async fn get_book_info(book_id: web::Path<BookId>, db: web::Data<Database>) -> WebResult<JsonResponse<api::MediaViewResponse>> {
    let book = BookModel::get_by_id(*book_id, &db).await?.unwrap();
    let people = PersonModel::get_all_by_book_id(book.id, &db).await?;
    let tags = BookTagWithTagModel::get_by_book_id(book.id, &db).await?;

    Ok(web::Json(WrappingResponse::okay(api::MediaViewResponse {
        metadata: book.into(),
        people: people.into_iter()
            .map(|p| p.into())
            .collect(),
        tags: tags.into_iter()
            .map(|t| t.into())
            .collect(),
    })))
}


#[post("/book/{id}")]
pub async fn update_book_id(
    book_id: web::Path<BookId>,
    body: web::Json<BookEdit>,
    member: MemberCookie,
    db: web::Data<Database>,
) -> WebResult<JsonResponse<&'static str>> {
    let body = body.into_inner();

    let member = member.fetch(&db).await?.unwrap();

    if !member.permissions.has_editing_perms() {
        return Ok(web::Json(WrappingResponse::error("You cannot do this! No Permissions!")));
    }

    let current_book = BookModel::get_by_id(*book_id, &db).await?;

    if let Some((updated_book, current_book)) = Some(body).zip(current_book) {
        // Make sure we have something we're updating.
        if !updated_book.is_empty() {
            let model = NewEditModel::from_book_modify(member.id, current_book, updated_book)?;

            if !model.data.is_empty() {
                model.insert(&db).await?;
            }
        }
    }

    Ok(web::Json(WrappingResponse::okay("success")))
}




#[get("/book/{id}/thumbnail")]
async fn load_book_thumbnail(path: web::Path<BookId>, req: HttpRequest, db: web::Data<Database>) -> WebResult<HttpResponse> {
    let book_id = path.into_inner();

    let meta = BookModel::get_by_id(book_id, &db).await?;

    if let Some(file_name) = meta.as_ref().and_then(|v| v.thumb_path.as_value()) {
        Ok(get_storage().await.get_http_response(file_name, &req).await?)
    } else {
        Ok(HttpResponse::NotFound().finish())
    }
}