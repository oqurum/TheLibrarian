use js_sys::Date;
use librarian_common::{api::{MediaViewResponse, self, GetPostersResponse}, Either};
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{HtmlInputElement, HtmlTextAreaElement};
use yew::{prelude::*, html::Scope};

use crate::request;

#[derive(Clone)]
pub enum Msg {
	// Retrive
	RetrieveMediaView(Box<MediaViewResponse>),
	RetrievePosters(GetPostersResponse),

	UpdatedPoster,

	// Events
	ToggleEdit,
	SaveEdits,
	UpdateEditing(ChangingType, String),

	ShowPopup(DisplayOverlay),
	ClosePopup,

	Ignore
}

#[derive(Properties, PartialEq)]
pub struct Property {
	pub id: usize
}

pub struct MediaView {
	media: Option<MediaViewResponse>,
	cached_posters: Option<GetPostersResponse>,

	media_popup: Option<DisplayOverlay>,

	editing_item: Option<MediaViewResponse>,
}

impl Component for MediaView {
	type Message = Msg;
	type Properties = Property;

	fn create(_ctx: &Context<Self>) -> Self {
		Self {
			media: None,
			cached_posters: None,
			media_popup: None,
			editing_item: None,
		}
	}

	fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
		match msg {
			Msg::Ignore => return false,

			Msg::UpdatedPoster => {
				let meta_id = self.media.as_ref().unwrap().metadata.id;

				ctx.link()
				.send_future(async move {
					Msg::RetrievePosters(request::get_posters_for_meta(meta_id).await)
				});

				return false;
			}

			// Edits
			Msg::ToggleEdit => {
				if self.editing_item.is_none() {
					self.editing_item = self.media.clone();

					if self.cached_posters.is_none() {
						let metadata_id = self.media.as_ref().unwrap().metadata.id;

						ctx.link()
						.send_future(async move {
							Msg::RetrievePosters(request::get_posters_for_meta(metadata_id).await)
						});
					}
				} else {
					self.editing_item = None;
				}
			}

			Msg::SaveEdits => {
				self.media = self.editing_item.clone();

				let metadata = self.media.as_ref().unwrap().metadata.clone();
				let meta_id = metadata.id;

				ctx.link()
				.send_future(async move {
					request::update_book(meta_id, &api::UpdateBookBody {
						metadata: Some(metadata),
						people: None,
					}).await;

					Msg::Ignore
				});
			}

			Msg::UpdateEditing(type_of, value) => {
				let mut updating = self.editing_item.as_mut().unwrap();

				let value = Some(value).filter(|v| !v.is_empty());

				match type_of {
					ChangingType::Title => updating.metadata.title = value,
					ChangingType::OriginalTitle => updating.metadata.clean_title = value,
					ChangingType::Description => updating.metadata.description = value,
					ChangingType::Rating => updating.metadata.rating = value.and_then(|v| v.parse().ok()).unwrap_or_default(),
					ChangingType::ThumbPath => todo!(),
					ChangingType::AvailableAt => updating.metadata.available_at = value.map(|v| Date::new(&JsValue::from_str(&v)).get_time() as i64),
					ChangingType::Year => updating.metadata.year = value.map(|v| Date::new(&JsValue::from_str(&v)).get_time() as i64),
					ChangingType::Isbn10 => updating.metadata.isbn_10 = value,
					ChangingType::Isbn13 => updating.metadata.isbn_13 = value,
				}
			}

			// Popup
			Msg::ClosePopup => {
				self.media_popup = None;
			}

			Msg::ShowPopup(new_disp) => {
				if let Some(old_disp) = self.media_popup.as_mut() {
					if *old_disp == new_disp {
						self.media_popup = None;
					} else {
						self.media_popup = Some(new_disp);
					}
				} else {
					self.media_popup = Some(new_disp);
				}
			}


			Msg::RetrievePosters(value) => {
				self.cached_posters = Some(value);
			}

			Msg::RetrieveMediaView(value) => {
				self.media = Some(*value);
			}
		}

		true
	}

	fn view(&self, ctx: &Context<Self>) -> Html {
		let resp = self.editing_item.as_ref().or(self.media.as_ref());

		if let Some(MediaViewResponse { people, metadata }) = resp {
			let meta_id = metadata.id;

			html! {
				<div class="media-view-container">
					<div class="sidebar">
					{
						if self.is_editing() {
							html! {
								<>
									<div class="sidebar-item">
										<button class="button" onclick={ctx.link().callback(|_| Msg::ToggleEdit)}>{"Stop Editing"}</button>
									</div>
									<div class="sidebar-item">
										<button class="button proceed" onclick={ctx.link().callback(|_| Msg::SaveEdits)}>
											{"Save"}
										</button>
									</div>
								</>
							}
						} else {
							html! {
								<div class="sidebar-item">
									<button class="button" onclick={ctx.link().callback(|_| Msg::ToggleEdit)}>{"Start Editing"}</button>
								</div>
							}
						}
					}
					</div>

					<div class="main-content-view">
						<div class="info-container">
							<div class="poster large">
								<img src={ metadata.get_thumb_url() } />
							</div>
							<div class="metadata">
								{
									if self.is_editing() {
										html! {
											<>
												<h5>{ "Book Display Info" }</h5>

												<span class="sub-title">{"Title"}</span>
												<input class="title" type="text"
													onchange={Self::on_change_input(ctx.link(), ChangingType::Title)}
													value={ metadata.title.clone().unwrap_or_default() }
												/>

												<span class="sub-title">{"Original Title"}</span>
												<input class="title" type="text"
													onchange={Self::on_change_input(ctx.link(), ChangingType::OriginalTitle)}
													value={ metadata.clean_title.clone().unwrap_or_default() }
												/>

												<span class="sub-title">{"Description"}</span>
												<textarea
													rows="9"
													cols="30"
													class="description"
													onchange={Self::on_change_textarea(ctx.link(), ChangingType::Description)}
													value={ metadata.description.clone().unwrap_or_default() }
												/>
											</>
										}
									} else {
										html! {
											<>
												<h3 class="title">{ metadata.get_title() }</h3>
												<p class="description">{ metadata.description.clone().unwrap_or_default() }</p>
											</>
										}
									}
								}
							</div>
							{
								if self.is_editing() {
									html! {
										<div class="metadata">
											<h5>{ "Book Info" }</h5>

											<span class="sub-title">{"Year"}</span>
											<input class="title" type="text"
												placeholder="YYYY-MM-DD"
												onchange={Self::on_change_input(ctx.link(), ChangingType::Year)}
												value={ metadata.year.unwrap_or_default().to_string() }
											/>

											<span class="sub-title">{"Available At"}</span>
											<input class="title" type="text"
												placeholder="YYYY-MM-DD"
												onchange={Self::on_change_input(ctx.link(), ChangingType::AvailableAt)}
												value={ metadata.available_at.unwrap_or_default().to_string() }
											/>

											<span class="sub-title">{"ISBN 10"}</span>
											<input class="title" type="text"
												onchange={Self::on_change_input(ctx.link(), ChangingType::Isbn10)}
												value={ metadata.isbn_10.clone().unwrap_or_default() }
											/>

											<span class="sub-title">{"ISBN 13"}</span>
											<input class="title" type="text"
												onchange={Self::on_change_input(ctx.link(), ChangingType::Isbn13)}
												value={ metadata.isbn_13.clone().unwrap_or_default() }
											/>
										</div>
									}
								} else {
									html! {}
								}
							}
							{
								if self.is_editing() {
									html! {
										<div class="metadata">
											<h5>{ "Sources" }</h5>

											<span class="sub-title">{ "Good Reads URL" }</span>
											<input class="title" type="text" />

											<span class="sub-title">{ "Open Library URL" }</span>
											<input class="title" type="text" />
										</div>
									}
								} else {
									html! {}
								}
							}
						</div>

						{
							if self.is_editing() {
								if let Some(resp) = self.cached_posters.as_ref() {
									html! {
										<section>
											<h2>{ "Posters" }</h2>
											<div class="posters-container">
												<div class="add-poster" title="Add Poster">
													<span class="material-icons">{ "add" }</span>
												</div>
												{
													for resp.items.iter().map(move |poster| {
														let url_or_id = poster.id.map(Either::Right).unwrap_or_else(|| Either::Left(poster.path.clone()));
														let is_selected = poster.selected;

														html_nested! {
															<div
																class={ classes!("poster", { if is_selected { "selected" } else { "" } }) }
																onclick={ctx.link().callback_future(move |_| {
																	let url_or_id = url_or_id.clone();

																	async move {
																		if is_selected {
																			Msg::Ignore
																		} else {
																			request::change_poster_for_meta(meta_id, url_or_id).await;

																			Msg::UpdatedPoster
																		}
																	}
																})}
															>
																<img src={poster.path.clone()} />
															</div>
														}
													})
												}
											</div>
										</section>
									}
								} else {
									html! {}
								}
							} else {
								html! {}
							}
						}

						<section>
							<h2>{ "Characters" }</h2>
							<div class="characters-container">
								{
									if self.is_editing() {
										html! {
											<div class="add-person" title="Add Book Character">
												<span class="material-icons">{ "add" }</span>
											</div>
										}
									} else {
										html! {}
									}
								}
							</div>
						</section>

						<section>
							<h2>{ "People" }</h2>
							<div class="authors-container">
								{
									if self.is_editing() {
										html! {
											<div class="add-person" title="Add Person">
												<span class="material-icons">{ "add" }</span>
											</div>
										}
									} else {
										html! {}
									}
								}

								{
									for people.iter().map(|person| {
										html! {
											<div class="person-item">
												<div class="photo"><img src={ person.get_thumb_url() } /></div>
												<span class="title">{ person.name.clone() }</span>
											</div>
										}
									})
								}
							</div>
						</section>
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
			let metadata_id = ctx.props().id;

			ctx.link().send_future(async move {
				Msg::RetrieveMediaView(Box::new(request::get_media_view(metadata_id).await))
			});
		}
	}
}

impl MediaView {
	fn is_editing(&self) -> bool {
		self.editing_item.is_some()
	}

	fn on_change_input(scope: &Scope<Self>, updating: ChangingType) -> Callback<Event> {
		scope.callback(move |e: Event| {
			Msg::UpdateEditing(updating, e.target().unwrap().dyn_into::<HtmlInputElement>().unwrap().value())
		})
	}

	fn on_change_textarea(scope: &Scope<Self>, updating: ChangingType) -> Callback<Event> {
		scope.callback(move |e: Event| {
			Msg::UpdateEditing(updating, e.target().unwrap().dyn_into::<HtmlTextAreaElement>().unwrap().value())
		})
	}

	/// A Callback which calls "prevent_default" and "stop_propagation"
	fn on_click_prevdef_stopprop(scope: &Scope<Self>, msg: Msg) -> Callback<MouseEvent> {
		scope.callback(move |e: MouseEvent| {
			e.prevent_default();
			e.stop_propagation();
			msg.clone()
		})
	}

	/// A Callback which calls "prevent_default"
	fn on_click_prevdef(scope: &Scope<Self>, msg: Msg) -> Callback<MouseEvent> {
		scope.callback(move |e: MouseEvent| {
			e.prevent_default();
			msg.clone()
		})
	}
}

#[derive(Clone, Copy)]
pub enum ChangingType {
	Title,
	OriginalTitle,
	Description,
	Rating,
	ThumbPath,
	AvailableAt,
	Year,
	Isbn10,
	Isbn13,
}




#[derive(Clone)]
pub enum DisplayOverlay {
	Info {
		meta_id: usize
	},

	Edit(Box<api::MediaViewResponse>),

	More {
		meta_id: usize,
		mouse_pos: (i32, i32)
	},
}

impl PartialEq for DisplayOverlay {
	fn eq(&self, other: &Self) -> bool {
		match (self, other) {
			(Self::Info { meta_id: l_id }, Self::Info { meta_id: r_id }) => l_id == r_id,
			(Self::More { meta_id: l_id, .. }, Self::More { meta_id: r_id, .. }) => l_id == r_id,

			_ => false
		}
	}
}