#![allow(dead_code)]

use common::{Source, PersonId, BookId, TagId, ImageId, Either, ImageIdType, api::{WrappingResponse, DeletionResponse, ApiErrorResponse, QueryListResponse}};
use serde::{Serialize, Deserialize};
use serde_json::json;
use wasm_bindgen::{JsValue, JsCast};
use wasm_bindgen_futures::JsFuture;
use web_sys::{RequestInit, Request, RequestMode, Response, Headers};

use common_local::{api::*, SearchType, TagType, EditId, item::edit::{UpdateEditModel, BookEdit}, update::OptionsUpdate, Member, SearchGroup, SearchGroupId, DisplayMetaItem, CollectionId};


// Collection

pub async fn get_collection_list(query: Option<&str>, offset: Option<usize>, limit: Option<usize>) -> WrappingResponse<GetCollectionListResponse> {
    let mut url = String::from("/api/v1/collections?");

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
    ).await.unwrap_or_else(def)
}


pub async fn update_collection(id: CollectionId, value: &UpdateCollectionModel) -> WrappingResponse<String> {
    fetch(
        "POST",
        &format!("/api/v1/collection/{}", id),
        Some(value)
    ).await.unwrap_or_else(def)
}

pub async fn get_collection(path: &str) -> WrappingResponse<GetCollectionResponse> {
    fetch(
        "GET",
        &format!("/api/v1/collection/{}", path),
        Option::<&()>::None
    ).await.unwrap_or_else(def)
}

pub async fn create_collection(value: NewCollectionBody) -> WrappingResponse<GetCollectionResponse> {
    fetch(
        "POST",
        "/api/v1/collection",
        Some(&value),
    ).await.unwrap_or_else(def)
}

pub async fn get_collection_books(path: &str) -> WrappingResponse<GetBookListResponse> {
    fetch(
        "GET",
        &format!("/api/v1/collection/{}/books", path),
        Option::<&()>::None
    ).await.unwrap_or_else(def)
}


// Edits

pub async fn get_edit_list(
    offset: Option<usize>,
    limit: Option<usize>
) -> WrappingResponse<GetEditListResponse> {
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
    ).await.unwrap_or_else(def)
}

pub async fn update_edit_item(id: EditId, value: &UpdateEditModel) -> WrappingResponse<PostEditResponse> {
    fetch(
        "POST",
        &format!("/api/v1/edit/{}", id),
        Some(value)
    ).await.unwrap_or_else(def)
}



// Tags

pub async fn get_tags() -> WrappingResponse<GetTagsResponse> {
    fetch(
        "GET",
        "/api/v1/tags",
        Option::<&()>::None
    ).await.unwrap_or_else(def)
}

pub async fn get_tag(id: TagId) -> WrappingResponse<GetTagResponse> {
    fetch(
        "GET",
        &format!("/api/v1/tag/{}", id),
        Option::<&()>::None
    ).await.unwrap_or_else(def)
}

pub async fn new_tag(name: String, type_of: TagType) -> WrappingResponse<NewTagResponse> {
    fetch(
        "POST",
        "/api/v1/tag",
        Some(&NewTagBody {
            name,
            type_of
        })
    ).await.unwrap_or_else(def)
}


// Book Tag

pub async fn new_book_tag(book_id: BookId, tag_id: TagId, index: Option<usize>) -> WrappingResponse<NewBookTagResponse> {
    fetch(
        "POST",
        &format!("/api/v1/tag/book/{book_id}"),
        Some(&NewBookTagBody {
            tag_id,
            index,
        })
    ).await.unwrap_or_else(def)
}

pub async fn get_book_tag(book_id: BookId, tag_id: TagId) -> WrappingResponse<GetBookTagResponse> {
    fetch(
        "GET",
        &format!("/api/v1/tag/{tag_id}/book/{book_id}"),
        Option::<&()>::None
    ).await.unwrap_or_else(def)
}

pub async fn delete_book_tag(book_id: BookId, tag_id: TagId) -> WrappingResponse<DeletionResponse> {
    fetch(
        "DELETE",
        &format!("/api/v1/tag/{tag_id}/book/{book_id}"),
        Option::<&()>::None
    ).await.unwrap_or_else(def)
}


// Image

pub async fn get_posters_for_meta(img_id_type: ImageIdType, query: Option<GetPostersQuery>) -> WrappingResponse<GetPostersResponse> {
    fetch(
        "GET",
        &format!("/api/v1/posters/{}?{}", img_id_type, query.and_then(|v| serde_qs::to_string(&v).ok()).unwrap_or_default()),
        Option::<&()>::None
    ).await.unwrap_or_else(def)
}

pub async fn change_poster_for_meta(img_id_type: ImageIdType, url_or_id: Either<String, ImageId>) -> WrappingResponse<String> {
    fetch(
        "POST",
        &format!("/api/v1/posters/{}", img_id_type),
        Some(&ChangePosterBody {
            url_or_id
        })
    ).await.unwrap_or_else(def)
}


// Member

pub async fn get_member_self() -> WrappingResponse<GetMemberSelfResponse> {
    fetch("GET", "/api/v1/member", Option::<&()>::None).await.unwrap_or_else(def)
}

pub async fn get_member_list(
    offset: Option<usize>,
    limit: Option<usize>
) -> WrappingResponse<QueryListResponse<Member>> {
    let mut url = String::from("/api/v1/members?");

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
    ).await.unwrap_or_else(def)
}


// Metadata

pub async fn update_book(id: BookId, value: &BookEdit) -> WrappingResponse<String> {
    fetch(
        "POST",
        &format!("/api/v1/book/{}", id),
        Some(value)
    ).await.unwrap_or_else(def)
}

pub async fn delete_book(id: BookId) -> WrappingResponse<bool> {
    fetch(
        "DELETE",
        &format!("/api/v1/book/{}", id),
        Option::<&()>::None
    ).await.unwrap_or_else(def)
}

pub async fn get_media_view(book_id: BookId) -> WrappingResponse<MediaViewResponse> {
    fetch(
        "GET",
        &format!("/api/v1/book/{}", book_id),
        Option::<&()>::None
    ).await.unwrap_or_else(def)
}


// External

pub async fn external_search_for(search: &str, search_for: SearchType) -> WrappingResponse<ExternalSearchResponse> {
    fetch(
        "GET",
        &format!(
            "/api/v1/external/search?query={}&search_type={}",
            urlencoding::encode(search),
            serde_json::to_string(&search_for).unwrap().replace('"', "")
        ),
        Option::<&()>::None
    ).await.unwrap_or_else(def)
}


pub async fn get_external_source_item(value: Source) -> WrappingResponse<ExternalSourceItemResponse> {
    fetch(
        "GET",
        &format!(
            "/api/v1/external/{}",
            value
        ),
        Option::<&()>::None
    ).await.unwrap_or_else(def)
}


// People


pub async fn update_person(id: PersonId, body: &PostPersonBody) {
    let _: Option<String> = fetch(
        "POST",
        &format!("/api/v1/person/{}", id),
        Some(body)
    ).await.ok();
}

pub async fn get_people(query: Option<&str>, offset: Option<usize>, limit: Option<usize>) -> WrappingResponse<GetPeopleResponse> {
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
    ).await.unwrap_or_else(def)
}

pub async fn get_person(id: PersonId) -> WrappingResponse<GetPersonResponse> {
    fetch(
        "GET",
        &format!("/api/v1/person/{}", id),
        Option::<&()>::None
    ).await.unwrap_or_else(def)
}

// Books

pub async fn new_book(value: NewBookBody) -> WrappingResponse<Option<DisplayMetaItem>> {
    fetch(
        "POST",
        "/api/v1/book",
        Some(&value)
    ).await.unwrap_or_else(def)
}


pub async fn get_books(
    offset: Option<usize>,
    limit: Option<usize>,
    search: Option<SearchQuery>,
    person_id: Option<PersonId>,
) -> WrappingResponse<GetBookListResponse> {
    let url = format!(
        "/api/v1/books?{}",
        serde_urlencoded::to_string(BookListQuery::new(offset, limit, search, person_id).unwrap()).unwrap()
    );

    fetch("GET", &url, Option::<&()>::None).await.unwrap_or_else(def)
}

pub async fn get_book_info(id: BookId) -> WrappingResponse<GetBookIdResponse> {
    fetch("GET", &format!("/api/v1/book/{}", id), Option::<&()>::None).await.unwrap_or_else(def)
}


// Searches


pub async fn get_search_list(
    offset: Option<usize>,
    limit: Option<usize>
) -> WrappingResponse<QueryListResponse<SearchGroup>> {
    let mut url = String::from("/api/v1/searches?");

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
    ).await.unwrap_or_else(def)
}

pub async fn update_search_item(id: SearchGroupId, value: PostUpdateSearchIdBody) -> WrappingResponse<String> {
    fetch(
        "POST",
        &format!("/api/v1/search/{id}"),
        Some(&value)
    ).await.unwrap_or_else(def)
}


// Options

pub async fn get_settings() -> WrappingResponse<GetSettingsResponse> {
    fetch("GET", "/api/v1/settings", Option::<&()>::None).await.unwrap_or_else(def)
}

pub async fn update_settings(value: OptionsUpdate) {
    let _: Option<String> = fetch(
        "POST",
        "/api/v1/settings",
        Some(&value)
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

// Login In

pub async fn login_with_password(email: String, password: String) -> WrappingResponse<String> {
    fetch(
        "POST",
        "/auth/password",
        Some(&json!({
            "email": email,
            "password": password,
        }))
    ).await.unwrap_or_else(def)
}

pub async fn login_without_password(email: String) -> WrappingResponse<String> {
    fetch(
        "POST",
        "/auth/passwordless",
        Some(&json!({
            "email": email,
        }))
    ).await.unwrap_or_else(def)
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

fn def<V>(e: JsValue) -> WrappingResponse<V> {
    WrappingResponse::Error(ApiErrorResponse {
        description: {
            use std::fmt::Write;

            let mut s = String::new();
            let _ = write!(&mut s, "{:?}", e);

            s
        }
    })
}