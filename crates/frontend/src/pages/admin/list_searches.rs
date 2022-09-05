use common::{api::{WrappingResponse, QueryListResponse}, ImageIdType};
use common_local::{SearchGroup, SearchType, SearchGroupId, api::{PostUpdateSearchIdBody, NewBookBody, SimpleListQuery}};
use gloo_utils::window;
use yew::{prelude::*, html::Scope};

use crate::{components::popup::search::{PopupSearch, SearchBy, SearchSelectedValue}, request};


#[derive(Clone)]
pub enum Msg {
    // Requests
    RequestSearches,
    AutoFindAll,

    Find((SearchGroupId, String)),
    CloseSearch,

    // Results
    MembersResults(WrappingResponse<QueryListResponse<SearchGroup>>),

    Ignore,
}

pub struct ListSearchesPage {
    items_resp: Option<WrappingResponse<QueryListResponse<SearchGroup>>>,
    search_input: Option<(SearchGroupId, String)>,
}

impl Component for ListSearchesPage {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        ctx.link().send_message(Msg::RequestSearches);

        Self {
            items_resp: None,
            search_input: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::RequestSearches => {
                ctx.link()
                .send_future(async move {
                    let query = SimpleListQuery::from_url_search_params();
                    Msg::MembersResults(request::get_search_list(query.offset, query.limit).await)
                });
            }

            Msg::AutoFindAll => {
                let searching_queries = match self.items_resp.as_ref().and_then(|v| v.as_ok().ok()) {
                    Some(v) => v.items.iter().map(|v| v.query.clone()).collect::<Vec<_>>(),
                    None => return false,
                };

                ctx.link()
                .send_future(async move {
                    log::info!("starting auto find all");

                    for value in searching_queries {
                        request::new_book(NewBookBody::FindAndAdd(value)).await;
                    }

                    log::info!("finished auto find all");

                    Msg::Ignore
                });
            }

            Msg::MembersResults(resp) => {
                self.items_resp = Some(resp);
            }

            Msg::Find(v) => self.search_input = Some(v),
            Msg::CloseSearch => self.search_input = None,
            Msg::Ignore => return false,
        }

        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        if let Some(resp) = self.items_resp.as_ref() {
            let resp = crate::continue_or_html_err!(resp);

            html! {
                <div class="view-container searches-list-view-container">
                    <div class="list-items">
                        <div class="search-item-card">
                            <button onclick={ Callback::from(|_| {
                                let mut query = SimpleListQuery::from_url_search_params();
                                query.set_page(query.get_page().saturating_sub(1));

                                let _ = window().location().set_href(&format!("{}?{}", window().location().pathname().unwrap(), query.to_query()));
                            }) }>{ "Previous Page" }</button>

                            <button onclick={ Callback::from(|_| {
                                let mut query = SimpleListQuery::from_url_search_params();
                                query.set_page(query.get_page() + 1);

                                let _ = window().location().set_href(&format!("{}?{}", window().location().pathname().unwrap(), query.to_query()));
                            }) }>{ "Next Page" }</button>

                            <button onclick={ ctx.link().callback(|_| Msg::AutoFindAll) } class="green">{ "Auto Find All" }</button>
                        </div>

                        { for resp.items.iter().map(|item| self.render_item(item, ctx.link())) }
                    </div>

                    {
                        if let Some((id, input_value)) = self.search_input.clone() {
                            html! {
                                <PopupSearch
                                    { input_value }
                                    type_of={ SearchBy::External }
                                    search_for={ SearchType::Book }
                                    comparable=false
                                    search_on_init=true

                                    on_close={ ctx.link().callback(|_| Msg::CloseSearch) }
                                    on_select={ ctx.link().callback_future(move |v: SearchSelectedValue| async move {
                                        // TODO: Handle Errors and responses
                                        if let WrappingResponse::Resp(Some(book)) = request::new_book(NewBookBody::Value(Box::new(v.into_external()))).await {
                                            request::update_search_item(
                                                id,
                                                PostUpdateSearchIdBody {
                                                    update_id: Some(Some(ImageIdType::new_book(book.id))),
                                                }
                                            ).await;
                                        }

                                        Msg::CloseSearch
                                    }) }
                                />
                            }
                        } else {
                            html! {}
                        }
                    }
                </div>
            }
        } else {
            html! {
                <h1>{ "Loading..." }</h1>
            }
        }
    }
}

impl ListSearchesPage {
    fn render_item(&self, item: &SearchGroup, scope: &Scope<Self>) -> Html {
        let find_cb = {
            let id = item.id;
            let value = item.query.clone();

            scope.callback(move |_| Msg::Find((id, value.clone())))
        };

        let auto_find_cb = {
            let value = item.query.clone();

            scope.callback_future(move |_| {
                let value = value.clone();
                async move {
                    request::new_book(NewBookBody::FindAndAdd(value)).await;

                    Msg::Ignore
                }
            })
        };

        let query = item.query.clone();

        html! {
            <div class="search-item-card">
                <div class="body">
                    <h4>{ "Query: " } { item.query.clone() }</h4>

                    <div>{ "Calls: " } { item.calls }</div>
                    <div>{ "Last Found Count: " } { item.last_found_amount }</div>
                    <div>{ "Month: " } { item.timeframe.format("%Y-%m") }</div>
                </div>

                <div class="footer">
                    <div>{ "Last Called: " } { item.updated_at.format("%a, %e %b %y %r %Z") }</div>
                    <div>{ "First Called: " } { item.created_at.format("%a, %e %b %y %r %Z") }</div>
                </div>

                <div class="tools">
                    <button class="yellow" onclick={ Callback::from(move |_| {
                        let _ = window().location().set_href(&format!("/?query={}", urlencoding::encode(&query)));
                    }) }>{ "View" }</button>
                    <button class="green" onclick={ find_cb }>{ "Find" }</button>
                    <button class="green" onclick={ auto_find_cb }>{ "Auto Find" }</button>
                    <button class="red disabled" disabled=true>{ "Delete" }</button>
                </div>
            </div>
        }
    }
}