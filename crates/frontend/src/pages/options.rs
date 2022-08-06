use common::api::WrappingResponse;
use common_local::{api, update::OptionsUpdate};
use web_sys::HtmlSelectElement;
use yew::prelude::*;

use crate::request;

pub enum Msg {
    // Request Results
    SettingsResults(WrappingResponse<api::GetSettingsResponse>),

    UpdateSettings,
}

pub struct OptionsPage {
    resp: Option<WrappingResponse<api::GetSettingsResponse>>,
}

impl Component for OptionsPage {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            resp: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::SettingsResults(resp) => {
                self.resp = Some(resp);
            }

            Msg::UpdateSettings => {
                ctx.link()
                .send_future(async {
                    Msg::SettingsResults(request::get_settings().await)
                });
            }
        }

        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        if let Some(resp) = self.resp.as_ref() {
            let resp = crate::continue_or_html_err!(resp);

            html! {
                <div class="settings-view-container">
                    <div class="view-container">
                        <h2>{ "Settings" }</h2>

                        <div class="form-container shrink-width-to-content">
                            <label for="new-users-select">{ "Allow new user registration" }</label>
                            <select
                                id="new-users-select"
                                onchange={
                                    ctx.link().callback_future(|e: Event| {
                                        let value = e.target_unchecked_into::<HtmlSelectElement>().value();

                                        async move {
                                            request::update_settings(OptionsUpdate {
                                                user_signup: Some(value == "true"),

                                                .. Default::default()
                                            }).await;

                                            Msg::UpdateSettings
                                        }
                                    })
                                }
                            >
                                <option selected={ resp.config.auth.new_users } value="true">{ "Allowed" }</option>
                                <option selected={ !resp.config.auth.new_users } value="false">{ "Denied" }</option>
                            </select>

                            <br />

                            <a class="button" href="/auth/logout">{ "Logout" }</a>
                        </div>


                        // <button class="button" onclick={ ctx.link().callback_future(|_| async {
                        //     request::run_task().await;
                        //     Msg::Ignore
                        // }) }>{ "Run Library Scan + Metadata Updater" }</button>
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
            ctx.link()
            .send_future(async {
                Msg::SettingsResults(request::get_settings().await)
            });
        }
    }
}