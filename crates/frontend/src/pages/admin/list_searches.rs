use common::{api::{WrappingResponse, QueryListResponse}, ImageIdType};
use common_local::{SearchGroup, SearchType, SearchGroupId, api::{PostUpdateSearchIdBody, NewBookBody}};
use gloo_utils::window;
use yew::{prelude::*, html::Scope};

use crate::{components::popup::search::PopupSearch, request};


#[derive(Clone)]
pub enum Msg {
    // Requests
    RequestSearches,

    Find((SearchGroupId, String)),
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
                    let page = get_page_param().unwrap_or_default();
                    let limit = 25;

                    Msg::MembersResults(request::get_search_list(Some(page * limit), Some(limit)).await)
                });
            }

            Msg::MembersResults(resp) => {
                self.items_resp = Some(resp);
            }

            Msg::Find(v) => self.search_input = Some(v),
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
                                        if let WrappingResponse::Resp(Some(book)) = request::new_book(NewBookBody::Value(Box::new(v))).await {
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

                    Msg::CloseSearch
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


fn get_page_param() -> Option<usize> {
    let search_params = web_sys::UrlSearchParams::new_with_str(
        &gloo_utils::window().location().search().ok()?
    ).ok()?;

    let page = search_params.get("page")
        .and_then(|v| v.parse().ok())
        .unwrap_or_default();

    Some(page)
}