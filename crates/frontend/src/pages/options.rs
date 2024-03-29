use common::api::WrappingResponse;
use common_local::{
    api,
    item::member::{MemberSettings, PageView},
    update::OptionsUpdate,
};
use wasm_bindgen::UnwrapThrowExt;
use web_sys::HtmlSelectElement;
use yew::prelude::*;

use crate::{get_member_self, request};

pub enum Msg {
    // Request Results
    SettingsResults(Box<WrappingResponse<api::GetSettingsResponse>>),

    UpdateSettings,
}

pub struct OptionsPage {
    resp: Option<WrappingResponse<api::GetSettingsResponse>>,
}

impl Component for OptionsPage {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self { resp: None }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::SettingsResults(resp) => {
                self.resp = Some(*resp);
            }

            Msg::UpdateSettings => {
                ctx.link().send_future(async {
                    Msg::SettingsResults(Box::new(request::get_settings().await))
                });
            }
        }

        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        if let Some(resp) = self.resp.as_ref() {
            let resp = crate::continue_or_html_err!(resp);

            let member = get_member_self().unwrap_throw();

            html! {
                <div class="settings-view-container">
                    <div class="view-container">
                        <h2>{ "Settings" }</h2>

                        <div class="mb-2 shrink-width-to-content">
                            <label for="new-users-select">{ "Allow new user registration" }</label>
                            <select
                                class="form-select"
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
                        </div>

                        <h3>{ "My Settings" }</h3>

                        <div class="mb-2 shrink-width-to-content">
                            <label for="page-view-type">{ "Default Page View Type" }</label>
                            <select
                                class="form-select"
                                id="page-view-type"
                                onchange={
                                    ctx.link().callback_future(|e: Event| {
                                        let index = e.target_unchecked_into::<HtmlSelectElement>().selected_index();

                                        async move {
                                            request::update_settings(OptionsUpdate {
                                                member: Some(MemberSettings {
                                                    page_view: Some(if index == 0 { PageView::Viewing } else { PageView::Editing }),
                                                }),

                                                .. Default::default()
                                            }).await;

                                            Msg::UpdateSettings
                                        }
                                    })
                                }
                            >
                                <option selected={ member.localsettings.page_view.map(|v| v.is_viewing()).unwrap_or(true) }>{ "Viewing" }</option>
                                <option selected={ member.localsettings.page_view.map(|v| v.is_editing()).unwrap_or(false) }>{ "Editing" }</option>
                            </select>
                        </div>

                        <br />

                        <div>
                            <a class="btn btn-danger" href="/auth/logout">{ "Logout" }</a>
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
            ctx.link().send_message(Msg::UpdateSettings);
        }
    }
}
