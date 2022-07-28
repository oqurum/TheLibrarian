use common::{
	component::{
		upload::UploadModule,
		multi_select::{MultiSelectEvent, MultiSelectModule, MultiSelectItem, MultiSelectNewItem},
		popup::{Popup, PopupType, compare::{PopupComparison, Comparable}},
	},
	Either, LANGUAGES, ImageIdType, BookId, TagId
};
use librarian_common::{api::{MediaViewResponse, GetPostersResponse, GetTagsResponse, self}, TagType, util::string_to_upper_case, item::edit::BookEdit, TagFE, SearchType};

use js_sys::Date;
use wasm_bindgen::{JsCast, JsValue, UnwrapThrowExt};
use web_sys::{HtmlInputElement, HtmlTextAreaElement, HtmlSelectElement};
use yew::{prelude::*, html::Scope};

use crate::{
	components::{LoginBarrier, PopupEditMetadata, PopupSearch},
	request
};



#[derive(Clone)]
pub enum Msg {
	// Retrive
	RetrieveMediaView(Box<api::WrappingResponse<MediaViewResponse>>),
	RetrievePosters(api::WrappingResponse<GetPostersResponse>),

	MultiselectToggle(bool, TagId),
	MultiselectCreate(TagType, MultiSelectNewItem<TagId>),
	MultiCreateResponse(TagFE),
	AllTagsResponse(api::WrappingResponse<GetTagsResponse>),

	ReloadPosters,

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
	pub id: BookId
}

pub struct BookView {
	media: Option<api::WrappingResponse<MediaViewResponse>>,
	cached_posters: Option<api::WrappingResponse<GetPostersResponse>>,

	media_popup: Option<DisplayOverlay>,

	/// If we're currently editing. This'll be set.
	editing_item: BookEdit,
	is_editing: bool,

	// Multiselect Values
	cached_tags: Vec<CachedTag>,
}

impl Component for BookView {
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
			Msg::Ignore => return false,

			// Multiselect
			Msg::MultiselectToggle(inserted, tag_id) => if let Some(curr_book) = self.media.as_ref().and_then(|v| v.resp.as_ref()) {
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

							match tag_resp.ok() {
								Ok(tag_resp) => {
									item.register.emit(tag_resp.id);

									Msg::MultiCreateResponse(tag_resp)
								}

								Err(err) => {
									log::error!("{err}");

									Msg::Ignore
								}
							}
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
				let resp = resp.ok().unwrap_throw();

				self.cached_tags = resp.items.into_iter()
					.map(|v| CachedTag {
						id: v.id,
						type_of: v.type_of,
						name: v.name
					})
					.collect();

				self.cached_tags.sort_unstable_by(|a, b| a.name.partial_cmp(&b.name).unwrap());
			}


			Msg::ReloadPosters => if let Some(curr_book) = self.media.as_ref().and_then(|v| v.resp.as_ref()) {
				let book_id = curr_book.metadata.id;

				ctx.link()
				.send_future(async move {
					Msg::RetrievePosters(request::get_posters_for_meta(ImageIdType::new_book(book_id)).await)
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

			Msg::SaveEdits => if let Some(curr_book) = self.media.as_ref().and_then(|v| v.resp.as_ref()) {
				let edit = self.editing_item.clone();
				self.editing_item = BookEdit::default();

				let book_id = curr_book.metadata.id;

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
					// ChangingType::Rating => updating.rating = value.and_then(|v| v.parse().ok()),
					// ChangingType::ThumbPath => todo!(),
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

		if let Some(resp) = self.media.as_ref() {
			let book_resp @ MediaViewResponse { people, metadata: book_model, tags } = crate::continue_or_html_err!(resp);

			let book_id = book_model.id;

			let on_click_more = ctx.link().callback(move |e: MouseEvent| {
				e.prevent_default();
				e.stop_propagation();

				Msg::ShowPopup(DisplayOverlay::More { book_id, mouse_pos: (e.page_x(), e.page_y()) })
			});

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
								<LoginBarrier>
									<div class="sidebar-item">
										<button class="button" onclick={ctx.link().callback(|_| Msg::ToggleEdit)}>{"Start Editing"}</button>
									</div>
								</LoginBarrier>
							}
						}
					}
					</div>

					<div class="main-content-view">
						<div class="info-container">
							<div class="poster large">
								<LoginBarrier>
									<div class="bottom-right">
										<span class="material-icons" onclick={on_click_more} title="More Options">{ "more_horiz" }</span>
									</div>

									<div class="bottom-left">
										<span class="material-icons" onclick={ctx.link().callback_future(move |e: MouseEvent| {
											e.prevent_default();
											e.stop_propagation();

											async move {
												Msg::ShowPopup(DisplayOverlay::Edit(Box::new(request::get_media_view(book_id).await)))
											}
										})} title="More Options">{ "edit" }</span>
									</div>
								</LoginBarrier>

								<img src={ book_model.get_thumb_url() } />
							</div>

							<div class="metadata-container">
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
												<MultiSelectModule<TagId>
													editing=true
													on_event={
														ctx.link().callback(|v| match v {
															MultiSelectEvent::Toggle { toggle, id } => {
																Msg::MultiselectToggle(toggle, id)
															}

															MultiSelectEvent::Create(new_item) => {
																Msg::MultiselectCreate(TagType::Genre, new_item)
															}
														})
													}
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
																	<MultiSelectItem<TagId> name={tag.name.clone()} id={tag.id} selected={filtered_tags.any(|tag_id| tag_id == tag.id)} />
																}
															})
													}
												</MultiSelectModule<TagId>>

												<span class="sub-title">{ "Subject" }</span>

												<MultiSelectModule<TagId>
													editing=true
													on_event={
														ctx.link().callback(|v| match v {
															MultiSelectEvent::Toggle { toggle, id } => {
																Msg::MultiselectToggle(toggle, id)
															}

															MultiSelectEvent::Create(new_item) => {
																Msg::MultiselectCreate(TagType::Subject, new_item)
															}
														})
													}
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
																	<MultiSelectItem<TagId> name={tag.name.clone()} id={tag.id} selected={filtered_tags.any(|tag_id| tag_id == tag.id)} />
																}
															})
													}
												</MultiSelectModule<TagId>>
											</div>
										}
									} else {
										html! {}
									}
								}
							</div>
						</div>

						{ // Posters
							if self.is_editing {
								if let Some(resp) = self.cached_posters.as_ref() {
									match resp.as_ok() {
										Ok(resp) => html! {
											<section>
												<h2>{ "Posters" }</h2>
												<div class="posters-container">
													<UploadModule
														class="add-poster"
														title="Add Poster"
														upload_url={ format!("/api/v1/posters/{}/upload", ImageIdType::new_book(ctx.props().id)) }
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
																				request::change_poster_for_meta(ImageIdType::new_book(book_id), url_or_id).await;

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
										},

										Err(e) => html! {
											<h2>{ e }</h2>
										}
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

					{
						if let Some(overlay_type) = self.media_popup.as_ref() {
							match overlay_type {
								&DisplayOverlay::More { mouse_pos, .. } => {
									html! {
										<Popup type_of={ PopupType::AtPoint(mouse_pos.0, mouse_pos.1) } on_close={ctx.link().callback(|_| Msg::ClosePopup)}>
											<div class="menu-list">
												// <div class="menu-item" yew-close-popup="" onclick={
												// 	Self::on_click_prevdef(ctx.link(), Msg::UpdateBook(book_id))
												// }>{ "Refresh Metadata" }</div>
												<div class="menu-item" yew-close-popup="" onclick={
													Self::on_click_prevdef_stopprop(ctx.link(), Msg::ShowPopup(DisplayOverlay::SearchForBook { input_value: None }))
												}>{ "Search New Metadata" }</div>
												<div class="menu-item" yew-close-popup="">{ "Delete" }</div>
											</div>
										</Popup>
									}
								}

								DisplayOverlay::Edit(resp) => {
									match resp.as_ok() {
										Ok(resp) => html! {
											<PopupEditMetadata
												on_close={ ctx.link().callback(|_| Msg::ClosePopup) }
												classes={ classes!("popup-book-edit") }
												media_resp={ resp.clone() }
											/>
										},

										Err(e) => html! {
											<h2>{ e }</h2>
										}
									}
								}

								DisplayOverlay::EditFromMetadata(new_meta) => {
									html! {
										<PopupComparison
											on_close={ ctx.link().callback(|_| Msg::ClosePopup) }
											on_submit={ ctx.link().callback_future(move |v| async move {
												request::update_book(book_id, &BookEdit::create_from_comparison(v).unwrap_throw()).await;
												Msg::Ignore
											}) }
											classes={ classes!("popup-book-edit") }
											compare={ BookEdit::from(book_resp.metadata.clone()).create_comparison_with(&**new_meta).unwrap_throw() }
										/>
									}
								}

								DisplayOverlay::SearchForBook { input_value } => {
									let input_value = if let Some(v) = input_value {
										v.to_string()
									} else {
										format!(
											"{} {}",
											book_model.title.as_deref().unwrap_or_default(),
											book_model.cached.author.as_deref().unwrap_or_default()
										)
									};

									let input_value = input_value.trim().to_string();

									html! {
										<PopupSearch
											{input_value}
											search_for={ SearchType::Book }
											on_close={ ctx.link().callback(|_| Msg::ClosePopup) }
											on_select={ ctx.link().callback_future(|value| async {
												Msg::ShowPopup(DisplayOverlay::EditFromMetadata(
													match value {
														Either::Left(source) => {
															let resp = request::get_external_source_item(source).await.ok().unwrap_throw();

															Box::new(resp.item.unwrap().into())
														}

														Either::Right(book) => Box::new(book),
													}
												))
											}) }
										/>
									}
								}
							}
						} else {
							html! {}
						}
					}
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

impl BookView {
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
	// Rating,
	// ThumbPath,
	AvailableAt,
	Language,
	Isbn10,
	Isbn13,
	Publicity,
}




#[derive(Clone)]
pub enum DisplayOverlay {
	Edit(Box<api::WrappingResponse<MediaViewResponse>>),

	EditFromMetadata(Box<BookEdit>),

	More {
		book_id: BookId,
		mouse_pos: (i32, i32)
	},

	SearchForBook {
		input_value: Option<String>,
	},
}

impl PartialEq for DisplayOverlay {
	fn eq(&self, other: &Self) -> bool {
		match (self, other) {
			(Self::More { book_id: l_id, .. }, Self::More { book_id: r_id, .. }) => l_id == r_id,
			(
				Self::SearchForBook { input_value: l_val, .. },
				Self::SearchForBook { input_value: r_val, .. }
			) => l_val == r_val,

			_ => false
		}
	}
}