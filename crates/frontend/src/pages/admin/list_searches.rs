use common::{api::{WrappingResponse, QueryListResponse}, ImageIdType};
use common_local::{SearchGroup, SearchType, SearchGroupId, api::PostUpdateSearchIdBody};
use gloo_utils::window;
use yew::{prelude::*, html::Scope};

use crate::{components::popup::search::PopupSearch, request};


#[derive(Clone)]
pub enum Msg {
    // Requests
    RequestSearches,

    Search((SearchGroupId, String)),
    CloseSearch,

    // Results
    MembersResults(WrappingResponse<QueryListResponse<SearchGroup>>),
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
                    Msg::MembersResults(request::get_search_list(None, None).await)
                });
            }

            Msg::MembersResults(resp) => {
                self.items_resp = Some(resp);
            }

            Msg::Search(v) => self.search_input = Some(v),
            Msg::CloseSearch => self.search_input = None,
        }

        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        if let Some(resp) = self.items_resp.as_ref() {
            let resp = crate::continue_or_html_err!(resp);

            html! {
                <div class="view-container searches-list-view-container">
                    <div class="list-items">
                        { for resp.items.iter().map(|item| self.render_item(item, ctx.link())) }
                    </div>

                    {
                        if let Some((id, input_value)) = self.search_input.clone() {
                            html! {
                                <PopupSearch
                                    { input_value }
                                    search_for={ SearchType::Book }
                                    comparable=false
                                    search_on_init=true

                                    on_close={ ctx.link().callback(|_| Msg::CloseSearch) }
                                    on_select={ ctx.link().callback_future(move |v| async move {
                                        // TODO: Handle Errors and responses
                                        if let WrappingResponse::Resp(Some(book)) = request::new_book(v).await {
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
        let id = item.id;
        let value = item.query.clone();
        let find_cb = scope.callback(move |_| Msg::Search((id, value.clone())));

        let value = item.query.clone();

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
                        let _ = window().location().set_href(&format!("/?query={}", urlencoding::encode(&value)));
                    }) }>{ "View" }</button>
                    <button class="green" onclick={ find_cb }>{ "Find" }</button>
                    <button class="red disabled" disabled=true>{ "Delete" }</button>
                </div>
            </div>
        }
    }
}