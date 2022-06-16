use serde::{Serialize, Deserialize};
use wasm_bindgen::{JsValue, JsCast};
use wasm_bindgen_futures::JsFuture;
use web_sys::{RequestInit, Request, RequestMode, Response, Headers, FormData};

use librarian_common::{api::*, SearchType, Either, Source, TagType, PersonId, BookId, TagId, ImageId, EditId, item::edit::{UpdateEditModel, BookEdit}, ImageIdType};

// TODO: Manage Errors.



// Edits

pub async fn get_edit_list(
	offset: Option<usize>,
	limit: Option<usize>
) -> GetEditListResponse {
	let mut url = String::from("/api/v1/edits?");

	if let Some(value) = offset {
		url += "offset=";
		url += &value.to_string();
		url += "&";
	}

	if let Some(value) = limit {
		url += "limit=";
		url += &value.to_string();
		url += "&";
	}

	fetch(
		"GET",
		&url,
		Option::<&()>::None
	).await.unwrap()
}

pub async fn update_edit_item(id: EditId, value: &UpdateEditModel) -> PostEditResponse {
	fetch(
		"POST",
		&format!("/api/v1/edit/{}", id),
		Some(value)
	).await.unwrap()
}



// Tags

pub async fn get_tags() -> GetTagsResponse {
	fetch(
		"GET",
		"/api/v1/tags",
		Option::<&()>::None
	).await.unwrap()
}

pub async fn get_tag(id: TagId) -> GetTagResponse {
	fetch(
		"GET",
		&format!("/api/v1/tag/{}", id),
		Option::<&()>::None
	).await.unwrap()
}

pub async fn new_tag(name: String, type_of: TagType) -> NewTagResponse {
	fetch(
		"POST",
		"/api/v1/tag",
		Some(&NewTagBody {
			name,
			type_of
		})
	).await.unwrap()
}


// Book Tag

pub async fn new_book_tag(book_id: BookId, tag_id: TagId, index: Option<usize>) -> NewBookTagResponse {
	fetch(
		"POST",
		&format!("/api/v1/tag/book/{book_id}"),
		Some(&NewBookTagBody {
			tag_id,
			index,
		})
	).await.unwrap()
}

pub async fn get_book_tag(book_id: BookId, tag_id: TagId) -> GetBookTagResponse {
	fetch(
		"GET",
		&format!("/api/v1/tag/{tag_id}/book/{book_id}"),
		Option::<&()>::None
	).await.unwrap()
}

pub async fn delete_book_tag(book_id: BookId, tag_id: TagId) -> DeletionResponse {
	fetch(
		"DELETE",
		&format!("/api/v1/tag/{tag_id}/book/{book_id}"),
		Option::<&()>::None
	).await.unwrap()
}


// Image

pub async fn get_posters_for_meta(img_id_type: ImageIdType) -> GetPostersResponse {
	fetch(
		"GET",
		&format!("/api/v1/posters/{}", img_id_type),
		Option::<&()>::None
	).await.unwrap()
}

pub async fn change_poster_for_meta(img_id_type: ImageIdType, url_or_id: Either<String, ImageId>) {
	let _: Option<String> = fetch(
		"POST",
		&format!("/api/v1/posters/{}", img_id_type),
		Some(&ChangePosterBody {
			url_or_id
		})
	).await.ok();
}


// Member

pub async fn get_member_self() -> GetMemberSelfResponse {
	fetch("GET", "/api/v1/member", Option::<&()>::None).await.unwrap_or_default()
}


// Metadata

pub async fn update_book(id: BookId, value: &BookEdit) {
	let _: Option<String> = fetch(
		"POST",
		&format!("/api/v1/book/{}", id),
		Some(value)
	).await.ok();
}

pub async fn get_media_view(book_id: BookId) -> MediaViewResponse {
	fetch(
		"GET",
		&format!("/api/v1/book/{}", book_id),
		Option::<&()>::None
	).await.unwrap()
}


// External

pub async fn external_search_for(search: &str, search_for: SearchType) -> ExternalSearchResponse {
	fetch(
		"GET",
		&format!(
			"/api/v1/external/search?query={}&search_type={}",
			urlencoding::encode(search),
			serde_json::to_string(&search_for).unwrap().replace('"', "")
		),
		Option::<&()>::None
	).await.unwrap()
}


pub async fn get_external_source_item(value: Source) -> ExternalSourceItemResponse {
	fetch(
		"GET",
		&format!(
			"/api/v1/external/{}",
			value
		),
		Option::<&()>::None
	).await.unwrap()
}


// People


pub async fn update_person(id: PersonId, body: &PostPersonBody) {
	let _: Option<String> = fetch(
		"POST",
		&format!("/api/v1/person/{}", id),
		Some(body)
	).await.ok();
}

pub async fn get_people(query: Option<&str>, offset: Option<usize>, limit: Option<usize>) -> GetPeopleResponse {
	let mut url = String::from("/api/v1/people?");

	if let Some(value) = offset {
		url += "offset=";
		url += &value.to_string();
		url += "&";
	}

	if let Some(value) = limit {
		url += "limit=";
		url += &value.to_string();
		url += "&";
	}

	if let Some(value) = query {
		url += "query=";
		url += &urlencoding::encode(value);
	}

	fetch(
		"GET",
		&url,
		Option::<&()>::None
	).await.unwrap()
}

pub async fn get_person(id: PersonId) -> GetPersonResponse {
	fetch(
		"GET",
		&format!("/api/v1/person/{}", id),
		Option::<&()>::None
	).await.unwrap()
}

// Books

pub async fn new_book(value: Source) {
	let _: Option<String> = fetch(
		"POST",
		"/api/v1/book",
		Some(&NewBookBody {
			source: value
		})
	).await.ok();
}


pub async fn get_books(
	offset: Option<usize>,
	limit: Option<usize>,
	search: Option<SearchQuery>,
	person_id: Option<PersonId>,
) -> GetBookListResponse {
	let url = format!(
		"/api/v1/books?{}",
		serde_urlencoded::to_string(BookListQuery::new(offset, limit, search, person_id).unwrap()).unwrap()
	);

	fetch("GET", &url, Option::<&()>::None).await.unwrap()
}

pub async fn get_book_info(id: BookId) -> GetBookIdResponse {
	fetch("GET", &format!("/api/v1/book/{}", id), Option::<&()>::None).await.unwrap()
}


// Options

pub async fn get_settings() -> GetSettingsResponse {
	fetch("GET", "/api/v1/settings", Option::<&()>::None).await.unwrap()
}

pub async fn update_options_add(options: ModifyOptionsBody) {
	let _: Option<String> = fetch(
		"POST",
		"/api/v1/options/add",
		Some(&options)
	).await.ok();
}

pub async fn update_options_remove(options: ModifyOptionsBody) {
	let _: Option<String> = fetch(
		"POST",
		"/api/v1/options/remove",
		Some(&options)
	).await.ok();
}

pub async fn run_task() { // TODO: Use common::api::RunTaskBody
	let _: Option<String> = fetch(
		"POST",
		"/api/v1/task",
		Some(&serde_json::json!({
			"run_search": true,
			"run_metadata": true
		}))
	).await.ok();
}





async fn fetch<V: for<'a> Deserialize<'a>>(method: &str, url: &str, body: Option<&impl Serialize>) -> Result<V, JsValue> {
	let mut opts = RequestInit::new();
	opts.method(method);
	opts.mode(RequestMode::Cors);

	if let Some(body) = body {
		opts.body(Some(&JsValue::from_str(&serde_json::to_string(body).unwrap())));

		let headers = Headers::new()?;
		headers.append("Content-Type", "application/json")?;
		opts.headers(&headers);
	}

	let request = Request::new_with_str_and_init(url, &opts)?;

	let window = gloo_utils::window();
	let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;
	let resp: Response = resp_value.dyn_into().unwrap();

	let text = JsFuture::from(resp.json()?).await?;

	Ok(text.into_serde().unwrap())
}


async fn fetch_url_encoded<V: for<'a> Deserialize<'a>>(method: &str, url: &str, form_data: FormData) -> Result<V, JsValue> {
	let mut opts = RequestInit::new();
	opts.method(method);
	opts.mode(RequestMode::Cors);

	opts.body(Some(&form_data));

	let request = Request::new_with_str_and_init(url, &opts)?;

	let window = gloo_utils::window();
	let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;
	let resp: Response = resp_value.dyn_into().unwrap();

	let text = JsFuture::from(resp.json()?).await?;

	Ok(text.into_serde().unwrap())
}