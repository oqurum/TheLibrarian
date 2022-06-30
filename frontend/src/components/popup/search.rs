use librarian_common::{api::{SearchItem, self}, SearchType, Source, util::string_to_upper_case, item::edit::BookEdit, Either};
use gloo_utils::document;
use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;
use yew::prelude::*;

use crate::{request, util::{self, LoadingItem}};

use super::{Popup, PopupType, book_update_with_meta::PopupBookUpdateWithMeta};


#[derive(Properties, PartialEq)]
pub struct Property {
	#[prop_or_default]
    pub classes: Classes,

	pub on_close: Callback<()>,
	pub on_select: Callback<Either<Source, BookEdit>>,

	pub input_value: String,
	pub search_for: SearchType,
}


pub enum Msg {
	BookSearchResponse(String, api::WrappingResponse<api::ExternalSearchResponse>),
	BookItemResponse(Source, api::WrappingResponse<api::ExternalSourceItemResponse>),

	SearchFor(String),

	OnChangeTab(String),

	OnSelectItem(Source),

	OnSubmitSingle,
	OnSubmitCompare(BookEdit),
}


pub struct PopupSearch {
	cached_posters: Option<LoadingItem<api::WrappingResponse<api::ExternalSearchResponse>>>,
	input_value: String,

	left_edit: Option<(BookEdit, Source)>,
	right_edit: Option<(BookEdit, Source)>,

	selected_tab: String,

	waiting_item_resp: bool,
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

			waiting_item_resp: false,
		}
	}

	fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
		match msg {
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

			Msg::BookItemResponse(source, resp) => {
				if let Some(item) = resp.resp.and_then(|v| v.item) {
					if self.left_edit.is_none() {
						self.left_edit = Some((item.into(), source));
					} else {
						self.right_edit = Some((item.into(), source));
					}
				}

				self.waiting_item_resp = false;
			}

			Msg::OnSelectItem(source) => {
				if self.waiting_item_resp {
					return false;
				}

				self.waiting_item_resp = true;

				ctx.link().send_future(async move {
					Msg::BookItemResponse(source.clone(), request::get_external_source_item(source).await)
				});
			}

			Msg::OnSubmitSingle => {
				if let Some((_, source)) = self.left_edit.as_ref() {
					ctx.props().on_select.emit(Either::Left(source.clone()));
				}
			}

			Msg::OnSubmitCompare(book) => {
				ctx.props().on_select.emit(Either::Right(book));
			}

			Msg::OnChangeTab(name) => {
				self.selected_tab = name;
			}
		}

		true
	}

	fn view(&self, ctx: &Context<Self>) -> Html {
		if let Some(((left, _), (right, _))) = self.left_edit.clone().zip(self.right_edit.clone()) {
			self.render_compare(left, right, ctx)
		} else {
			self.render_main(ctx)
		}
	}
}

impl PopupSearch {
	fn render_main(&self, ctx: &Context<Self>) -> Html {
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
					if self.left_edit.is_some() {
						html! {
							<div>
								<button onclick={ ctx.link().callback(|_| Msg::OnSubmitSingle) }>{ "Insert (Single)" }</button>
								<button disabled={ true }>{ "Insert (Compared)" }</button>

								<span class="yellow">{ "Select another to be able to compare and insert" }</span>
							</div>
						}
					} else {
						html! {}
					}
				}
			</Popup>
		}
	}

	fn render_compare(&self, left_edit: BookEdit, right_edit: BookEdit, ctx: &Context<Self>) -> Html {
		html! {
			<PopupBookUpdateWithMeta
				{left_edit}
				{right_edit}
				show_equal_rows={ true }
				on_close={ ctx.props().on_close.clone() }
				on_submit={ ctx.link().callback(Msg::OnSubmitCompare) }
			/>
		}
	}

	fn render_poster_container(site: &str, item: &SearchItem, ctx: &Context<Self>) -> Html {
		let item = item.as_book();

		let source = item.source.clone();

		html! {
			<div
				class="book-search-item"
				onclick={ ctx.link().callback(move |_| Msg::OnSelectItem(source.clone())) }
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