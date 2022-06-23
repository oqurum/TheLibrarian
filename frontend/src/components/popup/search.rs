use librarian_common::{api::{ExternalSearchResponse, SearchItem, self}, SearchType, Source, util::string_to_upper_case, item::edit::BookEdit};
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
	pub search_for: SearchType,
}


pub enum Msg {
	BookSearchResponse(String, api::WrappingResponse<ExternalSearchResponse>),

	SearchFor(String),

	OnChangeTab(String),

	OnSelect(Source),

	Ignore,
}


pub struct PopupSearch {
	cached_posters: Option<LoadingItem<api::WrappingResponse<ExternalSearchResponse>>>,
	input_value: String,

	left_edit: Option<BookEdit>,
	right_edit: Option<BookEdit>,

	selected_tab: String,
}

impl Component for PopupSearch {
	type Message = Msg;
	type Properties = Property;

	fn create(ctx: &Context<Self>) -> Self {
		Self {
			cached_posters: None,
			input_value: ctx.props().input_value.clone(),

			left_edit: None,
			right_edit: None,

			selected_tab: String::new(),
		}
	}

	fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
		match msg {
			Msg::Ignore => {
				return false;
			}

			Msg::SearchFor(search) => {
				self.cached_posters = Some(LoadingItem::Loading);

				let search_for = ctx.props().search_for;

				ctx.link()
				.send_future(async move {
					let resp = request::external_search_for(&search, search_for).await;

					Msg::BookSearchResponse(search, resp)
				});
			}

			Msg::BookSearchResponse(search, resp) => {
				if let Some(name) = resp.resp.as_ref().and_then(|v| v.items.keys().next()).cloned() {
					self.selected_tab = name;
				}

				self.cached_posters = Some(LoadingItem::Loaded(resp));
				self.input_value = search;
			}

			Msg::OnSelect(source) => {
				ctx.props().on_select.emit(source);
			}

			Msg::OnChangeTab(name) => {
				self.selected_tab = name;
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

				<hr />

				<div class="external-book-search-container">
					{
						if let Some(loading) = self.cached_posters.as_ref() {
							match loading {
								LoadingItem::Loaded(wrapper) => {
									match wrapper.as_ok() {
										Ok(search) => html! {
											<>
												<div class="tab-bar">
												{
													for search.items.iter()
														.map(|(name, values)| {
															let name2 = name.clone();

															html! {
																<div class="tab-bar-item" onclick={ ctx.link().callback(move |_| Msg::OnChangeTab(name2.clone())) }>
																	{ string_to_upper_case(name.clone()) } { format!(" ({})", values.len()) }
																</div>
															}
														})
												}
												</div>

												<div class="book-search-items">
												{
													for search.items.get(&self.selected_tab)
														.iter()
														.flat_map(|values| values.iter())
														.map(|item| Self::render_poster_container(&self.selected_tab, item, ctx))
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

				<hr />

				{
					match (self.left_edit.is_some(), self.right_edit.is_some()) {
						(true, false) => html! {
							<>
								<button>{ "Insert" }</button>
							</>
						},

						_ => html! {}
					}
				}
			</Popup>
		}
	}
}

impl PopupSearch {
	fn render_poster_container(site: &str, item: &SearchItem, ctx: &Context<Self>) -> Html {
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