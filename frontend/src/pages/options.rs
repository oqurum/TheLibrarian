use librarian_common::api;
use yew::prelude::*;

use crate::request;

pub enum Msg {
	// Request Results
	SettingsResults(api::GetSettingsResponse),

	Ignore,
}

pub struct OptionsPage {
	resp: Option<api::GetSettingsResponse>,
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

			Msg::Ignore => ()
		}

		true
	}

	fn view(&self, ctx: &Context<Self>) -> Html {
		if let Some(_resp) = self.resp.as_ref() {
			html! {
				<div class="settings-view-container">
					<div class="main-content-view">
						<h2>{ "Settings" }</h2>
						<a class="button" href="/auth/logout">{ "Logout" }</a>

						// <button class="button" onclick={ ctx.link().callback_future(|_| async {
						// 	request::run_task().await;
						// 	Msg::Ignore
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

impl OptionsPage {
	//
}