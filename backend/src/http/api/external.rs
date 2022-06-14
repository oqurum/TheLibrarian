use actix_web::{get, web};
use librarian_common::{api, SearchType, SearchForBooksBy, SearchFor, Source};

use crate::{WebResult, metadata};


#[get("/external/search")]
pub async fn get_external_search(
	body: web::Query<api::GetMetadataSearch>
) -> WebResult<web::Json<api::ExternalSearchResponse>> {
	let search = metadata::search_all_agents(
		&body.query,
		match body.search_type {
			// TODO: Allow for use in Query.
			SearchType::Book => SearchFor::Book(SearchForBooksBy::Query),
			SearchType::Person => SearchFor::Person
		}
	).await?;

	Ok(web::Json(api::ExternalSearchResponse {
		items: search.0.into_iter()
			.map(|(a, b)| (
				a,
				b.into_iter().map(|v| {
					match v {
						metadata::SearchItem::Book(book) => {
							api::SearchItem::Book(api::MetadataBookSearchItem {
								source: book.source,
								author: book.cached.author,
								description: book.description,
								name: book.title.unwrap_or_else(|| String::from("Unknown title")),
								thumbnail_url: book.thumb_locations.first()
									.and_then(|v| v.as_url_value())
									.map(|v| v.to_string())
									.unwrap_or_default(),
							})
						}

						metadata::SearchItem::Author(author) => {
							api::SearchItem::Person(api::MetadataPersonSearchItem {
								source: author.source,

								cover_image: author.cover_image_url,

								name: author.name,
								other_names: author.other_names,
								description: author.description,

								birth_date: author.birth_date,
								death_date: author.death_date,
							})
						}
					}
				}).collect()
			))
			.collect()
	}))
}


#[get("/external/{source}")]
pub async fn get_external_item(
	path: web::Path<Source>
) -> WebResult<web::Json<api::ExternalSourceItemResponse>> {
	if let Some(meta) = metadata::get_metadata_by_source(&*path, true).await? {
		Ok(web::Json(api::ExternalSourceItemResponse {
			item: Some(meta.meta.into()),
		}))
	} else {
		Ok(web::Json(api::ExternalSourceItemResponse {
			item: None,
		}))
	}
}