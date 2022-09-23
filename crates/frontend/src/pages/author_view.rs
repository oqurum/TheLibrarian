// TODO: Temporary. Some of the dead_code in here will be used.
#![allow(dead_code)]

use std::str::FromStr;

use chrono::NaiveDate;
use common::{component::upload::UploadModule, Either, PersonId, ImageIdType, api::WrappingResponse};
use common_local::{api::{self, GetPostersResponse, GetPersonResponse, BookListQuery}, TagType, item::edit::PersonEdit};
use wasm_bindgen::JsCast;
use web_sys::{HtmlInputElement, HtmlTextAreaElement};
use yew::{prelude::*, html::Scope};

use crate::{components::LoginBarrier, request, pages::home::MediaItem, get_member_self};



#[derive(Clone)]
pub enum Msg {
    // Retrive
    RetrieveMediaView(Box<WrappingResponse<GetPersonResponse>>),
    RetrievePosters(WrappingResponse<GetPostersResponse>),
    BooksListResults(WrappingResponse<api::GetBookListResponse>),

    UpdatedPoster,

    // Events
    ToggleEdit,
    SaveEdits,
    UpdateEditing(ChangingType, String),

    Ignore
}

#[derive(Properties, PartialEq, Eq)]
pub struct Property {
    pub id: PersonId,
}

pub struct AuthorView {
    media: Option<WrappingResponse<GetPersonResponse>>,
    cached_posters: Option<WrappingResponse<GetPostersResponse>>,
    cached_books: Option<WrappingResponse<api::GetBookListResponse>>,

    media_popup: Option<DisplayOverlay>,

    editing_item: PersonEdit,
    is_editing: bool,
}

impl Component for AuthorView {
    type Message = Msg;
    type Properties = Property;

    fn create(ctx: &Context<Self>) -> Self {
        let person_id = ctx.props().id;

        ctx.link()
        .send_future(async move {
            let resp = request::get_books(BookListQuery {
                search: Some(api::QueryType::Person(person_id)),
                .. Default::default()
            }).await;

            Msg::BooksListResults(resp)
        });

        Self {
            media: None,
            cached_posters: None,
            cached_books: None,

            media_popup: None,
            editing_item: PersonEdit::default(),
            is_editing: false,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Ignore => return false,

            Msg::BooksListResults(resp) => {
                self.cached_books = Some(resp);
            }

            Msg::UpdatedPoster => if let Some(book) = self.media.as_ref().and_then(|v| v.as_ok().ok()) {
                let person_id = ImageIdType::new_person(book.person.id);

                ctx.link()
                .send_future(async move {
                    Msg::RetrievePosters(request::get_posters_for_meta(person_id, None).await)
                });

                return false;
            }

            // Edits
            Msg::ToggleEdit => if let Some(book) = self.media.as_ref().and_then(|v| v.as_ok().ok()) {
                self.is_editing = !self.is_editing;
                self.editing_item = PersonEdit::default();

                if self.is_editing && self.cached_posters.is_none() {
                    let person_id = ImageIdType::new_person(book.person.id);

                    ctx.link()
                    .send_future(async move {
                        Msg::RetrievePosters(request::get_posters_for_meta(person_id, None).await)
                    });
                }
            }

            Msg::SaveEdits => {
                let person = &self.media.as_ref().and_then(|v| v.as_ok().ok()).unwrap().person;

                let edit = self.editing_item.clone();
                let person_id = person.id;

                ctx.link()
                .send_future(async move {
                    request::update_person(person_id, &api::PostPersonBody::Edit(edit)).await;

                    Msg::RetrieveMediaView(Box::new(request::get_person(person_id).await))
                });
            }

            Msg::UpdateEditing(type_of, value) => {
                let value = Some(value).filter(|v| !v.trim().is_empty());

                match type_of {
                    ChangingType::Name => self.editing_item.name = value,
                    ChangingType::Description => self.editing_item.description = value,
                    ChangingType::BirthDate => {
                        let value = value.map(|mut v| {
                            // TODO: Separate into own function, handle errors properly.

                            let dashes = v.chars().filter(|v|  *v == '-').count();

                            if dashes == 0 {
                                v.push_str("-01");
                            }

                            if dashes <= 1 {
                                v.push_str("-01");
                            }

                            v
                        });

                        self.editing_item.birth_date = value.and_then(|v| NaiveDate::from_str(&v).ok()).map(|v| v.to_string());
                    }
                    ChangingType::ThumbPath => unimplemented!(),
                }

            }

            Msg::RetrievePosters(value) => {
                self.cached_posters = Some(value);
            }

            Msg::RetrieveMediaView(value) => {
                self.media = Some(*value);
            }
        }

        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let media = match self.media.as_ref() {
            Some(v) => Some(crate::continue_or_html_err!(v)),
            None => None,
        };

        if let Some(GetPersonResponse { person, other_names }) = media {
            let editing = &self.editing_item;

            html! {
                <div class="outer-view-container">
                    <div class="sidebar-container">
                    {
                        if self.is_editing() {
                            html! {
                                <>
                                    <div class="sidebar-item">
                                        <button class="button" onclick={ctx.link().callback(|_| Msg::ToggleEdit)}>{ "Stop Editing" }</button>
                                    </div>
                                    <div class="sidebar-item">
                                        <button class="button proceed" onclick={ctx.link().callback(|_| Msg::SaveEdits)}>
                                            { "Save" }
                                        </button>
                                    </div>
                                </>
                            }
                        } else {
                            html! {
                                <LoginBarrier>
                                    <div class="sidebar-item">
                                        <button class="button" onclick={ctx.link().callback(|_| Msg::ToggleEdit)}>{ "Start Editing" }</button>
                                    </div>
                                </LoginBarrier>
                            }
                        }
                    }
                    </div>

                    <div class="view-container item-view-container">
                        <div class="info-container">
                            <div class="poster large">
                                <img src={ person.get_thumb_url() } />
                            </div>

                            <div class="metadata-container">
                                <div class="metadata">
                                    { // Book Display Info
                                        if self.is_editing() {
                                            html! {
                                                <>
                                                    <h5>{ "Book Display Info" }</h5>

                                                    <span class="sub-title">{ "Name" }</span>
                                                    <input class="title" type="text"
                                                        onchange={ Self::on_change_input(ctx.link(), ChangingType::Name) }
                                                        value={ editing.name.clone().unwrap_or_else(|| person.name.clone()) }
                                                    />

                                                    <span class="sub-title">{ "Description" }</span>
                                                    <textarea
                                                        rows="9"
                                                        cols="30"
                                                        class="description"
                                                        onchange={ Self::on_change_textarea(ctx.link(), ChangingType::Description) }
                                                        value={ editing.description.clone().or_else(|| person.description.clone()).unwrap_or_default() }
                                                    />
                                                </>
                                            }
                                        } else {
                                            html! {
                                                <>
                                                    <h3 class="title">{ person.name.clone() }</h3>
                                                    <p class="description">{ person.description.clone().unwrap_or_default() }</p>
                                                </>
                                            }
                                        }
                                    }
                                </div>

                                { // Book Info
                                    if self.is_editing() {
                                        html! {
                                            <div class="metadata">
                                                <h5>{ "Book Info" }</h5>

                                                <span class="sub-title">{ "Birth Date" }</span>
                                                <input class="title" type="text"
                                                    placeholder="YYYY-MM-DD"
                                                    onfocusout={ ctx.link().callback(move |e: FocusEvent| {
                                                        Msg::UpdateEditing(ChangingType::BirthDate, e.target().unwrap().dyn_into::<HtmlInputElement>().unwrap().value())
                                                    }) }
                                                    value={ editing.birth_date.clone().or_else(|| person.birth_date.map(|v| v.to_string())).unwrap_or_default() }
                                                />
                                            </div>
                                        }
                                    } else {
                                        html! {}
                                    }
                                }

                                { // Sources
                                    if self.is_editing() {
                                        html! {
                                            <div class="metadata">
                                                <h5>{ "Sources" }</h5>

                                                <span class="sub-title">{ "Good Reads URL" }</span>
                                                <input class="title" type="text" />

                                                <span class="sub-title">{ "Open Library URL" }</span>
                                                <input class="title" type="text" />

                                                <span class="sub-title">{ "Google Books URL" }</span>
                                                <input class="title" type="text" />

                                                <h5>{ "Tags" }</h5>
                                            </div>
                                        }
                                    } else {
                                        html! {}
                                    }
                                }
                            </div>
                        </div>

                        {
                            if self.is_editing() {
                                html! {}
                            } else {
                                html! {
                                    <>
                                        <h4>{ "Other Names" }</h4>
                                        <div class="label-group">
                                        {
                                            for other_names.iter()
                                                .map(|name| html! {
                                                    <div class="label">{ name.clone() }</div>
                                                })
                                        }
                                        </div>
                                    </>
                                }
                            }
                        }

                        { // Posters
                            if self.is_editing() {
                                if let Some(resp) = self.cached_posters.as_ref() {
                                    let person_id = person.id;

                                    match resp.as_ok() {
                                        Ok(resp) => html! {
                                            <section>
                                                <h2>{ "Posters" }</h2>
                                                <div class="posters-container">
                                                    <UploadModule
                                                        class="poster new-container"
                                                        title="Add Poster"
                                                        upload_url={ format!("/api/v1/posters/{}/upload", ImageIdType::new_person(ctx.props().id)) }
                                                        on_upload={ctx.link().callback(|_| Msg::UpdatedPoster)}
                                                    >
                                                        <span class="material-icons">{ "add" }</span>
                                                    </UploadModule>

                                                    {
                                                        for resp.items.iter().map(move |poster| {
                                                            let url_or_id = poster.id.map(Either::Right).unwrap_or_else(|| Either::Left(poster.path.clone()));
                                                            let is_selected = poster.selected;

                                                            html! {
                                                                <div
                                                                    class={ classes!("poster", { if is_selected { "selected" } else { "" } }) }
                                                                    onclick={ctx.link().callback_future(move |_| {
                                                                        let url_or_id = url_or_id.clone();

                                                                        async move {
                                                                            if is_selected {
                                                                                Msg::Ignore
                                                                            } else {
                                                                                request::change_poster_for_meta(ImageIdType::new_person(person_id), url_or_id).await;

                                                                                Msg::UpdatedPoster
                                                                            }
                                                                        }
                                                                    })}
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
                            } else {
                                html! {}
                            }
                        }

                        <section>
                            <h2>{ "Books" }</h2>
                            <div class="books-container">
                                <div class="book-list normal horizontal">
                                    // <div class="book-list-item new-container" title="Add Book">
                                    //     <span class="material-icons">{ "add" }</span>
                                    // </div>
                                    {
                                        if let Some(resp) = self.cached_books.as_ref() {
                                            match resp.as_ok() {
                                                Ok(resp) => html! {{
                                                    for resp.items.iter().map(|item| {
                                                        html! {
                                                            <MediaItem
                                                                is_editing=false
                                                                item={item.clone()}
                                                            />
                                                        }
                                                    })
                                                }},

                                                Err(e) => html! {
                                                    <h2>{ e }</h2>
                                                }
                                            }
                                        } else {
                                            html! {}
                                        }
                                    }
                                </div>
                            </div>
                        </section>
                    </div>
                </div>
            }
        } else {
            html! {
                <h1>{ "Loading..." }</h1>
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

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            let person_id = ctx.props().id;

            ctx.link().send_future(async move {
                Msg::RetrieveMediaView(Box::new(request::get_person(person_id).await))
            });

            if let Some(member) = get_member_self() {
                if member.localsettings.get_page_view_default().is_editing() {
                    self.is_editing = true;
                }
            }
        }
    }
}

impl AuthorView {
    fn is_editing(&self) -> bool {
        self.is_editing
    }

    fn on_change_input(scope: &Scope<Self>, updating: ChangingType) -> Callback<Event> {
        scope.callback(move |e: Event| {
            Msg::UpdateEditing(updating, e.target().unwrap().dyn_into::<HtmlInputElement>().unwrap().value())
        })
    }

    fn on_change_textarea(scope: &Scope<Self>, updating: ChangingType) -> Callback<Event> {
        scope.callback(move |e: Event| {
            Msg::UpdateEditing(updating, e.target().unwrap().dyn_into::<HtmlTextAreaElement>().unwrap().value())
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

    /// A Callback which calls "prevent_default"
    fn on_click_prevdef(scope: &Scope<Self>, msg: Msg) -> Callback<MouseEvent> {
        scope.callback(move |e: MouseEvent| {
            e.prevent_default();
            msg.clone()
        })
    }
}


#[derive(Debug, Clone)]
pub struct CachedTag {
    type_of: TagType,
    id: usize,
    name: String,
}



#[derive(Clone, Copy)]
pub enum ChangingType {
    Name,
    Description,
    BirthDate,
    ThumbPath,
}




#[derive(Clone)]
pub enum DisplayOverlay {
    Info {
        meta_id: usize
    },

    Edit(Box<api::MediaViewResponse>),

    More {
        meta_id: usize,
        mouse_pos: (i32, i32)
    },
}

impl PartialEq for DisplayOverlay {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Info { meta_id: l_id }, Self::Info { meta_id: r_id }) => l_id == r_id,
            (Self::More { meta_id: l_id, .. }, Self::More { meta_id: r_id, .. }) => l_id == r_id,

            _ => false
        }
    }
}