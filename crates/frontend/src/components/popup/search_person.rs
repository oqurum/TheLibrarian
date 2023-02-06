use common::{
    api::{QueryListResponse, WrappingResponse},
    component::popup::{Popup, PopupType},
    PersonId, Source,
};
use common_local::{
    api::{ExternalSearchResponse, SearchItem},
    item::edit::PersonEdit,
    Person, SearchType,
};
use gloo_utils::document;
use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;
use yew::prelude::*;

use crate::{
    request,
    util::{self, LoadingItem},
};

use super::SearchBy;

#[derive(Properties, PartialEq)]
pub struct Property {
    #[prop_or_default]
    pub classes: Classes,

    pub type_of: SearchBy,

    pub on_close: Callback<()>,
    pub on_select: Callback<SearchSelectedValue>,

    #[prop_or_default]
    pub input_value: String,

    #[prop_or_default]
    pub search_on_init: bool,
}

pub enum Msg {
    PersonLocalSearchResponse(String, WrappingResponse<QueryListResponse<Person>>),
    PersonExternalSearchResponse(String, WrappingResponse<ExternalSearchResponse>),

    SearchFor(String),

    OnSelectItem(SearchSelectedValue),
}

pub struct PopupSearchPerson {
    cached_loc_search: Option<LoadingItem<WrappingResponse<QueryListResponse<Person>>>>,
    cached_ext_search: Option<LoadingItem<WrappingResponse<ExternalSearchResponse>>>,
    input_value: String,
}

impl Component for PopupSearchPerson {
    type Message = Msg;
    type Properties = Property;

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            cached_loc_search: None,
            cached_ext_search: None,
            input_value: ctx.props().input_value.clone(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::SearchFor(search) => {
                self.cached_loc_search = Some(LoadingItem::Loading);

                if ctx.props().type_of == SearchBy::External {
                    ctx.link().send_future(async move {
                        let resp = request::external_search_for(&search, SearchType::Person).await;

                        Msg::PersonExternalSearchResponse(search, resp)
                    });
                } else {
                    ctx.link().send_future(async move {
                        let resp = request::get_people(Some(&search), None, None).await;

                        Msg::PersonLocalSearchResponse(search, resp)
                    });
                }
            }

            Msg::PersonLocalSearchResponse(search, resp) => {
                self.cached_loc_search = Some(LoadingItem::Loaded(resp));
                self.input_value = search;
            }

            Msg::PersonExternalSearchResponse(search, resp) => {
                self.cached_ext_search = Some(LoadingItem::Loaded(resp));
                self.input_value = search;
            }

            Msg::OnSelectItem(value) => {
                ctx.props().on_select.emit(value);
            }
        }

        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        self.render_main(ctx)
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render && ctx.props().search_on_init {
            ctx.link()
                .send_message(Msg::SearchFor(ctx.props().input_value.clone()));
        }
    }
}

impl PopupSearchPerson {
    fn render_main(&self, ctx: &Context<Self>) -> Html {
        let input_id = "external-book-search-input";

        html! {
            <Popup
                type_of={ PopupType::FullOverlay }
                on_close={ ctx.props().on_close.clone() }
                classes={ classes!("external-book-search-popup") }
            >
                <div class="modal-header">
                    <h1 class="modal-title">{"Book Search"}</h1>
                </div>

                <div class="modal-body">
                    <div class="container">
                        <form class="row">
                            <input class="form-control" id={input_id} name="book_search" placeholder="Search For Title" value={ self.input_value.clone() } />
                            <button class="btn btn-success" onclick={
                                ctx.link().callback(move |e: MouseEvent| {
                                    e.prevent_default();

                                    let input = document().get_element_by_id(input_id).unwrap().unchecked_into::<HtmlInputElement>();

                                    Msg::SearchFor(input.value())
                                })
                            }>{ "Search" }</button>
                        </form>

                        <hr />

                        <div class="external-book-search-container">
                            {
                                match ctx.props().type_of {
                                    SearchBy::External => {
                                        if let Some(loading) = self.cached_ext_search.as_ref() {
                                            match loading {
                                                LoadingItem::Loaded(wrapper) => {
                                                    match wrapper.as_ok() {
                                                        Ok(search) => html! {
                                                            <>
                                                                <div class="book-search-items">
                                                                {
                                                                    for search.items.values()
                                                                        .flat_map(|values| values.iter())
                                                                        .map(|item| Self::render_ext_search_container(item, ctx))
                                                                }
                                                                </div>
                                                            </>
                                                        },

                                                        Err(e) => html! {
                                                            <h2>{ e }</h2>
                                                        }
                                                    }
                                                },

                                                LoadingItem::Loading => html! {
                                                    <h2>{ "Loading..." }</h2>
                                                }
                                            }
                                        } else {
                                            html! {}
                                        }
                                    }

                                    SearchBy::Local => {
                                        if let Some(loading) = self.cached_loc_search.as_ref() {
                                            match loading {
                                                LoadingItem::Loaded(wrapper) => {
                                                    match wrapper.as_ok() {
                                                        Ok(search) => html! {
                                                            <>
                                                                <div class="book-search-items">
                                                                {
                                                                    for search.items.iter()
                                                                        .map(|item| Self::render_loc_search_container(item, ctx))
                                                                }
                                                                </div>
                                                            </>
                                                        },

                                                        Err(e) => html! {
                                                            <h2>{ e }</h2>
                                                        }
                                                    }
                                                },

                                                LoadingItem::Loading => html! {
                                                    <h2>{ "Loading..." }</h2>
                                                }
                                            }
                                        } else {
                                            html! {}
                                        }
                                    }
                                }
                            }
                        </div>
                    </div>
                </div>

            </Popup>
        }
    }

    fn render_ext_search_container(item: &SearchItem, ctx: &Context<Self>) -> Html {
        let item = item.as_person();

        let source = item.source.clone();

        html! {
            <div
                class="book-search-item"
                onclick={ ctx.link().callback(move |_| Msg::OnSelectItem(SearchSelectedValue::Source(source.clone()))) }
            >
                <img src={ item.cover_image.clone().unwrap_or_default() } />
                <div class="book-info">
                    <h4 class="book-name">{ item.name.clone() }</h4>
                    <p class="book-author">{ item.description.clone()
                            .map(|mut v| { util::truncate_on_indices(&mut v, 300); v })
                            .unwrap_or_default() }
                    </p>
                </div>
            </div>
        }
    }

    fn render_loc_search_container(item: &Person, ctx: &Context<Self>) -> Html {
        let book_id = item.id;

        html! {
            <div
                class="book-search-item"
                onclick={ ctx.link().callback(move |_| Msg::OnSelectItem(SearchSelectedValue::PersonId(book_id))) }
            >
                <img src={ item.get_thumb_url() } />
                <div class="book-info">
                    <h4 class="book-name">{ item.name.clone() }</h4>
                    <span class="book-author">{ item.description.clone().unwrap_or_default() }</span>
                </div>
            </div>
        }
    }
}

#[derive(PartialEq, Eq)]
pub enum SearchSelectedValue {
    Source(Source),
    PersonEdit(Box<PersonEdit>),
    PersonId(PersonId),
}
