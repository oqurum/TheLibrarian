use common::{
    api::WrappingResponse,
    component::popup::{Popup, PopupClose, PopupType},
};
use common_local::{
    api::{self, NewCollectionBody},
    Collection,
};
use wasm_bindgen::{prelude::Closure, JsCast, UnwrapThrowExt};
use web_sys::{HtmlElement, HtmlInputElement, HtmlSelectElement, HtmlTextAreaElement};
use yew::{html::Scope, prelude::*};
use yew_router::prelude::Link;

use crate::{components::LoginBarrier, request, Route};

#[derive(Properties, PartialEq, Eq)]
pub struct Property {}

#[derive(Clone)]
pub enum Msg {
    // Requests
    RequestCollections,

    // Results
    CollectionListResults(WrappingResponse<api::GetCollectionListResponse>),
    CreateResult(WrappingResponse<api::GetCollectionResponse>),

    // Events
    OnScroll(i32),
    InitEventListener,

    CreateNewCollection,
    ClosePopup,
}

pub struct ListCollectionsPage {
    on_scroll_fn: Option<Closure<dyn FnMut()>>,

    media_items: Option<WrappingResponse<Vec<Collection>>>,
    total_media_count: usize,

    is_fetching: bool,

    collection_list_ref: NodeRef,

    is_creating_collection: bool,
}

impl Component for ListCollectionsPage {
    type Message = Msg;
    type Properties = Property;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            on_scroll_fn: None,
            media_items: None,
            total_media_count: 0,
            is_fetching: false,
            collection_list_ref: NodeRef::default(),
            is_creating_collection: false,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::InitEventListener => {
                let lib_list_ref = self.collection_list_ref.clone();
                let link = ctx.link().clone();

                let func = Closure::wrap(Box::new(move || {
                    let lib_list = lib_list_ref.cast::<HtmlElement>().unwrap();

                    link.send_message(Msg::OnScroll(
                        lib_list.client_height() + lib_list.scroll_top(),
                    ));
                }) as Box<dyn FnMut()>);

                let _ = self
                    .collection_list_ref
                    .cast::<HtmlElement>()
                    .unwrap()
                    .add_event_listener_with_callback("scroll", func.as_ref().unchecked_ref());

                self.on_scroll_fn = Some(func);
            }

            Msg::RequestCollections => {
                if self.is_fetching {
                    return false;
                }

                self.is_fetching = true;

                let offset = Some(
                    self.media_items
                        .as_ref()
                        .and_then(|v| v.as_ok().ok())
                        .map(|v| v.len())
                        .unwrap_or_default(),
                )
                .filter(|v| *v != 0);

                ctx.link().send_future(async move {
                    Msg::CollectionListResults(
                        request::get_collection_list(None, offset, None).await,
                    )
                });
            }

            Msg::CollectionListResults(resp) => {
                self.is_fetching = false;

                match resp.ok() {
                    Ok(mut resp) => {
                        self.total_media_count = resp.total;

                        // TODO: Replace match with as_mut_ok()
                        if let Some(items) = self.media_items.as_mut().and_then(|v| match v {
                            WrappingResponse::Resp(v) => Some(v),
                            _ => None,
                        }) {
                            items.append(&mut resp.items);
                        } else {
                            self.media_items = Some(WrappingResponse::okay(resp.items));
                        }
                    }

                    Err(e) => {
                        log::error!("{e}")
                    }
                }
            }

            Msg::CreateResult(_resp) => {
                // TODO
            }

            Msg::OnScroll(scroll_y) => {
                let scroll_height = self
                    .collection_list_ref
                    .cast::<HtmlElement>()
                    .unwrap()
                    .scroll_height();

                if scroll_height - scroll_y < 600 && self.can_req_more() {
                    ctx.link().send_message(Msg::RequestCollections);
                }
            }

            Msg::CreateNewCollection => {
                self.is_creating_collection = true;
            }

            Msg::ClosePopup => self.is_creating_collection = false,
        }

        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let items = match self.media_items.as_ref() {
            Some(v) => Some(crate::continue_or_html_err!(v)),
            None => None,
        };

        if let Some(items) = items {
            html! {
                <>
                    <div class="outer-view-container h-100 px-0">
                        <div class="sidebar-container d-none d-md-flex flex-column flex-shrink-0 p-2 text-bg-dark">
                            <LoginBarrier>
                                <div class="sidebar-item">
                                    <button class="btn btn-secondary" onclick={ ctx.link().callback(|_| Msg::CreateNewCollection) }>
                                        { "New Collection" }
                                    </button>
                                </div>
                            </LoginBarrier>
                        </div>
                        <div class="view-container">
                            <div class="collection-list row-size-1 row-p-size-2 row-t-size-4" ref={ self.collection_list_ref.clone() }>
                                { for items.iter().map(|item| self.render_item(item)) }
                            </div>
                        </div>
                    </div>

                    {
                        if self.is_creating_collection {
                            html! { <CreateCollectionPopup scope={ ctx.link().clone() } /> }
                        } else {
                            html! {}
                        }
                    }
                </>
            }
        } else {
            html! {
                <h1>{ "Loading..." }</h1>
            }
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if self.on_scroll_fn.is_none() && self.collection_list_ref.get().is_some() {
            ctx.link().send_message(Msg::InitEventListener);
        } else if first_render {
            ctx.link().send_message(Msg::RequestCollections);
        }
    }

    fn destroy(&mut self, _ctx: &Context<Self>) {
        if let Some(f) = self.on_scroll_fn.take() {
            let _ = self
                .collection_list_ref
                .cast::<HtmlElement>()
                .unwrap()
                .remove_event_listener_with_callback("scroll", f.as_ref().unchecked_ref());
        }
    }
}

impl ListCollectionsPage {
    fn render_item(&self, item: &Collection) -> Html {
        html! {
            <Link<Route> to={ Route::Collection { path: format!("{}-{}", item.id, clean_title(&item.name)) } } classes="collection-list-item link-light">
                <h4>{ item.name.clone() }</h4>
                <p>{ item.description.clone().unwrap_or_default() }</p>
            </Link<Route>>
        }
    }

    pub fn can_req_more(&self) -> bool {
        let count = self
            .media_items
            .as_ref()
            .and_then(|v| v.as_ok().ok())
            .map(|v| v.len())
            .unwrap_or_default();

        count != 0 && count != self.total_media_count
    }
}

fn clean_title(value: &str) -> String {
    let mut value = value.trim().to_lowercase().replace(' ', "-");

    value.shrink_to(20);

    REPLACE_CHARS.into_iter().for_each(|v| {
        value = value.replace(v, "-");
    });

    value
}

static REPLACE_CHARS: [char; 14] = [
    '!', '@', '#', '$', '^', '&', '*', '(', ')', '?', '.', '|', '\\', '/',
];

#[derive(Properties)]
struct CreateCollectionPopupProps {
    scope: Scope<ListCollectionsPage>,
}

impl PartialEq for CreateCollectionPopupProps {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

#[function_component(CreateCollectionPopup)]
fn create_collection_popup(props: &CreateCollectionPopupProps) -> Html {
    let input_ref = use_node_ref();
    let textarea_ref = use_node_ref();
    let select_ref = use_node_ref();

    html! {
        <Popup
            type_of={ PopupType::FullOverlay }
            on_close={ props.scope.callback(|_| Msg::ClosePopup) }
        >
            <div class="modal-header">
                <h1 class="modal-title">{ "Create Collection" }</h1>
            </div>
            <div class="modal-body">
                <form class="form-container shrink-width-to-content">
                    <label for="new-collection-name">{ "Collection Name" }</label>
                    <input class="form-control" id="new-collection-name" ref={ input_ref.clone() } name="new_collection_name" placeholder="Collection Name" />

                    <label for="new-collection-desc">{ "Collection Description" }</label>
                    <textarea class="form-control" id="new-collection-desc" ref={ textarea_ref.clone() } name="new_collection_desc"></textarea>

                    <label for="new-collection-type">{ "Collection Type" }</label>
                    <select class="form-select" id="new-collection-type" ref={ select_ref.clone() }>
                        <option selected=true>{ "List" }</option>
                        <option>{ "Series" }</option>
                    </select>

                    <button class="btn btn-success" onclick={
                        props.scope.callback_future(move |e: MouseEvent| {
                            let input_ref = input_ref.clone();
                            let textarea_ref = textarea_ref.clone();
                            let select_ref = select_ref.clone();

                            async move {
                                e.prevent_default();

                                let resp = request::create_collection(NewCollectionBody {
                                    name: input_ref.cast::<HtmlInputElement>().unwrap_throw().value(),
                                    description: Some(textarea_ref.cast::<HtmlTextAreaElement>().unwrap_throw().value()).map(|v| v.trim().to_string()).filter(|v| !v.is_empty()),
                                    type_of: (select_ref.cast::<HtmlSelectElement>().unwrap_throw().selected_index() as u8).into(),
                                }).await;

                                Msg::CreateResult(resp)
                            }
                        })
                    }>{ "Create" }</button>

                    <PopupClose>
                        <button class="btn btn-danger">{ "Close" }</button>
                    </PopupClose>
                </form>
            </div>
        </Popup>
    }
}
