use common::api::WrappingResponse;
use common_local::{api::{self, UpdateCollectionModel}, SearchType};
use yew::prelude::*;

use crate::{components::{PopupSearch, LoginBarrier, popup::{SearchBy, search::SearchSelectedValue}}, request, pages::home::MediaItem};



pub enum Msg {
    // Retrive
    RetrieveMediaView(WrappingResponse<api::GetCollectionResponse>),
    BooksListResults(WrappingResponse<api::GetBookListResponse>),

    ToggleBookSearch,
}

#[derive(Properties, PartialEq, Eq)]
pub struct Property {
    pub path: String,
}

pub struct CollectionView {
    collection: Option<WrappingResponse<api::GetCollectionResponse>>,
    books: Option<WrappingResponse<api::GetBookListResponse>>,

    popup_search: bool,
}

impl Component for CollectionView {
    type Message = Msg;
    type Properties = Property;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            collection: None,
            books: None,

            popup_search: false,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::BooksListResults(resp) => {
                self.books = Some(resp);
            }

            Msg::RetrieveMediaView(value) => {
                self.collection = Some(value);
            }

            Msg::ToggleBookSearch => {
                self.popup_search = !self.popup_search;
            }
        }

        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let resp = match self.collection.as_ref() {
            Some(v) => Some(crate::continue_or_html_err!(v)),
            None => None,
        };

        if let Some(value) = resp.and_then(|v| v.value.as_ref()) {
            html! {
                <div class="outer-view-container">
                    <div class="sidebar-container">
                        <LoginBarrier>
                            <div class="sidebar-item">
                                <button class="button">{ "Start Editing" }</button>
                            </div>
                        </LoginBarrier>
                    </div>

                    <div class="view-container item-view-container">
                        <div class="info-container">
                            <div class="metadata-container">
                                <div class="metadata">
                                    // Book Display Info
                                    <h3 class="title">{ value.name.clone() }</h3>
                                    <p class="description">{ value.description.clone().unwrap_or_default() }</p>
                                </div>
                            </div>
                        </div>

                        <section>
                            <h2>{ "Books" }</h2>
                            <div class="books-container">
                                <div class="book-list normal horizontal">
                                    <div class="book-list-item new-container" title="Add Book" onclick={ ctx.link().callback(|_| Msg::ToggleBookSearch) }>
                                        <span class="material-icons">{ "add" }</span>
                                    </div>
                                    {
                                        if let Some(resp) = self.books.as_ref() {
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

                        {
                            if self.popup_search {
                                let coll_id = value.id;

                                html! {
                                    <PopupSearch
                                        type_of={ SearchBy::Local }
                                        search_for={ SearchType::Book }

                                        on_close={ ctx.link().callback(|_| Msg::ToggleBookSearch) }
                                        on_select={ ctx.link().callback_future(move |v: SearchSelectedValue| async move {
                                            request::update_collection(
                                                coll_id,
                                                &UpdateCollectionModel {
                                                    name: None,
                                                    description: None,
                                                    added_books: Some(vec![v.into_local()])
                                                }
                                            ).await;

                                            Msg::ToggleBookSearch
                                        }) }
                                    />
                                }
                            } else {
                                html! {}
                            }
                        }
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
            let path = ctx.props().path.clone();

            ctx.link().send_future(async move {
                Msg::RetrieveMediaView(request::get_collection(&path).await)
            });

            let path = ctx.props().path.clone();

            ctx.link().send_future(async move {
                Msg::BooksListResults(request::get_collection_books(&path).await)
            });
        }
    }
}