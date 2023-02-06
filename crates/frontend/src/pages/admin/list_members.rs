use common::api::{QueryListResponse, WrappingResponse};
use common_local::Member;
use yew::{html::Scope, prelude::*};

use crate::{get_member_self, request};

#[derive(Clone)]
pub enum Msg {
    // Requests
    RequestMembers,

    // Results
    MembersResults(WrappingResponse<QueryListResponse<Member>>),
}

pub struct ListMembersPage {
    items_resp: Option<WrappingResponse<QueryListResponse<Member>>>,
}

impl Component for ListMembersPage {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        ctx.link().send_message(Msg::RequestMembers);

        Self { items_resp: None }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::RequestMembers => {
                ctx.link().send_future(async move {
                    Msg::MembersResults(request::get_member_list(None, None).await)
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
                <div class="view-container member-list-view-container">
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

impl ListMembersPage {
    fn render_item(&self, item: &Member, _scope: &Scope<Self>) -> Html {
        html! {
            <div class="member-item-card">
                <div class="title-container">
                    <span class="badge badge-secondary index">{ item.id }</span>
                    <h4 class="name">{ item.name.as_str() }</h4>
                </div>

                <h5 class="email">{ item.email.as_deref().unwrap_or_default() }</h5>

                <span class="created">{ item.created_at.to_rfc2822() }</span>

                <div class="tools">
                    {
                        if item.id == get_member_self().unwrap().id {
                            html! {
                                <span class="badge bg-success">{ "This is YOU!" }</span>
                            }
                        } else {
                            html! {
                                // <button class="red">{ "Delete" }</button>
                            }
                        }
                    }
                </div>
            </div>
        }
    }
}
