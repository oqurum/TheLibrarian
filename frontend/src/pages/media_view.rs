use librarian_common::api::{MediaViewResponse, self};
use yew::{prelude::*, html::Scope};

use crate::{request, components::{Popup, PopupType, PopupEditMetadata}};

#[derive(Clone)]
pub enum Msg {
	// Retrive
	RetrieveMediaView(Box<MediaViewResponse>),

	// Events
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
}

impl Component for MediaView {
	type Message = Msg;
	type Properties = Property;

	fn create(ctx: &Context<Self>) -> Self {
		Self {
			media: None,
			media_popup: None,
		}
	}

	fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
		match msg {
			Msg::Ignore => return false,

			Msg::ClosePopup => {
				self.media_popup = None;
			}

			Msg::RetrieveMediaView(value) => {
				self.media = Some(*value);
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
		if let Some(MediaViewResponse { people, metadata, media, progress }) = self.media.as_ref() {
			let meta_id = metadata.id;
			let on_click_more = ctx.link().callback(move |e: MouseEvent| {
				e.prevent_default();
				e.stop_propagation();

				Msg::ShowPopup(DisplayOverlay::More { meta_id, mouse_pos: (e.page_x(), e.page_y()) })
			});

			let media_prog = media.iter().zip(progress.iter());

			html! {
				<div class="media-view-container">
					<div class="info-container">
						<div class="poster large">
							<div class="bottom-right">
								<span class="material-icons" onclick={on_click_more} title="More Options">{ "more_horiz" }</span>
							</div>
							<div class="bottom-left">
								<span class="material-icons" onclick={ctx.link().callback_future(move |e: MouseEvent| {
									e.prevent_default();
									e.stop_propagation();

									async move {
										Msg::ShowPopup(DisplayOverlay::Edit(Box::new(request::get_media_view(meta_id).await)))
									}
								})} title="More Options">{ "edit" }</span>
							</div>

							<img src={ metadata.get_thumb_url() } />
						</div>
						<div class="metadata">
							<h3 class="title">{ metadata.get_title() }</h3>
							<p class="description">{ metadata.description.clone().unwrap_or_default() }</p>
						</div>
					</div>

					<section>
						<h2>{ "Characters" }</h2>
						<div class="characters-container">
							<div class="person-item">
								<div class="photo"><img src="/images/missingperson.jpg" /></div>
								<span class="title">{ "Character #1" }</span>
							</div>
							<div class="person-item">
								<div class="photo"><img src="/images/missingperson.jpg" /></div>
								<span class="title">{ "Character #2" }</span>
							</div>
						</div>
					</section>

					<section>
						<h2>{ "People" }</h2>
						<div class="authors-container">
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

					{
						if let Some(overlay_type) = self.media_popup.as_ref() {
							match overlay_type {
								DisplayOverlay::Info { meta_id: _ } => {
									html! {
										<Popup type_of={ PopupType::FullOverlay } on_close={ctx.link().callback(|_| Msg::ClosePopup)}>
											<h1>{"Info"}</h1>
										</Popup>
									}
								}

								&DisplayOverlay::More { meta_id, mouse_pos } => {
									html! {
										<Popup type_of={ PopupType::AtPoint(mouse_pos.0, mouse_pos.1) } on_close={ctx.link().callback(|_| Msg::ClosePopup)}>
											<div class="menu-list">
												<div class="menu-item" yew-close-popup="">{ "Start Reading" }</div>
												<div class="menu-item" yew-close-popup="" onclick={
													Self::on_click_prevdef(ctx.link(), Msg::UpdateMeta(meta_id))
												}>{ "Refresh Metadata" }</div>
												<div class="menu-item" yew-close-popup="">{ "Delete" }</div>
												<div class="menu-item" yew-close-popup="" onclick={
													Self::on_click_prevdef_stopprop(ctx.link(), Msg::ShowPopup(DisplayOverlay::Info { meta_id }))
												}>{ "Show Info" }</div>
											</div>
										</Popup>
									}
								}

								DisplayOverlay::Edit(resp) => {
									html! {
										<PopupEditMetadata
											on_close={ ctx.link().callback(|_| Msg::ClosePopup) }
											classes={ classes!("popup-book-edit") }
											media_resp={ (&**resp).clone() }
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

impl MediaView {
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