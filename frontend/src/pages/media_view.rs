use js_sys::Date;
use librarian_common::{api::{MediaViewResponse, self, GetPostersResponse, GetTagsResponse}, Either, TagType, LANGUAGES, util::string_to_upper_case, BookId, TagId, item::edit::BookEdit, TagFE};
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{HtmlInputElement, HtmlTextAreaElement, HtmlSelectElement};
use yew::{prelude::*, html::Scope};

use crate::{components::{MultiselectModule, MultiselectItem, MultiselectNewItem, UploadModule}, request};



#[derive(Clone)]
pub enum Msg {
	// Retrive
	RetrieveMediaView(Box<MediaViewResponse>),
	RetrievePosters(GetPostersResponse),

	MultiselectToggle(bool, TagId),
	MultiselectCreate(TagType, MultiselectNewItem),
	MultiCreateResponse(TagFE),
	AllTagsResponse(GetTagsResponse),

	ReloadPosters,

	// Events
	ToggleEdit,
	SaveEdits,
	UpdateEditing(ChangingType, String),

	ShowPopup(DisplayOverlay),
	ClosePopup,

	Update,
	Ignore
}

#[derive(Properties, PartialEq)]
pub struct Property {
	pub id: BookId
}

pub struct MediaView {
	media: Option<MediaViewResponse>,
	cached_posters: Option<GetPostersResponse>,

	media_popup: Option<DisplayOverlay>,

	/// If we're currently editing. This'll be set.
	editing_item: BookEdit,
	is_editing: bool,

	// Multiselect Values
	cached_tags: Vec<CachedTag>,
}

impl Component for MediaView {
	type Message = Msg;
	type Properties = Property;

	fn create(ctx: &Context<Self>) -> Self {
		ctx.link().send_future(async {
			Msg::AllTagsResponse(request::get_tags().await)
		});

		Self {
			media: None,
			cached_posters: None,
			media_popup: None,
			editing_item: BookEdit::default(),
			is_editing: false,

			cached_tags: Vec::new(),
		}
	}

	fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
		match msg {
			Msg::Update => (),
			Msg::Ignore => return false,

			// Multiselect
			Msg::MultiselectToggle(inserted, tag_id) => if let Some(curr_book) = self.media.as_ref() {
				if inserted {
					// If the tag is in the db model.
					if curr_book.tags.iter().any(|bt| bt.tag.id == tag_id) {
						// We have to make sure it's on in the "removed_tags" vec
						self.editing_item.remove_tag(tag_id);
					} else {
						self.editing_item.insert_added_tag(tag_id);
					}
				} else {
					// If the tag is in the db model.
					if curr_book.tags.iter().any(|bt| bt.tag.id == tag_id) {
						self.editing_item.insert_removed_tag(tag_id);
					} else {
						// We have to make sure it's not in the "added_tags" vec
						self.editing_item.remove_tag(tag_id);
					}
				}
			}

			Msg::MultiselectCreate(type_of, item) => {
				match &type_of {
					TagType::Genre |
					TagType::Subject => {
						ctx.link()
						.send_future(async move {
							let tag_resp = request::new_tag(item.name.clone(), type_of).await;

							item.register.emit(*tag_resp.id);

							Msg::MultiCreateResponse(tag_resp)
						});
					}

					_ => unimplemented!("Msg::MultiselectCreate {:?}", type_of)
				}
			}

			Msg::MultiCreateResponse(tag) => {
				// Add original tag to cache.
				if !self.cached_tags.iter().any(|v| v.id == tag.id) {
					self.cached_tags.push(CachedTag {
						type_of: tag.type_of.clone(),
						name: tag.name.clone(),
						id: tag.id,
					});

					self.cached_tags.sort_unstable_by(|a, b| a.name.partial_cmp(&b.name).unwrap());
				}

				self.editing_item.insert_added_tag(tag.id);
			}

			Msg::AllTagsResponse(resp) => {
				self.cached_tags = resp.items.into_iter()
					.map(|v| CachedTag {
						id: v.id,
						type_of: v.type_of,
						name: v.name
					})
					.collect();

				self.cached_tags.sort_unstable_by(|a, b| a.name.partial_cmp(&b.name).unwrap());
			}


			Msg::ReloadPosters => {
				let meta_id = self.media.as_ref().unwrap().metadata.id;

				ctx.link()
				.send_future(async move {
					Msg::RetrievePosters(request::get_posters_for_meta(meta_id).await)
				});

				return false;
			}

			// Edits
			Msg::ToggleEdit => {
				// Is currently editing? We won't be.
				if self.is_editing {
					self.editing_item = BookEdit::default();
				} else if self.cached_posters.is_none() {
					ctx.link().send_message(Msg::ReloadPosters);
				}

				self.is_editing = !self.is_editing;
			}

			Msg::SaveEdits => {
				let edit = self.editing_item.clone();
				self.editing_item = BookEdit::default();

				let book_id = self.media.as_ref().unwrap().metadata.id;

				ctx.link()
				.send_future(async move {
					request::update_book(book_id, &edit).await;

					Msg::RetrieveMediaView(Box::new(request::get_media_view(book_id).await))
				});
			}

			Msg::UpdateEditing(type_of, value) => {
				let mut updating = &mut self.editing_item;

				let value = Some(value).filter(|v| !v.is_empty());

				match type_of {
					ChangingType::Title => updating.title = value,
					ChangingType::OriginalTitle => updating.clean_title = value,
					ChangingType::Description => updating.description = value,
					ChangingType::Rating => updating.rating = value.and_then(|v| v.parse().ok()),
					ChangingType::ThumbPath => todo!(),
					ChangingType::AvailableAt => updating.available_at = value.map(|v| {
						let date = Date::new(&JsValue::from_str(&v));
						format!("{}-{}-{}", date.get_full_year(), date.get_month() + 1, date.get_date())
					}),
					ChangingType::Language => updating.language = value.and_then(|v| v.parse().ok()),
					ChangingType::Isbn10 => updating.isbn_10 = value,
					ChangingType::Isbn13 => updating.isbn_13 = value,
					ChangingType::Publicity => updating.is_public = value.and_then(|v| v.parse().ok()),
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
		let editing = &self.editing_item;

		if let Some(MediaViewResponse { people, metadata: book_model, tags }) = self.media.as_ref() {
			let meta_id = book_model.id;

			html! {
				<div class="media-view-container">
					<div class="sidebar">
					{
						if self.is_editing {
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
								<img src={ book_model.get_thumb_url() } />
							</div>

							<div class="metadata">
								{ // Book Display Info
									if self.is_editing {
										html! {
											<>
												<h5>{ "Book Display Info" }</h5>

												<span class="sub-title">{"Publicity"}</span>
												<select
													class="title"
													type="text"
													onchange={Self::on_change_select(ctx.link(), ChangingType::Publicity)}
												>
													<option selected={editing.is_public.unwrap_or(book_model.is_public)} value="true">
														{"Public"}
													</option>
													<option selected={!editing.is_public.unwrap_or(book_model.is_public)} value="false">
														{"Private"}
													</option>
												</select>

												<span class="sub-title">{"Title"}</span>
												<input class="title" type="text"
													onchange={Self::on_change_input(ctx.link(), ChangingType::Title)}
													value={ editing.title.clone().or_else(|| book_model.title.clone()).unwrap_or_default() }
												/>

												<span class="sub-title">{"Original Title"}</span>
												<input class="title" type="text"
													onchange={Self::on_change_input(ctx.link(), ChangingType::OriginalTitle)}
													value={ editing.clean_title.clone().or_else(|| book_model.clean_title.clone()).unwrap_or_default() }
												/>

												<span class="sub-title">{"Description"}</span>
												<textarea
													rows="9"
													cols="30"
													class="description"
													onchange={Self::on_change_textarea(ctx.link(), ChangingType::Description)}
													value={ editing.description.clone().or_else(|| book_model.description.clone()).unwrap_or_default() }
												/>
											</>
										}
									} else {
										html! {
											<>
												<h3 class="title">{ book_model.get_title() }</h3>
												<p class="description">{ book_model.description.clone().unwrap_or_default() }</p>
											</>
										}
									}
								}
							</div>

							{ // Book Info
								if self.is_editing {
									html! {
										<div class="metadata">
											<h5>{ "Book Info" }</h5>

											<span class="sub-title">{"Available At"}</span>
											<input class="title" type="text"
												placeholder="YYYY-MM-DD"
												onchange={Self::on_change_input(ctx.link(), ChangingType::AvailableAt)}
												value={ editing.available_at.clone().or_else(|| book_model.available_at.clone()).unwrap_or_default() }
											/>

											<span class="sub-title">{"ISBN 10"}</span>
											<input class="title" type="text"
												onchange={Self::on_change_input(ctx.link(), ChangingType::Isbn10)}
												value={ editing.isbn_10.clone().or_else(|| book_model.isbn_10.clone()).unwrap_or_default() }
											/>

											<span class="sub-title">{"ISBN 13"}</span>
											<input class="title" type="text"
												onchange={Self::on_change_input(ctx.link(), ChangingType::Isbn13)}
												value={ editing.isbn_13.clone().or_else(|| book_model.isbn_13.clone()).unwrap_or_default() }
											/>

											<span class="sub-title">{"Publisher"}</span>
											<input class="title" type="text" />

											<span class="sub-title">{"Language"}</span>
											<select
												class="title"
												type="text"
												onchange={Self::on_change_select(ctx.link(), ChangingType::Language)}
											>
												<option value="-1" selected={editing.language.or(book_model.language).is_none()}>{ "Unknown" }</option>
												{
													for LANGUAGES.iter()
														.enumerate()
														.map(|(index, lang)| {
															let selected = editing.language.or(book_model.language).filter(|v| index as u16 == *v).is_some();

															html! {
																<option
																	{selected}
																	value={index.to_string()}
																>
																	{ string_to_upper_case(lang.to_string()) }
																</option>
															}
														})
												}
											</select>
										</div>
									}
								} else {
									html! {}
								}
							}

							{ // Sources
								if self.is_editing {
									html! {
										<div class="metadata">
											<h5>{ "Sources" }</h5>

											<span class="sub-title">{ "Good Reads URL" }</span>
											<input class="title" type="text" />

											<span class="sub-title">{ "Open Library URL" }</span>
											<input class="title" type="text" />

											<span class="sub-title">{ "Google Books URL" }</span>
											<input class="title" type="text" />

											<h5>{ "Tags" }</h5>

											<span class="sub-title">{ "Genre" }</span>
											<MultiselectModule
												on_create_item={ctx.link().callback(|v| Msg::MultiselectCreate(TagType::Genre, v))}
												on_toggle_item={ctx.link().callback(|(a, b)| Msg::MultiselectToggle(a, TagId::from(b)))}
											>
												{
													for self.cached_tags
														.iter()
														.filter(|v| v.type_of.into_u8() == TagType::Genre.into_u8())
														.map(|tag| {
															let mut filtered_tags = tags.iter()
																// We only need the tag ids
																.map(|bt| bt.tag.id)
																// Filter out editing "removed tags"
																.filter(|tag_id| !editing.removed_tags.as_ref().map(|v| v.iter().any(|r| r == tag_id)).unwrap_or_default())
																// Chain into editing "added tags"
																.chain(editing.added_tags.iter().flat_map(|v| v.iter()).copied());

															html_nested! {
																// TODO: Remove deref
																<MultiselectItem name={tag.name.clone()} id={*tag.id} selected={filtered_tags.any(|tag_id| tag_id == tag.id)} />
															}
														})
												}
											</MultiselectModule>

											<span class="sub-title">{ "Subject" }</span>
											<MultiselectModule
												on_create_item={ctx.link().callback(|v| Msg::MultiselectCreate(TagType::Subject, v))}
												on_toggle_item={ctx.link().callback(|(a, b)| Msg::MultiselectToggle(a, TagId::from(b)))}
											>
												{
													for self.cached_tags
														.iter()
														.filter(|v| v.type_of.into_u8() == TagType::Subject.into_u8())
														.map(|tag| {
															let mut filtered_tags = tags.iter()
																// We only need the tag ids
																.map(|bt| bt.tag.id)
																// Filter out editing "removed tags"
																.filter(|tag_id| !editing.removed_tags.as_ref().map(|v| v.iter().any(|r| r == tag_id)).unwrap_or_default())
																// Chain into editing "added tags"
																.chain(editing.added_tags.iter().flat_map(|v| v.iter()).copied());

															html_nested! {
																// TODO: Remove deref
																<MultiselectItem name={tag.name.clone()} id={*tag.id} selected={filtered_tags.any(|tag_id| tag_id == tag.id)} />
															}
														})
												}
											</MultiselectModule>
										</div>
									}
								} else {
									html! {}
								}
							}
						</div>

						{ // Posters
							if self.is_editing {
								if let Some(resp) = self.cached_posters.as_ref() {
									html! {
										<section>
											<h2>{ "Posters" }</h2>
											<div class="posters-container">
												<UploadModule
													id={Either::Left(BookId::from(ctx.props().id))}
													class="add-poster"
													title="Add Poster"
													on_upload={ctx.link().callback(|_| Msg::ReloadPosters)}
												>
													<span class="material-icons">{ "add" }</span>
												</UploadModule>

												{
													for resp.items.iter().map(move |poster| {
														let url_or_id = poster.id.map(Either::Right).unwrap_or_else(|| Either::Left(poster.path.clone()));
														let is_selected = poster.selected;

														html! {
															<div
																class={ classes!("poster", { if is_selected { "selected" } else { "" } }) }
																onclick={ctx.link().callback_future(move |_| {
																	let url_or_id = url_or_id.clone();

																	async move {
																		if is_selected {
																			Msg::Ignore
																		} else {
																			request::change_poster_for_meta(meta_id, url_or_id).await;

																			Msg::ReloadPosters
																		}
																	}
																})}
															>
																<div class="top-right">
																	<span
																		class="material-icons"
																	>{ "delete" }</span>
																</div>
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
									if self.is_editing {
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
									if self.is_editing {
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
	fn on_change_select(scope: &Scope<Self>, updating: ChangingType) -> Callback<Event> {
		scope.callback(move |e: Event| {
			Msg::UpdateEditing(updating, e.target().unwrap().dyn_into::<HtmlSelectElement>().unwrap().value())
		})
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


#[derive(Debug, Clone)]
pub struct CachedTag {
	type_of: TagType,
	id: TagId,
	name: String,
}



#[derive(Clone, Copy)]
pub enum ChangingType {
	Title,
	OriginalTitle,
	Description,
	Rating,
	ThumbPath,
	AvailableAt,
	Language,
	Isbn10,
	Isbn13,
	Publicity,
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