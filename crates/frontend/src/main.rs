use std::sync::{Arc, Mutex};

use common::{BookId, PersonId, api::{WrappingResponse, ApiErrorResponse}};
use common_local::{api, Member};
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


#[macro_export]
macro_rules! continue_or_html_err {
    ($value:ident) => {
        match $value.as_ok() {
            Ok(v) => v,
            Err(e) => return html! { <> { "An Error Occured: " } { e } </> }
        }
    };
}



enum Msg {
    LoadMemberSelf(WrappingResponse<api::GetMemberSelfResponse>)
}

struct Model {
    has_loaded_member: bool,
    error: Option<ApiErrorResponse>,
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
            error: None,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::LoadMemberSelf(member) => {
                match member.ok() {
                    Ok(resp) => {
                        *MEMBER_SELF.lock().unwrap() = resp.member;
                        self.has_loaded_member = true;
                    }

                    Err(e) => {
                        self.error = Some(e);
                    }
                }

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
                    } else if let Some(err) = self.error.as_ref() {
                        html! {
                            <div>
                                <h1>{ err.description.clone() }</h1>
                            </div>
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

    #[at("/logout")]
    Logout,

    #[at("/book/:meta_id")]
    ViewMeta { meta_id: BookId },

    #[at("/people")]
    People,

    #[at("/person/:id")]
    Person { id: PersonId },

    #[at("/edits")]
    EditList,

    #[at("/options")]
    Options,

    #[at("/authorize")]
    Authorize,

    #[at("/")]
    #[not_found]
    Home
}


fn switch(route: &Route) -> Html {
    log::info!("{:?}", route);

    match route.clone() {
        Route::Authorize => {
            html! { <pages::AuthorizePage /> }
        }

        Route::Login => {
            html! { <pages::LoginPage /> }
        }

        Route::Logout => {
            html! { <pages::LogoutPage /> }
        }

        Route::ViewMeta { meta_id } => {
            html! { <pages::BookView id={meta_id} /> }
        }

        Route::People => {
            html! { <pages::AuthorListPage /> }
        }

        Route::Person { id } => {
            html! { <pages::AuthorView id={id} /> }
        }

        Route::EditList => {
            html! { <pages::EditListPage /> }
        }

        Route::Options => {
            // Require a sign in for the Options Page
            if !is_signed_in() {
                return html! { <pages::LoginPage /> };
            }

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