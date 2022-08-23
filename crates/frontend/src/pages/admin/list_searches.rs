use common::api::{WrappingResponse, QueryListResponse};
use common_local::SearchGroup;
use yew::{prelude::*, html::Scope};

use crate::request;


#[derive(Clone)]
pub enum Msg {
    // Requests
    RequestSearches,

    // Results
    MembersResults(WrappingResponse<QueryListResponse<SearchGroup>>),
}

pub struct ListSearchesPage {
    items_resp: Option<WrappingResponse<QueryListResponse<SearchGroup>>>,
}

impl Component for ListSearchesPage {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        ctx.link().send_message(Msg::RequestSearches);

        Self {
            items_resp: None,
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
    fn render_item(&self, item: &SearchGroup, _scope: &Scope<Self>) -> Html {
        html! {
            <div class="search-item-card">
                <div class="body">
                    <div>{ "Calls: " } { item.calls }</div>
                    <div>{ "Last Found Count: " } { item.last_found_amount }</div>
                    <div>{ "Month: " } { item.timeframe.format("%Y-%m") }</div>

                    <div>{ "Query: " } { item.query.clone() }</div>
                </div>

                <div class="footer">
                    <div>{ "Last Called: " } { item.updated_at.format("%a, %e %b %y %r %Z") }</div>
                    <div>{ "First Called: " } { item.created_at.format("%a, %e %b %y %r %Z") }</div>
                </div>

                <div class="tools">
                    <button class="green">{ "Find" }</button>
                    <button class="red">{ "Delete" }</button>
                </div>
            </div>
        }
    }
}