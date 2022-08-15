// TODO: Temporary. Some of the dead_code in here will be used.
#![allow(dead_code)]

use common::{component::upload::UploadModule, Either, PersonId, ImageIdType, api::WrappingResponse};
use common_local::{api::{self, GetPostersResponse, GetPersonResponse}, TagType};
use wasm_bindgen::JsCast;
use web_sys::{HtmlInputElement, HtmlTextAreaElement};
use yew::{prelude::*, html::Scope};

use crate::{components::LoginBarrier, request, pages::home::MediaItem};



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

#[derive(Properties, PartialEq)]
pub struct Property {
    pub id: PersonId,
}

pub struct AuthorView {
    media: Option<WrappingResponse<GetPersonResponse>>,
    cached_posters: Option<WrappingResponse<GetPostersResponse>>,
    cached_books: Option<WrappingResponse<api::GetBookListResponse>>,

    media_popup: Option<DisplayOverlay>,

    /// If we're currently editing. This'll be set.
    editing_item: Option<GetPersonResponse>,
}

impl Component for AuthorView {
    type Message = Msg;
    type Properties = Property;

    fn create(ctx: &Context<Self>) -> Self {
        let person_id = ctx.props().id;

        ctx.link()
        .send_future(async move {
            let resp = request::get_books(
                None,
                None,
                None,
                Some(person_id),
            ).await;

            Msg::BooksListResults(resp)
        });

        Self {
            media: None,
            cached_posters: None,
            cached_books: None,

            media_popup: None,
            editing_item: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Ignore => return false,

            Msg::BooksListResults(resp) => {
                self.cached_books = Some(resp);
            }

            Msg::UpdatedPoster => if let Some(book) = self.media.as_ref().and_then(|v| v.resp.as_ref()) {
                let person_id = ImageIdType::new_person(book.person.id);

                ctx.link()
                .send_future(async move {
                    Msg::RetrievePosters(request::get_posters_for_meta(person_id).await)
                });

                return false;
            }

            // Edits
            Msg::ToggleEdit => if let Some(book) = self.media.as_ref().and_then(|v| v.resp.as_ref()) {
                if self.editing_item.is_none() {
                    self.editing_item = Some(book.clone());

                    if self.cached_posters.is_none() {
                        let person_id = ImageIdType::new_person(book.person.id);

                        ctx.link()
                        .send_future(async move {
                            Msg::RetrievePosters(request::get_posters_for_meta(person_id).await)
                        });
                    }
                } else {
                    self.editing_item = None;
                }
            }

            Msg::SaveEdits => {
                self.media = self.editing_item.clone().map(WrappingResponse::okay);

                // let metadata = self.media.as_ref().and_then(|v| v.resp.as_ref()).unwrap().person.clone();
                // let meta_id = metadata.id;

                // ctx.link()
                // .send_future(async move {
                //     request::update_book(meta_id, &api::UpdateBookBody {
                //         metadata: Some(metadata),
                //         people: None,
                //     }).await;

                //     Msg::Ignore
                // });
            }

            Msg::UpdateEditing(type_of, value) => {
                let mut updating = self.editing_item.as_mut().unwrap();

                let value = Some(value).filter(|v| !v.is_empty());

                match type_of {
                    ChangingType::Name => updating.person.name = value.unwrap_or_default(),
                    ChangingType::Description => updating.person.description = value,
                    ChangingType::BirthDate => updating.person.birth_date = value,
                    ChangingType::ThumbPath => todo!(),
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

        let resp = self.editing_item.as_ref().or(media);

        if let Some(GetPersonResponse { person }) = resp {
            html! {
                <div class="outer-view-container">
                    <div class="sidebar-container">
                    {
                        if self.is_editing() {
                            html! {
                                <>
                                    <div class="sidebar-item">
                                        <button class="button" onclick={ctx.link().callback(|_| Msg::ToggleEdit)}>{"Stop Editing"}</button>
                                    </div>
                                    <div class="sidebar-item">
                                        <button class="button proceed" onclick={ctx.link().callback(|_| Msg::SaveEdits)}>
                                            {"Save"}
                                        </button>
                                    </div>
                                </>
                            }
                        } else {
                            html! {
                                <LoginBarrier>
                                    <div class="sidebar-item">
                                        <button class="button" onclick={ctx.link().callback(|_| Msg::ToggleEdit)}>{"Start Editing"}</button>
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

                                                    <span class="sub-title">{"Name"}</span>
                                                    <input class="title" type="text"
                                                        onchange={Self::on_change_input(ctx.link(), ChangingType::Name)}
                                                        value={ person.name.clone() }
                                                    />

                                                    <span class="sub-title">{"Description"}</span>
                                                    <textarea
                                                        rows="9"
                                                        cols="30"
                                                        class="description"
                                                        onchange={Self::on_change_textarea(ctx.link(), ChangingType::Description)}
                                                        value={ person.description.clone().unwrap_or_default() }
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

                                                <span class="sub-title">{"Birth Date"}</span>
                                                <input class="title" type="text"
                                                    placeholder="YYYY"
                                                    onchange={Self::on_change_input(ctx.link(), ChangingType::BirthDate)}
                                                    value={ person.birth_date.clone().unwrap_or_default() }
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
                                                        class="add-poster"
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
                                    // <div class="add-book" title="Add Book">
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

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            let person_id = ctx.props().id;

            ctx.link().send_future(async move {
                Msg::RetrieveMediaView(Box::new(request::get_person(person_id).await))
            });
        }
    }
}

impl AuthorView {
    fn is_editing(&self) -> bool {
        self.editing_item.is_some()
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