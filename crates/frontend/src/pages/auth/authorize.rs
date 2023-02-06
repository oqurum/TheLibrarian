use serde::{Deserialize, Serialize};
use wasm_bindgen::UnwrapThrowExt;
use yew::{html::Scope, prelude::*};

use crate::get_member_self;

pub struct AuthorizePage;

impl Component for AuthorizePage {
    type Message = ();
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn update(&mut self, _ctx: &Context<Self>, _msg: Self::Message) -> bool {
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        match AuthorizeQuery::load() {
            Ok(query) => {
                html! {
                    <div class="login-container">
                        <div class="center-normal">
                            <div class="center-container">
                                <AuthForm cb={ ctx.link().clone() } { query } />
                            </div>
                        </div>
                    </div>
                }
            }

            Err(err) => {
                html! {
                    <div class="login-container">
                        <div class="center-normal">
                            <div class="center-container">
                                <h2>{ err }</h2>
                            </div>
                        </div>
                    </div>
                }
            }
        }
    }
}

#[derive(Properties)]
struct InnerProps {
    cb: Scope<AuthorizePage>,
    query: AuthorizeQuery,
}

impl PartialEq for InnerProps {
    fn eq(&self, other: &Self) -> bool {
        self.query == other.query
    }
}

#[function_component(AuthForm)]
fn _auth_form(props: &InnerProps) -> Html {
    let query = props.query.clone();
    let member = get_member_self().unwrap();

    html! {
        <>
            <h2>{ "Server Linking" }</h2>

            <br />

            <b>{ "Hey, You! Yes, You!" }</b>
            <div>{ "You're about to link your account to a users book server." }</div>

            <br />

            <div>
                <h4>{ "Account" }</h4>

                <div>
                    <b>{ "Name: " }</b>
                    <span>{ member.name }</span>
                </div>

                {
                    if let Some(email) = member.email.clone() {
                        html! {
                            <div>
                                <b>{ "Email: " }</b>
                                <span>{ email }</span>
                            </div>
                        }
                    } else {
                        html! {}
                    }
                }
            </div>

            <br />

            <div>
                <h4>{ "External Book Server" }</h4>

                {
                    if let Some(name) = query.server_owner_name.clone() {
                        html! {
                            <div>
                                <b>{ "Owner Name: " }</b>
                                <span>{ name }</span>
                            </div>
                        }
                    } else {
                        html! {}
                    }
                }

                {
                    if let Some(name) = query.server_name.clone() {
                        html! {
                            <div>
                                <b>{ "Server Name: " }</b>
                                <span>{ name }</span>
                            </div>
                        }
                    } else {
                        html! {}
                    }
                }

                {
                    if let Some(id) = query.server_id.clone() {
                        html! {
                            <div>
                                <b>{ "Server ID: " }</b>
                                <span>{ id }</span>
                            </div>
                        }
                    } else {
                        html! {}
                    }
                }
            </div>

            <div class="badge bg-danger">
                <b>{ "Only do this if you trust the user running the server!" }</b>
            </div>

            <br />

            <form action="/auth/link" method="POST" enctype="application/x-www-form-urlencoded">
                <input name="server_owner_name" value={ query.server_owner_name } type="text" style="display: none;" />
                <input name="server_name" value={ query.server_name } type="text" style="display: none;" />
                <input name="server_id" value={ query.server_id } type="text" style="display: none;" />
                <input name="redirect_uri" value={ query.redirect_uri } type="text" style="display: none;" />
                <input name="state" value={ query.state } type="text" style="display: none;" />
                <input name="scope" value={ query.scope } type="text" style="display: none;" />

                <button type="submit">{ "Authenticate" }</button>
            </form>
        </>
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthorizeQuery {
    pub server_owner_name: Option<String>,
    pub server_name: Option<String>,
    pub server_id: Option<String>,
    pub redirect_uri: Option<String>,
    pub state: Option<String>,
    pub scope: Option<String>,
}

impl AuthorizeQuery {
    pub fn load() -> std::result::Result<Self, &'static str> {
        let q = gloo_utils::window().location().search().unwrap_or_default();

        if q.is_empty() {
            Err("Invalid Query")
        } else {
            let this: Self = serde_qs::from_str(&q[1..]).unwrap_throw();

            if let Some(err) = this.is_missing_field() {
                Err(err)
            } else {
                Ok(this)
            }
        }
    }

    fn is_missing_field(&self) -> Option<&'static str> {
        if self.state.is_none() && self.redirect_uri.is_none() && self.server_id.is_none() {
            return Some("Invalid Query");
        }

        if self.state.is_none() {
            return Some("State");
        }

        if self.redirect_uri.is_none() {
            return Some("Redirect URI");
        }

        if self.scope.is_none() {
            return Some("Scope");
        }

        if self.server_id.is_none()
            && !self
                .scope
                .as_deref()
                .map(|v| v == "server_register")
                .unwrap_or_default()
        {
            return Some("Server ID");
        }

        None
    }
}
