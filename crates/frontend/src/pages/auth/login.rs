use common::api::ApiErrorResponse;
use gloo_utils::window;
use wasm_bindgen::UnwrapThrowExt;
use web_sys::HtmlInputElement;
use yew::{html::Scope, prelude::*};
use yew_router::prelude::RouterScopeExt;

use crate::{request, Route};

pub enum Msg {
    LoginPasswordResponse(std::result::Result<String, ApiErrorResponse>),
    LoginPasswordlessResponse(std::result::Result<String, ApiErrorResponse>),
}

pub struct LoginPage {
    password_response: Option<std::result::Result<String, ApiErrorResponse>>,
    passwordless_response: Option<std::result::Result<String, ApiErrorResponse>>,
    // prevent_submit: bool,
}

impl Component for LoginPage {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            password_response: None,
            passwordless_response: None,
            // prevent_submit: false,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::LoginPasswordResponse(resp) => {
                if resp.is_ok() {
                    let location = ctx.link().location().unwrap();
                    if location.path() == "/login" {
                        let nav = ctx.link().navigator().unwrap();
                        nav.push(&Route::Home);
                    } else {
                        window().location().reload().unwrap_throw();
                    }
                }

                self.password_response = Some(resp);
            }

            Msg::LoginPasswordlessResponse(resp) => {
                if resp.is_ok() {
                    let location = ctx.link().location().unwrap();

                    if location.path() == "/login" {
                        let nav = ctx.link().navigator().unwrap();
                        nav.push(&Route::Home);
                    } else {
                        window().location().reload().unwrap_throw();
                    }
                }

                self.passwordless_response = Some(resp);
            }
        }

        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div class="login-container">
                <div class="center-normal">
                    <div class="center-container">
                        <PasswordLogin cb={ ctx.link().clone() } />
                        <PasswordlessLogin cb={ ctx.link().clone() } />
                    </div>
                </div>
            </div>
        }
    }
}

#[derive(Properties)]
struct InnerProps {
    cb: Scope<LoginPage>,
}

impl PartialEq for InnerProps {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}

#[function_component(PasswordlessLogin)]
fn passwordless(props: &InnerProps) -> Html {
    let passless_email = use_state(String::new);

    let on_change_passless_email = {
        let value = passless_email.setter();
        Callback::from(move |e: Event| {
            value.set(e.target_unchecked_into::<HtmlInputElement>().value())
        })
    };

    let submit_passless = {
        props.cb.callback_future(move |e: SubmitEvent| {
            e.prevent_default();

            let email = passless_email.clone();

            async move {
                let resp = request::login_without_password(email.to_string()).await;

                Msg::LoginPasswordlessResponse(resp.ok())
            }
        })
    };

    html! {
        <>
            <h2>{ "Passwordless Login" }</h2>
            <form class="mb-2" onsubmit={ submit_passless }>
                <label for="emailpassless">{ "Email Address" }</label>
                <input class="form-control" type="email" name="email" id="emailpassless" onchange={ on_change_passless_email } />

                <input class="btn btn-primary" type="submit" value="Log in" />
            </form>
        </>
    }
}

#[function_component(PasswordLogin)]
fn password(props: &InnerProps) -> Html {
    let pass_email = use_state(String::new);
    let pass_pass = use_state(String::new);

    let on_change_pass_email = {
        let value = pass_email.setter();
        Callback::from(move |e: Event| {
            value.set(e.target_unchecked_into::<HtmlInputElement>().value())
        })
    };

    let on_change_pass_pass = {
        let value = pass_pass.setter();
        Callback::from(move |e: Event| {
            value.set(e.target_unchecked_into::<HtmlInputElement>().value())
        })
    };

    let submit_pass = {
        props.cb.callback_future(move |e: SubmitEvent| {
            e.prevent_default();

            let email = pass_email.clone();
            let pass = pass_pass.clone();

            async move {
                let resp = request::login_with_password(email.to_string(), pass.to_string()).await;

                Msg::LoginPasswordResponse(resp.ok())
            }
        })
    };

    html! {
        <>
            <h2>{ "Password Login" }</h2>
            <form class="mb-2" onsubmit={ submit_pass }>
                <label for="email">{ "Email Address" }</label>
                <input class="form-control" type="email" name="email" id="email" onchange={ on_change_pass_email } />

                <label for="password">{ "Password" }</label>
                <input class="form-control" type="password" name="password" id="password" onchange={ on_change_pass_pass } />

                <input class="btn btn-primary" type="submit" value="Log in" />
            </form>
        </>
    }
}
