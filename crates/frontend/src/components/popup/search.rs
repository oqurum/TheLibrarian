use common::{component::popup::{compare::{Comparable, PopupComparison}, Popup, PopupType}, Either, Source, api::WrappingResponse, util::upper_case_first_char, BookId};
use common_local::{api::{SearchItem, self, SearchQuery}, SearchType, item::edit::BookEdit, DisplayItem};
use gloo_utils::document;
use wasm_bindgen::{JsCast, UnwrapThrowExt};
use web_sys::HtmlInputElement;
use yew::prelude::*;

use crate::{request, util::{self, LoadingItem}};

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
    pub search_for: SearchType,

    #[prop_or_default]
    pub search_on_init: bool,

    #[prop_or_default]
    pub comparable: bool,
}


pub enum Msg {
    BookLocalSearchResponse(String, WrappingResponse<api::GetBookListResponse>),
    BookExternalSearchResponse(String, WrappingResponse<api::ExternalSearchResponse>),
    BookItemResponse(Source, Box<WrappingResponse<api::ExternalSourceItemResponse>>),

    SearchFor(String),

    OnChangeTab(String),

    OnSelectItem(SearchSelectedValue),

    OnSubmitSingle,
}


pub struct PopupSearch {
    cached_ext_search: Option<LoadingItem<WrappingResponse<api::ExternalSearchResponse>>>,
    cached_loc_search: Option<LoadingItem<WrappingResponse<api::GetBookListResponse>>>,
    input_value: String,

    left_edit: Option<(BookEdit, Source)>,
    right_edit: Option<(BookEdit, Source)>,

    selected_tab: String,

    waiting_item_resp: bool,
}

impl Component for PopupSearch {
    type Message = Msg;
    type Properties = Property;

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            cached_ext_search: None,
            cached_loc_search: None,
            input_value: ctx.props().input_value.clone(),

            left_edit: None,
            right_edit: None,

            selected_tab: String::new(),

            waiting_item_resp: false,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::SearchFor(search) => {
                self.cached_ext_search = Some(LoadingItem::Loading);


                if ctx.props().type_of == SearchBy::External {
                    let search_for = ctx.props().search_for;

                    ctx.link()
                    .send_future(async move {
                        let resp = request::external_search_for(&search, search_for).await;

                        Msg::BookExternalSearchResponse(search, resp)
                    });
                } else {
                    ctx.link()
                    .send_future(async move {
                        let resp = request::get_books(
                            None,
                            None,
                            Some(SearchQuery { query: Some(search.clone()), source: None, order: None, }),
                            None
                        ).await;

                        Msg::BookLocalSearchResponse(search, resp)
                    });
                }
            }

            Msg::BookExternalSearchResponse(search, resp) => {
                if let Some(name) = resp.as_ok().ok().and_then(|v| v.items.keys().next()).cloned() {
                    self.selected_tab = name;
                }

                self.cached_ext_search = Some(LoadingItem::Loaded(resp));
                self.input_value = search;
            }

            Msg::BookLocalSearchResponse(search, resp) => {
                self.cached_loc_search = Some(LoadingItem::Loaded(resp));
                self.input_value = search;
            }

            Msg::BookItemResponse(source, resp) => {
                if let Some(item) = resp.ok().ok().and_then(|v| v.item) {
                    if self.left_edit.is_none() {
                        self.left_edit = Some((item.into(), source));
                    } else {
                        self.right_edit = Some((item.into(), source));
                    }
                }

                self.waiting_item_resp = false;
            }

            Msg::OnSelectItem(value) => {
                match value {
                    SearchSelectedValue::Source(source) => {
                        if !ctx.props().comparable {
                            ctx.props().on_select.emit(SearchSelectedValue::Source(source));
                            return false;
                        }

                        if self.waiting_item_resp {
                            return false;
                        }

                        self.waiting_item_resp = true;

                        // TODO: Only Request once we've selected both sources.
                        ctx.link().send_future(async move {
                            Msg::BookItemResponse(source.clone(), Box::new(request::get_external_source_item(source).await))
                        });
                    }

                    v => ctx.props().on_select.emit(v),
                }
            }

            Msg::OnSubmitSingle => {
                if let Some((_, source)) = self.left_edit.as_ref() {
                    ctx.props().on_select.emit(SearchSelectedValue::Source(source.clone()));
                }
            }

            Msg::OnChangeTab(name) => {
                self.selected_tab = name;
            }
        }

        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        if let Some(((left, _), (right, _))) = self.left_edit.clone().zip(self.right_edit.clone()) {
            self.render_compare(left, right, ctx)
        } else {
            self.render_main(ctx)
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render && ctx.props().search_on_init {
            ctx.link().send_message(Msg::SearchFor(ctx.props().input_value.clone()));
        }
    }
}

impl PopupSearch {
    fn render_main(&self, ctx: &Context<Self>) -> Html {
        let input_id = "external-book-search-input";

        html! {
            <Popup
                type_of={ PopupType::FullOverlay }
                on_close={ ctx.props().on_close.clone() }
                classes={ classes!("external-book-search-popup") }
            >
                <h1>{"Book Search"}</h1>

                <form class="row">
                    <input id={input_id} name="book_search" placeholder="Search For Title" value={ self.input_value.clone() } />
                    <button onclick={
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
                                                        <div class="tab-bar">
                                                        {
                                                            for search.items.iter()
                                                                .map(|(name, values)| {
                                                                    let name2 = name.clone();

                                                                    html! {
                                                                        <div class="tab-bar-item" onclick={ ctx.link().callback(move |_| Msg::OnChangeTab(name2.clone())) }>
                                                                            { upper_case_first_char(name.clone()) } { format!(" ({})", values.len()) }
                                                                        </div>
                                                                    }
                                                                })
                                                        }
                                                        </div>

                                                        <div class="book-search-items">
                                                        {
                                                            for search.items.get(&self.selected_tab)
                                                                .iter()
                                                                .flat_map(|values| values.iter())
                                                                .map(|item| Self::render_ext_search_container(&self.selected_tab, item, ctx))
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

                <hr />

                {
                    if self.left_edit.is_some() {
                        html! {
                            <div>
                                <button onclick={ ctx.link().callback(|_| Msg::OnSubmitSingle) }>{ "Insert (Single)" }</button>
                                <button disabled={ true }>{ "Insert (Compared)" }</button>

                                <span class="yellow">{ "Select another to be able to compare and insert" }</span>
                            </div>
                        }
                    } else {
                        html! {}
                    }
                }
            </Popup>
        }
    }

    fn render_compare(&self, left_edit: BookEdit, right_edit: BookEdit, ctx: &Context<Self>) -> Html {
        html! {
            <PopupComparison
                compare={ left_edit.create_comparison_with(&right_edit).unwrap_throw() }
                show_equal_rows={ true }
                on_close={ ctx.props().on_close.clone() }
                on_submit={ ctx.link().callback(|v| Msg::OnSelectItem(SearchSelectedValue::BookEdit(Box::new(BookEdit::create_from_comparison(v).unwrap_throw())))) }
            />
        }
    }

    fn render_ext_search_container(site: &str, item: &SearchItem, ctx: &Context<Self>) -> Html {
        let item = item.as_book();

        let source = item.source.clone();

        html! {
            <div
                class="book-search-item"
                onclick={ ctx.link().callback(move |_| Msg::OnSelectItem(SearchSelectedValue::Source(source.clone()))) }
            >
                <img src={ item.thumbnail_url.to_string() } />
                <div class="book-info">
                    <h4 class="book-name">{ item.name.clone() }</h4>
                    <h5>{ site }</h5>
                    <span class="book-author">{ item.author.clone().unwrap_or_default() }</span>
                    <p class="book-author">{ item.description.clone()
                            .map(|mut v| { util::truncate_on_indices(&mut v, 300); v })
                            .unwrap_or_default() }
                    </p>
                </div>
            </div>
        }
    }

    fn render_loc_search_container(item: &DisplayItem, ctx: &Context<Self>) -> Html {
        let book_id = item.id;

        html! {
            <div
                class="book-search-item"
                onclick={ ctx.link().callback(move |_| Msg::OnSelectItem(SearchSelectedValue::BookId(book_id))) }
            >
                <img src={ item.get_thumb_url() } />
                <div class="book-info">
                    <h4 class="book-name">{ item.title.clone() }</h4>
                    <span class="book-author">{ item.cached.author.clone().unwrap_or_default() }</span>
                </div>
            </div>
        }
    }
}

#[derive(PartialEq)]
pub enum SearchSelectedValue {
    Source(Source),
    BookEdit(Box<BookEdit>),
    BookId(BookId)
}

impl SearchSelectedValue {
    pub fn into_external(self) -> Either<Source, BookEdit> {
        match self {
            Self::Source(v) => Either::Left(v),
            Self::BookEdit(v) => Either::Right(*v),
            _ => unreachable!(),
        }
    }

    pub fn into_local(self) -> BookId {
        match self {
            Self::BookId(v) => v,
            _ => unreachable!(),
        }
    }
}