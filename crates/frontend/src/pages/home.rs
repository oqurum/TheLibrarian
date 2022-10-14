use std::{rc::Rc, sync::Mutex};

use common::{
    api::WrappingResponse,
    component::{InfiniteScroll, InfiniteScrollEvent},
    BookId,
};
use common_local::{
    api::{self, BookListQuery, NewBookBody, QueryType},
    DisplayItem, SearchType,
};
use gloo_utils::window;
use js_sys::decode_uri_component;
use wasm_bindgen::{JsValue, UnwrapThrowExt};
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_router::prelude::Link;

use crate::{
    components::{
        popup::{search::SearchSelectedValue, SearchBy},
        LoginBarrier, MassSelectBar, PopupSearch,
    },
    get_member_self, request, Route,
};

#[derive(Clone)]
pub enum Msg {
    // Requests
    RequestMediaItems,

    // Results
    MediaListResults(WrappingResponse<api::GetBookListResponse>),

    // Events
    OnScrollEvent(InfiniteScrollEvent),
    ClosePopup,
    OpenPopup(DisplayOverlay),

    AddOrRemoveItemFromEditing(BookId, bool),
    DeselectAllEditing,

    Reload,

    Ignore,
}

pub struct HomePage {
    media_items: Option<Vec<DisplayItem>>,
    total_media_count: usize,

    is_fetching_media_items: bool,

    media_popup: Option<DisplayOverlay>,

    library_list_ref: NodeRef,

    editing_items: Rc<Mutex<Vec<BookId>>>,
}

impl Component for HomePage {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            media_items: None,
            total_media_count: 0,

            is_fetching_media_items: false,

            media_popup: None,

            library_list_ref: NodeRef::default(),

            editing_items: Rc::new(Mutex::new(Vec::new())),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::OpenPopup(disp) => {
                self.media_popup = Some(disp);
            }

            Msg::ClosePopup => {
                self.media_popup = None;
            }

            Msg::DeselectAllEditing => {
                self.editing_items.lock().unwrap().clear();
            }

            Msg::AddOrRemoveItemFromEditing(id, value) => {
                let mut items = self.editing_items.lock().unwrap();

                if value {
                    if !items.iter().any(|v| id == *v) {
                        items.push(id);
                    }
                } else if let Some(index) = items.iter().position(|v| id == *v) {
                    items.swap_remove(index);
                }
            }

            Msg::RequestMediaItems => {
                if self.is_fetching_media_items {
                    return false;
                }

                self.is_fetching_media_items = true;

                let mut query = get_query().unwrap_or_default();
                query.offset = Some(
                    self.media_items
                        .as_ref()
                        .map(|v| v.len())
                        .unwrap_or_default(),
                )
                .filter(|v| *v != 0);

                ctx.link().send_future(async move {
                    Msg::MediaListResults(request::get_books(query).await)
                });
            }

            Msg::MediaListResults(resp) => {
                let mut resp = resp.ok().unwrap_throw();

                self.is_fetching_media_items = false;
                self.total_media_count = resp.count;

                if let Some(items) = self.media_items.as_mut() {
                    items.append(&mut resp.items);
                } else {
                    self.media_items = Some(resp.items);
                }
            }

            Msg::OnScrollEvent(event) => {
                if event.scroll_height - event.scroll_pos < 600 && self.can_req_more() {
                    ctx.link().send_message(Msg::RequestMediaItems);
                }

                return false;
            }

            Msg::Reload => {
                self.total_media_count = 0;
                self.media_items = None;

                self.media_popup = None;
                self.editing_items.lock().unwrap().clear();

                return self.update(ctx, Msg::RequestMediaItems);
            }

            Msg::Ignore => return false,
        }

        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let content = if let Some(items) = self.media_items.as_deref() {
            html! {
                <div class="view-container">
                    // Filter Bar
                    <div class="filter-bar">
                        <div class="bar-container">
                            <div class="left-content">
                                <button class="slim"
                                    onclick={ ctx.link().callback(|_| {
                                        let loc = window().location();

                                        let history = window().history().unwrap_throw();

                                        history.push_state_with_url(
                                            &JsValue::NULL,
                                            "",
                                            Some(&loc.pathname().unwrap())
                                        ).expect_throw("failed to push state");

                                        Msg::Reload
                                    }) }
                                >{ "Unset Filter" }</button>
                                <button class="slim"
                                    onclick={ ctx.link().callback(move |_| {
                                        let mut query = get_query().unwrap_or_default();
                                        query.search = Some(QueryType::HasPerson(false));

                                        let loc = window().location();

                                        let history = window().history().unwrap_throw();

                                        history.push_state_with_url(
                                            &JsValue::NULL,
                                            "",
                                            Some(&format!(
                                                "{}?{}",
                                                loc.pathname().unwrap(),
                                                serde_qs::to_string(&query).unwrap_throw()
                                            ))
                                        ).expect_throw("failed to push state");

                                        Msg::Reload
                                    }) }
                                >{ "Filter Missing Person" }</button>
                            </div>
                            <div class="right-content">
                                <span>{ "Total: " } { self.total_media_count }</span>
                            </div>
                        </div>
                    </div>

                    // Book List
                    <InfiniteScroll
                        class="book-list normal"
                        ref={ self.library_list_ref.clone() }
                        event={ ctx.link().callback(Msg::OnScrollEvent) }
                    >
                        {
                            for items.iter().map(|item| {
                                let is_editing = self.editing_items.lock().unwrap().contains(&item.id);

                                html! {
                                    <MediaItem
                                        {is_editing}

                                        item={item.clone()}
                                        callback={ctx.link().callback(|v| v)}
                                    />
                                }
                            })
                        }

                        {
                            if let Some(DisplayOverlay::SearchForBook { input_value }) = self.media_popup.as_ref() {
                                let input_value = if let Some(v) = input_value {
                                    v.trim().to_string()
                                } else {
                                    String::new()
                                };

                                html! {
                                    <PopupSearch
                                        {input_value}
                                        type_of={ SearchBy::External }
                                        comparable=true
                                        search_for={ SearchType::Book }
                                        on_close={ ctx.link().callback(|_| Msg::ClosePopup) }
                                        on_select={ ctx.link().callback_future(|value: SearchSelectedValue| async {
                                            request::update_one_or_more_books(NewBookBody::Value(Box::new(value.into_external()))).await;

                                            Msg::Ignore
                                        }) }
                                    />
                                }
                            } else {
                                html! {}
                            }
                        }
                    </InfiniteScroll>

                    <MassSelectBar
                        on_deselect_all={ctx.link().callback(|_| Msg::DeselectAllEditing)}
                        editing_container={self.library_list_ref.clone()}
                        editing_items={self.editing_items.clone()}
                    />
                </div>
            }
        } else {
            html! {
                <h1>{ "Loading..." }</h1>
            }
        };

        html! {
            <div class="outer-view-container">
                <div class="sidebar-container">
                    <LoginBarrier>
                        <div class="sidebar-item">
                            <button class="button" onclick={ctx.link().callback(|_| Msg::OpenPopup(DisplayOverlay::SearchForBook { input_value: None }))}>{"Add New Book"}</button>
                        </div>
                    </LoginBarrier>
                </div>
                { content }
            </div>
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            ctx.link().send_message(Msg::RequestMediaItems);
        }
    }
}

impl HomePage {
    // fn render_placeholder_item() -> Html {
    //     html! {
    //         <div class="book-list-item placeholder">
    //             <div class="poster"></div>
    //             <div class="info">
    //                 <a class="author"></a>
    //                 <a class="title"></a>
    //             </div>
    //         </div>
    //     }
    // }

    pub fn can_req_more(&self) -> bool {
        let count = self
            .media_items
            .as_ref()
            .map(|v| v.len())
            .unwrap_or_default();

        count != 0 && count != self.total_media_count as usize
    }
}

// Media Item

#[derive(Properties)]
pub struct MediaItemProps {
    pub item: DisplayItem,
    pub callback: Option<Callback<Msg>>,
    pub is_editing: bool,
}

impl PartialEq for MediaItemProps {
    fn eq(&self, other: &Self) -> bool {
        self.item == other.item && self.is_editing == other.is_editing
    }
}

pub struct MediaItem;

impl Component for MediaItem {
    type Message = Msg;
    type Properties = MediaItemProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        if let Some(cb) = ctx.props().callback.as_ref() {
            cb.emit(msg);
        }

        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let editing_perms = get_member_self()
            .map(|v| v.permissions.has_editing_perms())
            .unwrap_or_default();

        let &MediaItemProps {
            is_editing,
            ref item,
            ..
        } = ctx.props();

        let meta_id = item.id;

        html! {
            <Link<Route> to={Route::ViewMeta { meta_id: item.id }} classes={ classes!("book-list-item") }>
                <div class="poster">
                    <div class="top-left">
                    {
                        if editing_perms {
                            html! {
                                <input
                                    checked={is_editing}
                                    type="checkbox"
                                    onclick={ctx.link().callback(move |e: MouseEvent| {
                                        e.prevent_default();
                                        e.stop_propagation();

                                        Msg::Ignore
                                    })}
                                    onmouseup={ctx.link().callback(move |e: MouseEvent| {
                                        let input = e.target_unchecked_into::<HtmlInputElement>();

                                        let value = !input.checked();

                                        input.set_checked(value);

                                        Msg::AddOrRemoveItemFromEditing(meta_id, value)
                                    })}
                                />
                            }
                        } else {
                            html! {}
                        }
                    }
                    </div>
                    <img src={ item.get_thumb_url() } />
                </div>
                <div class="info">
                    <div class="title" title={ item.title.clone() }>{ item.title.clone() }</div>
                    {
                        if let Some(author) = item.cached.author.as_ref() {
                            if let Some(author_id) = item.cached.author_id {
                                html! {
                                    <Link<Route> to={ Route::Person { id: author_id } } classes={ "author" }>
                                        { author.clone() }
                                    </Link<Route>>
                                }
                            } else {
                                html! {
                                    <div class="author" title={ author.clone() }>{ author.clone() }</div>
                                }
                            }
                        } else {
                            html! {}
                        }
                    }
                </div>
            </Link<Route>>
        }
    }
}

#[derive(Clone)]
pub enum DisplayOverlay {
    SearchForBook { input_value: Option<String> },
}

impl PartialEq for DisplayOverlay {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Self::SearchForBook {
                    input_value: l_val, ..
                },
                Self::SearchForBook {
                    input_value: r_val, ..
                },
            ) => l_val == r_val,
        }
    }
}

fn get_query() -> Option<BookListQuery> {
    let query = gloo_utils::window().location().search().ok()?;

    if query.is_empty() {
        None
    } else {
        serde_qs::from_str(
            &decode_uri_component(&query[1..])
                .unwrap_throw()
                .as_string()
                .unwrap_throw(),
        )
        .ok()
    }
}
