use chrono::{TimeZone, Utc};
use common::{
    api::WrappingResponse,
    component::{
        multi_select::{MultiSelectEvent, MultiSelectItem, MultiSelectModule, MultiSelectNewItem},
        popup::{
            compare::{Comparable, PopupComparison},
            Popup, PopupType,
        },
        upload::UploadModule,
    },
    util::upper_case_first_char,
    BookId, Either, ImageId, ImageIdType, PersonId, TagId, ThumbnailStore, LANGUAGES,
};
use common_local::{
    api::{GetPostersQuery, GetPostersResponse, GetTagsResponse, MediaViewResponse},
    item::edit::BookEdit,
    Person, SearchType, TagFE, TagType,
};

use js_sys::Date;
use wasm_bindgen::{JsCast, JsValue, UnwrapThrowExt};
use web_sys::{HtmlInputElement, HtmlSelectElement, HtmlTextAreaElement};
use yew::{html::Scope, prelude::*};

use crate::{
    components::{
        popup::{
            search::SearchSelectedValue,
            search_person::SearchSelectedValue as PersonSearchSelectedValue, SearchBy,
        },
        LoginBarrier, PopupEditMetadata, PopupSearch, PopupSearchPerson,
    },
    get_member_self, request,
};

#[derive(Clone)]
pub enum Msg {
    // Retrieve
    RetrieveMediaView(Box<WrappingResponse<MediaViewResponse>>),
    RetrievePosters(WrappingResponse<GetPostersResponse>),

    MultiselectToggle(bool, TagId),
    MultiselectCreate(TagType, MultiSelectNewItem<TagId>),
    MultiCreateResponse(TagFE),
    AllTagsResponse(WrappingResponse<GetTagsResponse>),

    ReloadPosters,
    TogglePosterMetaSearch,
    UpdatePoster(BookId, Either<String, ImageId>),

    // Events
    ToggleEdit,
    SaveEdits,
    UpdateEditing(ChangingType, String),

    OnDelete(WrappingResponse<bool>),

    ShowPopup(DisplayOverlay),
    ClosePopup,

    Ignore,
}

#[derive(Properties, PartialEq, Eq)]
pub struct Property {
    pub id: BookId,
}

pub struct BookView {
    media: Option<WrappingResponse<MediaViewResponse>>,
    cached_posters: Option<WrappingResponse<GetPostersResponse>>,

    media_popup: Option<DisplayOverlay>,

    /// If we're currently editing. This'll be set.
    editing_item: BookEdit,
    is_editing: bool,

    // Multiselect Values
    cached_tags: Vec<CachedTag>,

    search_poster_metadata: bool,
}

impl Component for BookView {
    type Message = Msg;
    type Properties = Property;

    fn create(ctx: &Context<Self>) -> Self {
        ctx.link()
            .send_future(async { Msg::AllTagsResponse(request::get_tags().await) });

        Self {
            media: None,
            cached_posters: None,
            media_popup: None,
            editing_item: BookEdit::default(),
            is_editing: false,

            cached_tags: Vec::new(),

            search_poster_metadata: false,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Ignore => return false,

            // Multiselect
            Msg::MultiselectToggle(inserted, tag_id) => {
                if let Some(curr_book) = self.media.as_ref().and_then(|v| v.as_ok().ok()) {
                    if inserted {
                        // If the tag is in the db model.
                        if curr_book.tags.iter().any(|bt| bt.tag.id == tag_id) {
                            // We have to make sure it's on in the "removed_tags" vec
                            self.editing_item.remove_tag(tag_id);
                        } else {
                            self.editing_item.insert_added_tag(tag_id);
                        }
                    } else {
                        // If the tag is in the db model.
                        if curr_book.tags.iter().any(|bt| bt.tag.id == tag_id) {
                            self.editing_item.insert_removed_tag(tag_id);
                        } else {
                            // We have to make sure it's not in the "added_tags" vec
                            self.editing_item.remove_tag(tag_id);
                        }
                    }
                }
            }

            Msg::MultiselectCreate(type_of, item) => match &type_of {
                TagType::Genre | TagType::Subject => {
                    ctx.link().send_future(async move {
                        let tag_resp = request::new_tag(item.name.clone(), type_of).await;

                        match tag_resp.ok() {
                            Ok(tag_resp) => {
                                item.register.emit(tag_resp.id);

                                Msg::MultiCreateResponse(tag_resp)
                            }

                            Err(err) => {
                                log::error!("{err}");

                                Msg::Ignore
                            }
                        }
                    });
                }

                _ => unimplemented!("Msg::MultiselectCreate {:?}", type_of),
            },

            Msg::MultiCreateResponse(tag) => {
                // Add original tag to cache.
                if !self.cached_tags.iter().any(|v| v.id == tag.id) {
                    self.cached_tags.push(CachedTag {
                        type_of: tag.type_of.clone(),
                        name: tag.name.clone(),
                        id: tag.id,
                    });

                    self.cached_tags
                        .sort_unstable_by(|a, b| a.name.partial_cmp(&b.name).unwrap());
                }

                self.editing_item.insert_added_tag(tag.id);
            }

            Msg::AllTagsResponse(resp) => {
                let resp = resp.ok().unwrap_throw();

                self.cached_tags = resp
                    .items
                    .into_iter()
                    .map(|v| CachedTag {
                        id: v.id,
                        type_of: v.type_of,
                        name: v.name,
                    })
                    .collect();

                self.cached_tags
                    .sort_unstable_by(|a, b| a.name.partial_cmp(&b.name).unwrap());
            }

            Msg::UpdatePoster(book_id, url_or_id) => {
                ctx.link().send_future(async move {
                    request::change_poster_for_meta(ImageIdType::new_book(book_id), url_or_id)
                        .await;

                    Msg::ReloadPosters
                });

                return false;
            }

            Msg::ReloadPosters => {
                if let Some(curr_book) = self.media.as_ref().and_then(|v| v.as_ok().ok()) {
                    let book_id = curr_book.metadata.id;

                    let search_metadata = self.search_poster_metadata;

                    ctx.link().send_future(async move {
                        Msg::RetrievePosters(
                            request::get_posters_for_meta(
                                ImageIdType::new_book(book_id),
                                Some(GetPostersQuery { search_metadata }),
                            )
                            .await,
                        )
                    });

                    return false;
                }
            }

            Msg::TogglePosterMetaSearch => {
                self.search_poster_metadata = !self.search_poster_metadata;
                ctx.link().send_message(Msg::ReloadPosters);
            }

            // Edits
            Msg::ToggleEdit => {
                // Is currently editing? We won't be.
                if self.is_editing {
                    self.editing_item = BookEdit::default();
                } else if self.cached_posters.is_none() {
                    ctx.link().send_message(Msg::ReloadPosters);
                }

                self.is_editing = !self.is_editing;
            }

            Msg::SaveEdits => {
                if let Some(curr_book) = self.media.as_ref().and_then(|v| v.as_ok().ok()) {
                    let edit = self.editing_item.clone();

                    let book_id = curr_book.metadata.id;

                    ctx.link().send_future(async move {
                        request::update_book(book_id, &edit).await;

                        Msg::RetrieveMediaView(Box::new(request::get_media_view(book_id).await))
                    });
                }
            }

            Msg::UpdateEditing(type_of, value) => {
                let mut updating = &mut self.editing_item;

                let value = Some(value).filter(|v| !v.trim().is_empty());

                match type_of {
                    ChangingType::Title => updating.title = value,
                    ChangingType::OriginalTitle => updating.clean_title = value,
                    ChangingType::Description => updating.description = value,
                    // ChangingType::Rating => updating.rating = value.and_then(|v| v.parse().ok()),
                    // ChangingType::ThumbPath => todo!(),
                    ChangingType::AvailableAt => {
                        updating.available_at = value.map(|v| {
                            let date = Date::new(&JsValue::from_str(&v));
                            date.get_seconds() as i64
                        })
                    }
                    ChangingType::Language => {
                        updating.language = value.and_then(|v| v.parse().ok())
                    }
                    ChangingType::Isbn10 => updating.isbn_10 = value,
                    ChangingType::Isbn13 => updating.isbn_13 = value,
                    ChangingType::Publicity => {
                        updating.is_public = value.and_then(|v| v.parse().ok())
                    }

                    ChangingType::PersonRelation(id) => {
                        updating.insert_updated_person(id, value);
                    }

                    ChangingType::PersonDisplayed(id) => {
                        updating.display_person_id = Some(id);
                    }

                    ChangingType::PersonAdding(id) => {
                        updating.insert_added_person(id);
                        self.media_popup = None;
                    }
                }
            }

            Msg::OnDelete(resp) => {
                match resp {
                    WrappingResponse::Resp(okay) => {
                        if okay {
                            // TODO: Deletion View.
                            self.media = Some(WrappingResponse::error("Deleted..."));
                        }
                    }

                    WrappingResponse::Error(e) => self.media = Some(WrappingResponse::Error(e)),
                }
            }

            // Popup
            Msg::ClosePopup => {
                self.media_popup = None;
            }

            Msg::ShowPopup(new_disp) => {
                if let Some(old_disp) = self.media_popup.as_mut() {
                    if *old_disp == new_disp {
                        self.media_popup = None;
                    } else {
                        self.media_popup = Some(new_disp);
                    }
                } else {
                    self.media_popup = Some(new_disp);
                }
            }

            Msg::RetrievePosters(value) => {
                // TODO: Simplify
                if let Some(WrappingResponse::Resp(book)) = self.media.as_mut() {
                    if let Ok(posters) = value.as_ok() {
                        if let Some(poster) = posters.items.iter().find(|v| v.selected) {
                            book.metadata.thumb_path = ThumbnailStore::Path(
                                poster.path.rsplit_once('/').unwrap().1.to_string(),
                            );
                        }
                    }
                }

                self.cached_posters = Some(value);
            }

            Msg::RetrieveMediaView(value) => {
                self.media = Some(*value);

                if self.is_editing && self.cached_posters.is_none() {
                    ctx.link().send_message(Msg::ReloadPosters);
                }
            }
        }

        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        if self.is_editing {
            self.render_editing(ctx)
        } else {
            self.render_viewing(ctx)
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            let metadata_id = ctx.props().id;

            ctx.link().send_future(async move {
                Msg::RetrieveMediaView(Box::new(request::get_media_view(metadata_id).await))
            });

            if let Some(member) = get_member_self() {
                if member.localsettings.get_page_view_default().is_editing() {
                    self.is_editing = true;
                }
            }
        }
    }

    fn changed(&mut self, _ctx: &Context<Self>) -> bool {
        if let Some(member) = get_member_self() {
            if member.localsettings.get_page_view_default().is_editing() {
                self.is_editing = true;
            }
        }

        true
    }
}

impl BookView {
    fn render_editing(&self, ctx: &Context<Self>) -> Html {
        if let Some(resp) = self.media.as_ref() {
            let book_resp @ MediaViewResponse {
                people,
                metadata: book_model,
                tags,
            } = crate::continue_or_html_err!(resp);

            let editing = &self.editing_item;

            let book_id = book_model.id;

            let on_click_more = ctx.link().callback(move |e: MouseEvent| {
                e.prevent_default();
                e.stop_propagation();

                Msg::ShowPopup(DisplayOverlay::More {
                    book_id,
                    mouse_pos: (e.page_x(), e.page_y()),
                })
            });

            html! {
                <div class="outer-view-container">
                    <div class="sidebar-container">
                        <div class="sidebar-item">
                            <button class="button" onclick={ctx.link().callback(|_| Msg::ToggleEdit)}>{"Stop Editing"}</button>
                        </div>
                        <div class="sidebar-item">
                            <button class="button proceed" onclick={ctx.link().callback(|_| Msg::SaveEdits)}>
                                {"Save"}
                            </button>
                        </div>
                    </div>

                    <div class="view-container item-view-container">
                        <div class="info-container">
                            <div class="poster large">
                                <LoginBarrier>
                                    <div class="bottom-right">
                                        <span class="material-icons" onclick={on_click_more} title="More Options">{ "more_horiz" }</span>
                                    </div>

                                    <div class="bottom-left">
                                        <span class="material-icons" onclick={ctx.link().callback_future(move |e: MouseEvent| {
                                            e.prevent_default();
                                            e.stop_propagation();

                                            async move {
                                                Msg::ShowPopup(DisplayOverlay::Edit(Box::new(request::get_media_view(book_id).await)))
                                            }
                                        })} title="More Options">{ "edit" }</span>
                                    </div>
                                </LoginBarrier>

                                <img src={ book_model.get_thumb_url() } />
                            </div>

                            <div class="metadata-container">
                                <div class="metadata">
                                    // Book Display Info
                                    <h5>{ "Book Display Info" }</h5>

                                    <span class="sub-title">{"Publicity"}</span>
                                    <select
                                        class="title"
                                        type="text"
                                        onchange={Self::on_change_select(ctx.link(), ChangingType::Publicity)}
                                    >
                                        <option selected={editing.is_public.unwrap_or(book_model.is_public)} value="true">
                                            {"Public"}
                                        </option>
                                        <option selected={!editing.is_public.unwrap_or(book_model.is_public)} value="false">
                                            {"Unlisted"}
                                        </option>
                                    </select>

                                    <span class="sub-title">{"Title"}</span>
                                    <input class="title" type="text"
                                        onchange={Self::on_change_input(ctx.link(), ChangingType::Title)}
                                        value={ editing.title.clone().or_else(|| book_model.title.clone()).unwrap_or_default() }
                                    />

                                    <span class="sub-title">{"Original Title"}</span>
                                    <input class="title" type="text"
                                        onchange={Self::on_change_input(ctx.link(), ChangingType::OriginalTitle)}
                                        value={ editing.clean_title.clone().or_else(|| book_model.clean_title.clone()).unwrap_or_default() }
                                    />

                                    <span class="sub-title">{"Description"}</span>
                                    <textarea
                                        rows="9"
                                        cols="30"
                                        class="description"
                                        onchange={Self::on_change_textarea(ctx.link(), ChangingType::Description)}
                                        value={ editing.description.clone().or_else(|| book_model.description.clone()).unwrap_or_default() }
                                    />
                                </div>

                                // Book Info
                                <div class="metadata">
                                    <h5>{ "Book Info" }</h5>

                                    <span class="sub-title">{"Available At"}</span>
                                    <input class="title" type="text"
                                        placeholder="YYYY-MM-DD"
                                        onchange={Self::on_change_input(ctx.link(), ChangingType::AvailableAt)}
                                        value={ editing.available_at.map(|v| Utc.timestamp(v, 0).date_naive()).or(book_model.available_at).map(|v| v.format("%Y-%m-%d").to_string()).unwrap_or_default() }
                                    />

                                    <span class="sub-title">{"ISBN 10"}</span>
                                    <input class="title" type="text"
                                        onchange={Self::on_change_input(ctx.link(), ChangingType::Isbn10)}
                                        value={ editing.isbn_10.clone().or_else(|| book_model.isbn_10.clone()).unwrap_or_default() }
                                    />

                                    <span class="sub-title">{"ISBN 13"}</span>
                                    <input class="title" type="text"
                                        onchange={Self::on_change_input(ctx.link(), ChangingType::Isbn13)}
                                        value={ editing.isbn_13.clone().or_else(|| book_model.isbn_13.clone()).unwrap_or_default() }
                                    />

                                    <span class="sub-title">{"Publisher"}</span>
                                    <input class="title" type="text" />

                                    <span class="sub-title">{"Language"}</span>
                                    <select
                                        class="title"
                                        type="text"
                                        onchange={Self::on_change_select(ctx.link(), ChangingType::Language)}
                                    >
                                        {
                                            for LANGUAGES.iter()
                                                .enumerate()
                                                .map(|(index, lang)| {
                                                    let selected = editing.language.unwrap_or(book_model.language) == index as u16;

                                                    html! {
                                                        <option
                                                            {selected}
                                                            value={index.to_string()}
                                                        >
                                                            { upper_case_first_char(lang.to_string()) }
                                                        </option>
                                                    }
                                                })
                                        }
                                    </select>
                                </div>

                                // Sources
                                <div class="metadata">
                                    <h5>{ "Sources" }</h5>

                                    <span class="sub-title">{ "Good Reads URL" }</span>
                                    <input class="title" type="text" />

                                    <span class="sub-title">{ "Open Library URL" }</span>
                                    <input class="title" type="text" />

                                    <span class="sub-title">{ "Google Books URL" }</span>
                                    <input class="title" type="text" />

                                    <h5>{ "Tags" }</h5>

                                    <span class="sub-title">{ "Genre" }</span>
                                    <MultiSelectModule<TagId>
                                        editing=true
                                        on_event={
                                            ctx.link().callback(|v| match v {
                                                MultiSelectEvent::Toggle { toggle, id } => {
                                                    Msg::MultiselectToggle(toggle, id)
                                                }

                                                MultiSelectEvent::Create(new_item) => {
                                                    Msg::MultiselectCreate(TagType::Genre, new_item)
                                                }

                                                _ => Msg::Ignore
                                            })
                                        }
                                    >
                                        {
                                            for self.cached_tags
                                                .iter()
                                                .filter(|v| v.type_of.into_u8() == TagType::Genre.into_u8())
                                                .map(|tag| {
                                                    let mut filtered_tags = tags.iter()
                                                        // We only need the tag ids
                                                        .map(|bt| bt.tag.id)
                                                        // Filter out editing "removed tags"
                                                        .filter(|tag_id| !editing.removed_tags.as_ref().map(|v| v.iter().any(|r| r == tag_id)).unwrap_or_default())
                                                        // Chain into editing "added tags"
                                                        .chain(editing.added_tags.iter().flat_map(|v| v.iter()).copied());

                                                    html_nested! {
                                                        <MultiSelectItem<TagId> name={tag.name.clone()} id={tag.id} selected={filtered_tags.any(|tag_id| tag_id == tag.id)} />
                                                    }
                                                })
                                        }
                                    </MultiSelectModule<TagId>>

                                    <span class="sub-title">{ "Subject" }</span>

                                    <MultiSelectModule<TagId>
                                        editing=true
                                        on_event={
                                            ctx.link().callback(|v| match v {
                                                MultiSelectEvent::Toggle { toggle, id } => {
                                                    Msg::MultiselectToggle(toggle, id)
                                                }

                                                MultiSelectEvent::Create(new_item) => {
                                                    Msg::MultiselectCreate(TagType::Subject, new_item)
                                                }

                                                _ => Msg::Ignore
                                            })
                                        }
                                    >
                                        {
                                            for self.cached_tags
                                                .iter()
                                                .filter(|v| v.type_of.into_u8() == TagType::Subject.into_u8())
                                                .map(|tag| {
                                                    let mut filtered_tags = tags.iter()
                                                        // We only need the tag ids
                                                        .map(|bt| bt.tag.id)
                                                        // Filter out editing "removed tags"
                                                        .filter(|tag_id| !editing.removed_tags.as_ref().map(|v| v.iter().any(|r| r == tag_id)).unwrap_or_default())
                                                        // Chain into editing "added tags"
                                                        .chain(editing.added_tags.iter().flat_map(|v| v.iter()).copied());

                                                    html_nested! {
                                                        <MultiSelectItem<TagId> name={tag.name.clone()} id={tag.id} selected={filtered_tags.any(|tag_id| tag_id == tag.id)} />
                                                    }
                                                })
                                        }
                                    </MultiSelectModule<TagId>>
                                </div>
                            </div>
                        </div>

                        { // Posters
                            if let Some(resp) = self.cached_posters.as_ref() {
                                match resp.as_ok() {
                                    Ok(resp) => html! {
                                        <section>
                                            <h2>{ "Posters" }</h2>
                                            <div class="posters-container">
                                                <UploadModule
                                                    class="poster new-container"
                                                    title="Add Poster"
                                                    upload_url={ format!("/api/v1/posters/{}/upload", ImageIdType::new_book(ctx.props().id)) }
                                                    on_upload={ctx.link().callback(|_| Msg::ReloadPosters)}
                                                >
                                                    <span class="material-icons">{ "add" }</span>
                                                </UploadModule>

                                                {
                                                    if !self.search_poster_metadata {
                                                        html! {
                                                            <div
                                                                class="poster new-container"
                                                                title="Search Posters By Name"
                                                                onclick={ ctx.link().callback(|_| Msg::TogglePosterMetaSearch) }
                                                            >
                                                                <span><b>{ "Search For" }</b></span>
                                                            </div>
                                                        }
                                                    } else {
                                                        html! {}
                                                    }
                                                }

                                                {
                                                    for resp.items.iter().map(move |poster| {
                                                        let url_or_id = poster.id.map(Either::Right).unwrap_or_else(|| Either::Left(poster.path.clone()));
                                                        let is_selected = poster.selected;

                                                        html! {
                                                            <div
                                                                class={ classes!("poster", { if is_selected { "selected" } else { "" } }) }
                                                                onclick={ ctx.link().callback_future(move |_| {
                                                                    let url_or_id = url_or_id.clone();

                                                                    async move {
                                                                        if is_selected {
                                                                            Msg::Ignore
                                                                        } else {
                                                                            Msg::UpdatePoster(book_id, url_or_id)
                                                                        }
                                                                    }
                                                                }) }
                                                            >
                                                                <div class="top-right">
                                                                    <span
                                                                        class="material-icons"
                                                                    >{ "delete" }</span>
                                                                </div>
                                                                <img src={poster.path.clone()} />
                                                            </div>
                                                        }
                                                    })
                                                }
                                            </div>
                                        </section>
                                    },

                                    Err(e) => html! {
                                        <h2>{ e }</h2>
                                    }
                                }
                            } else {
                                html! {}
                            }
                        }

                        <section>
                            <h2>{ "Characters" }</h2>
                            <div class="characters-container">
                                <div class="person-container new-container" title="Add Book Character">
                                    <span class="material-icons">{ "add" }</span>
                                </div>
                            </div>
                        </section>

                        <section>
                            <h2>{ "People" }</h2>
                            <div class="authors-container">
                                <div class="person-container new-container" title="Add Person">
                                    <span class="material-icons" onclick={ ctx.link().callback(|_| Msg::ShowPopup(DisplayOverlay::AddAuthor)) }>{ "add" }</span>
                                </div>

                                {
                                    for people.iter().cloned().map(|person| html! {
                                        <PersonItem
                                            edited_info={ match editing.updated_people.as_ref() {
                                                Some(v) => v.iter().find_map(|v| if v.0 == person.id { Some(v.1.clone()) } else { None }).flatten(),
                                                None => None
                                            } }
                                            {person}
                                            display_id={ book_model.cached.author_id }
                                            editing={ true }
                                            scope={ ctx.link() }
                                        />
                                    })
                                }
                            </div>
                        </section>
                    </div>

                    { self.render_popup(book_resp, ctx) }
                </div>
            }
        } else {
            html! {
                <h1>{ "Loading..." }</h1>
            }
        }
    }

    fn render_viewing(&self, ctx: &Context<Self>) -> Html {
        if let Some(resp) = self.media.as_ref() {
            let book_resp @ MediaViewResponse {
                people,
                metadata: book_model,
                tags,
            } = crate::continue_or_html_err!(resp);

            let book_id = book_model.id;

            let on_click_more = ctx.link().callback(move |e: MouseEvent| {
                e.prevent_default();
                e.stop_propagation();

                Msg::ShowPopup(DisplayOverlay::More {
                    book_id,
                    mouse_pos: (e.page_x(), e.page_y()),
                })
            });

            html! {
                <div class="outer-view-container">
                    <div class="sidebar-container">
                        <LoginBarrier>
                            <div class="sidebar-item">
                                <button class="button" onclick={ctx.link().callback(|_| Msg::ToggleEdit)}>{"Start Editing"}</button>
                            </div>
                        </LoginBarrier>
                    </div>

                    <div class="view-container item-view-container">
                        <div class="info-container">
                            <div class="poster large">
                                <LoginBarrier>
                                    <div class="bottom-right">
                                        <span class="material-icons" onclick={on_click_more} title="More Options">{ "more_horiz" }</span>
                                    </div>

                                    <div class="bottom-left">
                                        <span class="material-icons" onclick={ctx.link().callback_future(move |e: MouseEvent| {
                                            e.prevent_default();
                                            e.stop_propagation();

                                            async move {
                                                Msg::ShowPopup(DisplayOverlay::Edit(Box::new(request::get_media_view(book_id).await)))
                                            }
                                        })} title="More Options">{ "edit" }</span>
                                    </div>
                                </LoginBarrier>

                                <img src={ book_model.get_thumb_url() } />
                            </div>

                            <div class="metadata-container">
                                <div class="metadata">
                                    <div class="label-group">
                                        {
                                            if book_model.is_public {
                                                html! {
                                                    <div class="label green">{ "Public" }</div>
                                                }
                                            } else {
                                                html! {
                                                    <div class="label red">{ "Unlisted" }</div>
                                                }
                                            }
                                        }
                                    </div>

                                    // Book Display Info
                                    <h3 class="title">{ book_model.get_title() }</h3>
                                    <p class="description">{ book_model.description.clone().unwrap_or_default() }</p>

                                    <h4>{ "Genre" }</h4>
                                    <div class="label-group">
                                        {
                                            for self.cached_tags
                                                .iter()
                                                .filter(|v| v.type_of.into_u8() == TagType::Genre.into_u8() && tags.iter().any(|bt| bt.tag.id == v.id))
                                                .take(5) // TODO: Temp. Need to add max-size to div container.
                                                .map(|tag| {
                                                    html! {
                                                        <div class="label">{ tag.name.clone() }</div>
                                                    }
                                                })
                                        }

                                        { "..." }
                                    </div>

                                    <h4>{ "Subject" }</h4>
                                    <div class="label-group">
                                        {
                                            for self.cached_tags
                                                .iter()
                                                .filter(|v| v.type_of.into_u8() == TagType::Subject.into_u8() && tags.iter().any(|bt| bt.tag.id == v.id))
                                                .take(5) // TODO: Temp. Need to add max-size to div container.
                                                .map(|tag| {
                                                    html! {
                                                        <div class="label">{ tag.name.clone() }</div>
                                                    }
                                                })
                                        }

                                        { "..." }
                                    </div>
                                </div>

                                // Book Info

                                // Sources
                            </div>
                        </div>

                        <section>
                            <h2>{ "Characters" }</h2>
                            <div class="characters-container">
                                //
                            </div>
                        </section>

                        <section>
                            <h2>{ "People" }</h2>
                            <div class="authors-container">
                            {
                                for people.iter().cloned().map(|person| {
                                    html! {
                                        <PersonItem
                                            {person}
                                            display_id={ book_model.cached.author_id }
                                            editing={ false }
                                            scope={ ctx.link() }
                                        />
                                    }
                                })
                            }
                            </div>
                        </section>
                    </div>

                    { self.render_popup(book_resp, ctx) }
                </div>
            }
        } else {
            html! {
                <h1>{ "Loading..." }</h1>
            }
        }
    }

    fn render_popup(
        &self,
        MediaViewResponse {
            metadata: book_model,
            ..
        }: &MediaViewResponse,
        ctx: &Context<Self>,
    ) -> Html {
        if let Some(overlay_type) = self.media_popup.as_ref() {
            let book_id = book_model.id;

            match overlay_type {
                &DisplayOverlay::More { mouse_pos, .. } => {
                    html! {
                        <Popup type_of={ PopupType::AtPoint(mouse_pos.0, mouse_pos.1) } on_close={ctx.link().callback(|_| Msg::ClosePopup)}>
                            <div class="menu-list">
                                // <div class="menu-item" yew-close-popup="" onclick={
                                //     Self::on_click_prevdef(ctx.link(), Msg::UpdateBook(book_id))
                                // }>{ "Refresh Metadata" }</div>
                                <div class="menu-item" yew-close-popup="" onclick={
                                    Self::on_click_prevdef_stopprop(ctx.link(), Msg::ShowPopup(DisplayOverlay::SearchForBook { input_value: None }))
                                }>{ "Search New Metadata" }</div>
                                <div class="menu-item" yew-close-popup="" onclick={ ctx.link().callback_future(move |_| async move {
                                    Msg::OnDelete(request::delete_book(book_id).await)
                                }) }>{ "Delete" }</div>
                            </div>
                        </Popup>
                    }
                }

                DisplayOverlay::Edit(resp) => match resp.as_ok() {
                    Ok(resp) => html! {
                        <PopupEditMetadata
                            on_close={ ctx.link().callback(|_| Msg::ClosePopup) }
                            classes={ classes!("popup-book-edit") }
                            media_resp={ resp.clone() }
                        />
                    },

                    Err(e) => html! {
                        <h2>{ e }</h2>
                    },
                },

                DisplayOverlay::EditFromMetadata(new_meta) => {
                    html! {
                        <PopupComparison
                            on_close={ ctx.link().callback(|_| Msg::ClosePopup) }
                            on_submit={ ctx.link().callback_future(move |v| async move {
                                request::update_book(book_id, &BookEdit::create_from_comparison(v).unwrap_throw()).await;
                                Msg::Ignore
                            }) }
                            classes={ classes!("popup-book-edit") }
                            compare={ BookEdit::from(book_model.clone()).create_comparison_with(&**new_meta).unwrap_throw() }
                        />
                    }
                }

                DisplayOverlay::SearchForBook { input_value } => {
                    let input_value = if let Some(v) = input_value {
                        v.to_string()
                    } else {
                        format!(
                            "{} {}",
                            book_model.title.as_deref().unwrap_or_default(),
                            book_model.cached.author.as_deref().unwrap_or_default()
                        )
                    };

                    let input_value = input_value.trim().to_string();

                    html! {
                        <PopupSearch
                            {input_value}
                            type_of={ SearchBy::External }
                            search_for={ SearchType::Book }
                            on_close={ ctx.link().callback(|_| Msg::ClosePopup) }
                            on_select={ ctx.link().callback_future(|value: SearchSelectedValue| async {
                                Msg::ShowPopup(DisplayOverlay::EditFromMetadata(
                                    match value.into_external() {
                                        Either::Left(source) => {
                                            let resp = request::get_external_source_item(source).await.ok().unwrap_throw();

                                            Box::new(resp.item.unwrap().into())
                                        }

                                        Either::Right(book) => Box::new(book),
                                    }
                                ))
                            }) }
                        />
                    }
                }

                DisplayOverlay::AddAuthor => {
                    html! {
                        <PopupSearchPerson
                            type_of={ SearchBy::Local }
                            on_close={ ctx.link().callback(|_| Msg::ClosePopup) }
                            on_select={ ctx.link().callback_future(move |value: PersonSearchSelectedValue| async move {
                                if let PersonSearchSelectedValue::PersonId(id) = value {
                                    Msg::UpdateEditing(ChangingType::PersonAdding(id), String::new())
                                } else {
                                    Msg::Ignore
                                }
                            }) }
                        />
                    }
                }
            }
        } else {
            html! {}
        }
    }

    fn on_change_select(scope: &Scope<Self>, updating: ChangingType) -> Callback<Event> {
        scope.callback(move |e: Event| {
            Msg::UpdateEditing(
                updating,
                e.target()
                    .unwrap()
                    .dyn_into::<HtmlSelectElement>()
                    .unwrap()
                    .value(),
            )
        })
    }

    fn on_change_input(scope: &Scope<Self>, updating: ChangingType) -> Callback<Event> {
        scope.callback(move |e: Event| {
            Msg::UpdateEditing(
                updating,
                e.target()
                    .unwrap()
                    .dyn_into::<HtmlInputElement>()
                    .unwrap()
                    .value(),
            )
        })
    }

    fn on_change_textarea(scope: &Scope<Self>, updating: ChangingType) -> Callback<Event> {
        scope.callback(move |e: Event| {
            Msg::UpdateEditing(
                updating,
                e.target()
                    .unwrap()
                    .dyn_into::<HtmlTextAreaElement>()
                    .unwrap()
                    .value(),
            )
        })
    }

    /// A Callback which calls "prevent_default" and "stop_propagation"
    fn on_click_prevdef_stopprop(scope: &Scope<Self>, msg: Msg) -> Callback<MouseEvent> {
        scope.callback(move |e: MouseEvent| {
            e.prevent_default();
            e.stop_propagation();
            msg.clone()
        })
    }
}

#[derive(Properties)]
struct PersonItemProps {
    person: Person,

    display_id: Option<PersonId>,

    #[prop_or_default]
    edited_info: Option<String>,

    #[prop_or_default]
    editing: bool,

    scope: Scope<BookView>,
}

impl PartialEq for PersonItemProps {
    fn eq(&self, other: &Self) -> bool {
        self.person == other.person && self.edited_info == other.edited_info
    }
}

#[function_component(PersonItem)]
fn _person_item(props: &PersonItemProps) -> Html {
    html! {
        <div class="person-container">
            <div class="photo">
                <img src={ props.person.get_thumb_url() } />

                {
                    if props.editing {
                        if props.display_id.filter(|v| *v == props.person.id).is_some() {
                            html! {
                                <div class="top-right">
                                    <span class="material-icons" title="Current Display User">{ "check_circle" }</span>
                                </div>
                            }
                        } else {
                            let id = props.person.id;

                            html! {
                                <div class="top-right hover">
                                    <span
                                        class="material-icons"
                                        title="Set as Display User"
                                        onclick={ props.scope.callback(move |_| Msg::UpdateEditing(ChangingType::PersonDisplayed(id), String::new())) }
                                    >{ "upgrade" }</span>
                                </div>
                            }
                        }
                    } else {
                        html! {}
                    }
                }
            </div>
            <span class="title">{ props.person.name.clone() }</span>
            {
                if props.editing {
                    html! {
                        <input
                            type="text"
                            placeholder="Relation"
                            onchange={ BookView::on_change_input(&props.scope, ChangingType::PersonRelation(props.person.id)) }
                            value={ props.edited_info.clone().or_else(|| props.person.info.clone()).unwrap_or_default() }
                        />
                    }
                } else {
                    html! {
                        <span class="title">{ props.person.info.clone().unwrap_or_default() }</span>
                    }
                }
            }
        </div>
    }
}

#[derive(Debug, Clone)]
pub struct CachedTag {
    type_of: TagType,
    id: TagId,
    name: String,
}

#[derive(Clone, Copy)]
pub enum ChangingType {
    Title,
    OriginalTitle,
    Description,
    // Rating,
    // ThumbPath,
    AvailableAt,
    Language,
    Isbn10,
    Isbn13,
    Publicity,

    PersonDisplayed(PersonId),
    PersonRelation(PersonId),
    PersonAdding(PersonId),
}

#[derive(Clone)]
pub enum DisplayOverlay {
    Edit(Box<WrappingResponse<MediaViewResponse>>),

    EditFromMetadata(Box<BookEdit>),

    More {
        book_id: BookId,
        mouse_pos: (i32, i32),
    },

    SearchForBook {
        input_value: Option<String>,
    },

    AddAuthor,
}

impl PartialEq for DisplayOverlay {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::More { book_id: l_id, .. }, Self::More { book_id: r_id, .. }) => l_id == r_id,
            (
                Self::SearchForBook {
                    input_value: l_val, ..
                },
                Self::SearchForBook {
                    input_value: r_val, ..
                },
            ) => l_val == r_val,

            _ => false,
        }
    }
}
