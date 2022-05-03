use librarian_common::api::{MediaViewResponse, self};
use wasm_bindgen::JsCast;
use web_sys::{HtmlInputElement, HtmlTextAreaElement};
use yew::{prelude::*, html::Scope};

use crate::request;

#[derive(Clone)]
pub enum Msg {
	// Retrive
	RetrieveMediaView(Box<MediaViewResponse>),

	// Events
	ToggleEdit,
	SaveEdits,
	UpdateEditing(ChangingType, String),

	ShowPopup(DisplayOverlay),
	ClosePopup,

	// Popup Events
	UpdateMeta(usize),

	Ignore
}

#[derive(Properties, PartialEq)]
pub struct Property {
	pub id: usize
}

pub struct MediaView {
	media: Option<MediaViewResponse>,

	media_popup: Option<DisplayOverlay>,

	editing_item: Option<MediaViewResponse>,
}

impl Component for MediaView {
	type Message = Msg;
	type Properties = Property;

	fn create(_ctx: &Context<Self>) -> Self {
		Self {
			media: None,
			media_popup: None,
			editing_item: None,
		}
	}

	fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
		match msg {
			Msg::Ignore => return false,

			// Edits
			Msg::ToggleEdit => {
				if self.editing_item.is_none() {
					self.editing_item = self.media.clone();
				} else {
					self.editing_item = None;
				}
			}

			Msg::SaveEdits => {
				self.media = self.editing_item.clone();
			}

			Msg::UpdateEditing(type_of, value) => {
				let mut updating = self.editing_item.as_mut().unwrap();

				match type_of {
					ChangingType::Title => updating.metadata.title = Some(value).filter(|v| !v.is_empty()),
					ChangingType::OriginalTitle => updating.metadata.original_title = Some(value).filter(|v| !v.is_empty()),
					ChangingType::Description => updating.metadata.description = Some(value).filter(|v| !v.is_empty()),
					ChangingType::Rating => updating.metadata.rating = Some(value).and_then(|v| v.parse().ok()).unwrap_or_default(),
					ChangingType::ThumbPath => todo!(),
					ChangingType::AvailableAt => updating.metadata.available_at = Some(value).and_then(|v| v.parse().ok()),
					ChangingType::Year => updating.metadata.year = Some(value).and_then(|v| v.parse().ok()),
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


			Msg::RetrieveMediaView(value) => {
				self.media = Some(*value);
			}

			Msg::UpdateMeta(meta_id) => {
				ctx.link()
				.send_future(async move {
					request::update_metadata(meta_id, &api::PostMetadataBody::AutoMatchMetaIdBySource).await;

					Msg::Ignore
				});
			}
		}

		true
	}

	fn view(&self, ctx: &Context<Self>) -> Html {
		let resp = self.editing_item.as_ref().or(self.media.as_ref());

		if let Some(MediaViewResponse { people, metadata }) = resp {
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
												<span class="sub-title">{"Title"}</span>
												<input class="title" type="text"
													onchange={Self::on_change_input(ctx.link(), ChangingType::Title)}
													value={ metadata.title.clone().unwrap_or_default() } />

												<span class="sub-title">{"Original Title"}</span>
												<input class="title" type="text"
													onchange={Self::on_change_input(ctx.link(), ChangingType::OriginalTitle)}
													value={ metadata.original_title.clone().unwrap_or_default() } />

												<span class="sub-title">{"Description"}</span>
												<textarea
													rows="10"
													cols="33"
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
											<span class="sub-title">{"Year"}</span>
											<input class="title" type="text"
												onchange={Self::on_change_input(ctx.link(), ChangingType::Year)}
												value={ metadata.year.unwrap_or_default().to_string() } />

											<span class="sub-title">{"Available At"}</span>
											<input class="title" type="text"
												onchange={Self::on_change_input(ctx.link(), ChangingType::AvailableAt)}
												value={ metadata.available_at.unwrap_or_default().to_string() } />
										</div>
									}
								} else {
									html! {}
								}
							}
						</div>

						<section>
							<h2>{ "Characters" }</h2>
							<div class="characters-container">
								{
									if self.is_editing() {
										html! {
											<div class="add-person">
												<span class="material-icons" title="Add Person">{ "add" }</span>
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
											<div class="add-person">
												<span class="material-icons" title="Add Person">{ "add" }</span>
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