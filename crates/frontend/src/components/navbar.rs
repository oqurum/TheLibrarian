use std::sync::{Arc, Mutex};

use common::api::WrappingResponse;
use common_local::api::{self, BookListQuery, GetBookListResponse};
use gloo_utils::{body, document};
use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_router::components::Link;

use crate::{is_signed_in, request, util, Route};

pub enum Msg {
    Close,
    SearchFor(String),
    SearchResults(WrappingResponse<GetBookListResponse>),
}

pub struct NavbarModule {
    left_items: Vec<(bool, Route, DisplayType)>,
    right_items: Vec<(bool, Route, DisplayType)>,

    search_results: Option<WrappingResponse<GetBookListResponse>>,
    #[allow(clippy::type_complexity)]
    closure: Arc<Mutex<Option<Closure<dyn FnMut(MouseEvent)>>>>,
}

impl Component for NavbarModule {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            left_items: vec![
                (
                    false,
                    Route::Home,
                    DisplayType::Icon("library_books", "Books"),
                ),
                (false, Route::People, DisplayType::Icon("person", "Authors")),
                (
                    false,
                    Route::EditList,
                    DisplayType::Icon("fact_check", "Edits"),
                ),
                (
                    false,
                    Route::Collections,
                    DisplayType::Icon("collections_bookmark", "Collections"),
                ),
            ],
            right_items: vec![
                (
                    true,
                    Route::Options,
                    DisplayType::Icon("settings", "Settings"),
                ),
                (true, Route::Logout, DisplayType::Icon("logout", "Logout")),
            ],

            search_results: None,
            closure: Arc::new(Mutex::new(None)),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Close => {
                self.search_results = None;
            }

            Msg::SearchFor(value) => {
                self.search_results = None;

                ctx.link().send_future(async move {
                    Msg::SearchResults(
                        request::get_books(BookListQuery {
                            search: Some(api::QueryType::Query(value)),
                            ..Default::default()
                        })
                        .await,
                    )
                });
            }

            Msg::SearchResults(res) => self.search_results = Some(res),
        }

        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let input_id = "book-search-input";

        html! {
            <nav class="navbar navbar-dark navbar-expand-sm text-bg-dark">
                <div class="container-fluid">
                    <button class="navbar-toggler" type="button" data-bs-toggle="collapse" data-bs-target="#navbarSupportedContent" aria-controls="navbarSupportedContent" aria-expanded="false" aria-label="Toggle navigation">
                        <span class="navbar-toggler-icon"></span>
                    </button>

                    // Collapsed Search
                    <div class="d-sm-none" style="max-width: 16em;">
                        <form class="search-bar row">
                            <div class="input-group">
                                <input id={input_id} class="form-control" placeholder="Search" />
                                <button for={input_id} class="btn btn-sm btn-secondary" onclick={
                                    ctx.link().callback(move |e: MouseEvent| {
                                        e.prevent_default();

                                        let input = document().get_element_by_id(input_id).unwrap().unchecked_into::<HtmlInputElement>();

                                        Msg::SearchFor(input.value())
                                    })
                                }>{ "Search" }</button>
                            </div>
                        </form>

                        { self.render_dropdown_results() }
                    </div>

                    // Collapsed List
                    <div class="collapse navbar-collapse" id="navbarSupportedContent">
                        <ul class="navbar-nav me-auto mb-2">
                            { for self.left_items.iter().map(|item| Self::render_item(item.0, item.1.clone(), &item.2)) }
                        </ul>

                        <div class="center-content me-auto d-none d-sm-block">
                            <form class="search-bar row">
                                <div class="input-group">
                                    <input id={input_id} class="form-control" placeholder="Search" />
                                    <button for={input_id} class="btn btn-sm btn-secondary" onclick={
                                        ctx.link().callback(move |e: MouseEvent| {
                                            e.prevent_default();

                                            let input = document().get_element_by_id(input_id).unwrap().unchecked_into::<HtmlInputElement>();

                                            Msg::SearchFor(input.value())
                                        })
                                    }>{ "Search" }</button>
                                </div>
                            </form>

                            { self.render_dropdown_results() }
                        </div>

                        <ul class="navbar-nav mb-2">
                            { for self.right_items.iter().map(|item| Self::render_item(item.0, item.1.clone(), &item.2)) }

                            {
                                if !is_signed_in() {
                                    Self::render_item(false, Route::Login, &DisplayType::Icon("login", "Login/Register"))
                                } else {
                                    html! {}
                                }
                            }
                        </ul>
                    </div>
                </div>
            </nav>
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, _first_render: bool) {
        if let Some(func) = (*self.closure.lock().unwrap()).take() {
            let _ =
                body().remove_event_listener_with_callback("click", func.as_ref().unchecked_ref());
        }

        let closure = Arc::new(Mutex::default());

        let link = ctx.link().clone();
        let on_click = Closure::wrap(Box::new(move |event: MouseEvent| {
            if let Some(target) = event.target() {
                if !util::does_parent_contain_class(&target.unchecked_into(), "search-bar") {
                    link.send_message(Msg::Close);
                }
            }
        }) as Box<dyn FnMut(MouseEvent)>);

        let _ = body().add_event_listener_with_callback("click", on_click.as_ref().unchecked_ref());

        *closure.lock().unwrap() = Some(on_click);

        self.closure = closure;
    }

    fn destroy(&mut self, _ctx: &Context<Self>) {
        let func = (*self.closure.lock().unwrap()).take().unwrap();
        let _ = body().remove_event_listener_with_callback("click", func.as_ref().unchecked_ref());
    }
}

impl NavbarModule {
    fn render_item(login_required: bool, route: Route, name: &DisplayType) -> Html {
        if login_required && !is_signed_in() {
            return html! {};
        }

        match name {
            // DisplayType::Text(v) => html! {
            //     <Link<Route> to={route}>{ v }</Link<Route>>
            // },
            DisplayType::Icon(icon, title) => html! {
                <li class="nav-item">
                    <Link<Route> to={ route }>
                        <span class="nav-link text-light material-icons" title={ *title }>{ icon }</span>
                        <span class="d-inline d-sm-none link-light ms-2">{ *title }</span>
                    </Link<Route>>
                </li>
            },
        }
    }

    fn render_dropdown_results(&self) -> Html {
        if let Some(resp) = self.search_results.as_ref() {
            let resp = crate::continue_or_html_err!(resp);

            html! {
                <div class="search-dropdown">
                    {
                        for resp.items.iter().map(|item| {
                            html_nested! {
                                <Link<Route> to={Route::ViewMeta { meta_id: item.id }} classes={ classes!("search-item") }>
                                    <div class="poster max-vertical">
                                        <img src={ item.get_thumb_url() } />
                                    </div>
                                    <div class="info">
                                        <h5 class="book-name" title={ item.title.clone() }>{ item.title.clone() }</h5>
                                    </div>
                                </Link<Route>>
                            }
                        })
                    }
                </div>
            }
        } else {
            html! {}
        }
    }
}

pub enum DisplayType {
    // Text(&'static str),
    Icon(&'static str, &'static str),
}
