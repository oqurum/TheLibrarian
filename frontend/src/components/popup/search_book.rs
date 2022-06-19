use librarian_common::{api::{ExternalSearchResponse, SearchItem, self}, SearchType, Source};
use gloo_utils::document;
use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;
use yew::prelude::*;

use crate::{request, util::{self, LoadingItem}};

use super::{Popup, PopupType, YEW_CLOSE_POPUP};


#[derive(Properties, PartialEq)]
pub struct Property {
	#[prop_or_default]
    pub classes: Classes,

	pub on_close: Callback<()>,
	pub on_select: Callback<Source>,

	pub input_value: String,
}


pub enum Msg {
	BookSearchResponse(String, api::WrappingResponse<ExternalSearchResponse>),

	SearchFor(String),

	OnSelect(Source),

	Ignore,
}


pub struct PopupSearchBook {
	cached_posters: Option<LoadingItem<api::WrappingResponse<ExternalSearchResponse>>>,
	input_value: String,
}

impl Component for PopupSearchBook {
	type Message = Msg;
	type Properties = Property;

	fn create(ctx: &Context<Self>) -> Self {
		Self {
			cached_posters: None,
			input_value: ctx.props().input_value.clone(),
		}
	}

	fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
		match msg {
			Msg::Ignore => {
				return false;
			}

			Msg::SearchFor(search) => {
				self.cached_posters = Some(LoadingItem::Loading);

				ctx.link()
				.send_future(async move {
					let resp = request::external_search_for(&search, SearchType::Book).await;

					Msg::BookSearchResponse(search, resp)
				});
			}

			Msg::BookSearchResponse(search, resp) => {
				self.cached_posters = Some(LoadingItem::Loaded(resp));
				self.input_value = search;
			}

			Msg::OnSelect(source) => {
				ctx.props().on_select.emit(source);
			}
		}

		true
	}

	fn view(&self, ctx: &Context<Self>) -> Html {
		let input_id = "external-book-search-input";

		html! {
			<Popup
				type_of={ PopupType::FullOverlay }
				on_close={ ctx.props().on_close.clone() }
				classes={ classes!("external-book-search-popup") }
			>
				<h1>{"Book Search"}</h1>

				<form class="row">
					<input id={input_id} name="book_search" placeholder="Search For Title" value={ self.input_value.clone() } />
					<button onclick={
						ctx.link().callback(move |e: MouseEvent| {
							e.prevent_default();

							let input = document().get_element_by_id(input_id).unwrap().unchecked_into::<HtmlInputElement>();

							Msg::SearchFor(input.value())
						})
					}>{ "Search" }</button>
				</form>

				<div class="external-book-search-container">
					{
						if let Some(resp) = self.cached_posters.as_ref() {
							match resp {
								LoadingItem::Loaded(resp) => {
									match resp.as_ok() {
										Ok(resp) => html! {
											<>
												<h2>{ "Results" }</h2>
												<div class="book-search-items">
												{
													for resp.items.iter()
														.flat_map(|(name, values)| values.iter().map(|v| (name.clone(), v)))
														.map(|(site, item)| Self::render_poster_container(site, item, ctx))
												}
												</div>
											</>
										},

										Err(e) => html! {
											<h2>{ e }</h2>
										}
									}

								},

								LoadingItem::Loading => html! {
									<h2>{ "Loading..." }</h2>
								}
							}
						} else {
							html! {}
						}
					}
				</div>
			</Popup>
		}
	}
}

impl PopupSearchBook {
	fn render_poster_container(site: String, item: &SearchItem, ctx: &Context<Self>) -> Html {
		let item = item.as_book();

		let source = item.source.clone();

		html! {
			<div
				class="book-search-item"
				{YEW_CLOSE_POPUP}
				onclick={ ctx.link().callback(move |_| Msg::OnSelect(source.clone())) }
			>
				<img src={ item.thumbnail_url.to_string() } />
				<div class="book-info">
					<h4 class="book-name">{ item.name.clone() }</h4>
					<h5>{ site }</h5>
					<span class="book-author">{ item.author.clone().unwrap_or_default() }</span>
					<p class="book-author">{ item.description.clone()
							.map(|mut v| { util::truncate_on_indices(&mut v, 300); v })
							.unwrap_or_default() }
					</p>
				</div>
			</div>
		}
	}
}