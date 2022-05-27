use std::sync::{Arc, Mutex};

use librarian_common::{api, Member, PersonId, BookId};
use lazy_static::lazy_static;
use yew::prelude::*;
use yew_router::prelude::*;

use components::NavbarModule;

mod util;
mod pages;
mod request;
mod components;


lazy_static! {
	pub static ref MEMBER_SELF: Arc<Mutex<Option<Member>>> = Arc::new(Mutex::new(None));
}

pub fn get_member_self() -> Option<Member> {
	MEMBER_SELF.lock().unwrap().clone()
}

pub fn is_signed_in() -> bool {
	get_member_self().is_some()
}


enum Msg {
	LoadMemberSelf(api::GetMemberSelfResponse)
}

struct Model {
	has_loaded_member: bool
}

impl Component for Model {
	type Message = Msg;
	type Properties = ();

	fn create(ctx: &Context<Self>) -> Self {
		ctx.link()
		.send_future(async {
			Msg::LoadMemberSelf(request::get_member_self().await)
		});

		Self {
			has_loaded_member: false,
		}
	}

	fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
		match msg {
			Msg::LoadMemberSelf(member) => {
				*MEMBER_SELF.lock().unwrap() = member.member;
				self.has_loaded_member = true;
			}
		}

		true
	}

	fn view(&self, _ctx: &Context<Self>) -> Html {
		html! {
			<BrowserRouter>
				<NavbarModule />
				{
					if self.has_loaded_member {
						html! {
							<Switch<Route> render={Switch::render(switch)} />
						}
					} else {
						html! {
							<div>
								<h1>{ "Loading..." }</h1>
							</div>
						}
					}
				}
			</BrowserRouter>
		}
	}
}


#[derive(Routable, PartialEq, Clone, Debug)]
pub enum Route {
	#[at("/login")]
	Login,

	#[at("/book/:meta_id")]
	ViewMeta { meta_id: BookId },

	#[at("/people")]
	People,

	#[at("/person/:id")]
	Person { id: PersonId },

	#[at("/options")]
	Options,

	#[at("/")]
	#[not_found]
	Home
}


fn switch(route: &Route) -> Html {
	log::info!("{:?}", route);

	if !is_signed_in() {
		return html! { <pages::LoginPage /> };
	}

	match route.clone() {
		Route::Login => {
			html! { <pages::LoginPage /> }
		}

		Route::ViewMeta { meta_id } => {
			html! { <pages::MediaView id={meta_id} /> }
		}

		Route::People => {
			html! { <pages::AuthorListPage /> }
		}

		Route::Person { id } => {
			html! { <pages::AuthorView id={id} /> }
		}

		Route::Options => {
			html! { <pages::OptionsPage /> }
		}

		Route::Home => {
			html! { <pages::HomePage /> }
		}
	}
}


fn main() {
	wasm_logger::init(wasm_logger::Config::default());

	yew::start_app::<Model>();
}