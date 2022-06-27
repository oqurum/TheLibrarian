use std::fmt;

use librarian_common::item::edit::{BookEdit, NewOrCachedImage};
use yew::prelude::*;

use super::{Popup, PopupType};



#[derive(Properties, PartialEq)]
pub struct Property {
	#[prop_or_default]
    pub classes: Classes,

	pub on_close: Callback<()>,
	pub on_submit: Callback<BookEdit>,

	pub left_edit: BookEdit,
	pub right_edit: BookEdit,

	#[prop_or_default]
	pub show_equal_rows: bool,
}


pub enum Msg {
	Ignore,

	OnClose,
	OnSubmit,

	UpdateNew(UpdateValue, bool),
}


pub struct PopupBookUpdateWithMeta {
	edits: BookEdit,
}

impl Component for PopupBookUpdateWithMeta {
	type Message = Msg;
	type Properties = Property;

	fn create(_ctx: &Context<Self>) -> Self {
		Self {
			edits: BookEdit::default(),
		}
	}

	fn changed(&mut self, ctx: &Context<Self>) -> bool {
		let left_edit = &ctx.props().left_edit;

		self.edits = left_edit.clone();

		true
	}

	fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
		match msg {
			Msg::Ignore => return false,

			Msg::OnClose => ctx.props().on_close.emit(()),
			Msg::OnSubmit => {
				let left_edit = &ctx.props().left_edit;

				let edits = self.edits.clone();

				ctx.props().on_submit.emit(BookEdit {
					title: edits.title.or_else(|| left_edit.title.clone()),
					clean_title: edits.clean_title.or_else(|| left_edit.clean_title.clone()),
					description: edits.description.or_else(|| left_edit.description.clone()),
					rating: edits.rating.or(left_edit.rating),
					isbn_10: edits.isbn_10.or_else(|| left_edit.isbn_10.clone()),
					isbn_13: edits.isbn_13.or_else(|| left_edit.isbn_13.clone()),
					available_at: edits.available_at.or_else(|| left_edit.available_at.clone()),

					// TODO: Currently need these since BookEdit is sent to backend. If we don't have it it'll think it's unset.
					is_public: edits.is_public,
					language: edits.language,
					publisher: edits.publisher,

					// added_people: edits.added_people,
					// removed_people: edits.removed_people,
					// added_tags: edits.added_tags,
					// removed_tags: edits.removed_tags,
					added_images: edits.added_images.or_else(|| left_edit.added_images.clone()),
					// removed_images: edits.removed_images,

					.. BookEdit::default()
				})
			},

			Msg::UpdateNew(value, is_set) => {
				let left_edit = &ctx.props().left_edit;
				let right_edit = &ctx.props().right_edit;

				// TODO: Use then_some after stable.
				match value {
					UpdateValue::Title => self.edits.title = is_set.then(|| 0).map_or_else(|| left_edit.title.clone(), |_| right_edit.title.clone()),
					UpdateValue::Description => self.edits.description = is_set.then(|| 0).map_or_else(|| left_edit.description.clone(), |_| right_edit.description.clone()),
					UpdateValue::Rating => self.edits.rating = is_set.then(|| 0).map_or_else(|| left_edit.rating, |_| right_edit.rating),
					UpdateValue::Isbn10 => self.edits.isbn_10 = is_set.then(|| 0).map_or_else(|| left_edit.isbn_10.clone(), |_| right_edit.isbn_10.clone()),
					UpdateValue::Isbn13 => self.edits.isbn_13 = is_set.then(|| 0).map_or_else(|| left_edit.isbn_13.clone(), |_| right_edit.isbn_13.clone()),
					UpdateValue::AvailableAt => self.edits.available_at = is_set.then(|| 0).map_or_else(|| left_edit.available_at.clone(), |_| right_edit.available_at.clone()),

					UpdateValue::AddedImages => self.edits.added_images = is_set.then(|| 0).map_or_else(|| left_edit.added_images.clone(), |_| right_edit.added_images.clone()),
				}
			}
		}

		true
	}

	fn view(&self, ctx: &Context<Self>) -> Html {
		html! {
			<Popup
				type_of={ PopupType::FullOverlay }
				on_close={ ctx.props().on_close.clone() }
				classes={ classes!("popup-comparison-edit") }
			>
				<div class="header">
					<h2>{ "Book Update" }</h2>
				</div>

				{ self.render_body(ctx) }

				<div class="footer">
					<button class="button" onclick={ ctx.link().callback(|_| Msg::OnClose) }>{ "Cancel" }</button>
					<button class="button" onclick={ ctx.link().callback(|_| Msg::OnSubmit) }>{ "Save" }</button>
				</div>
			</Popup>
		}
	}
}

impl PopupBookUpdateWithMeta {
	fn render_body(&self, ctx: &Context<Self>) -> Html {
		let left_edit = &ctx.props().left_edit;
		let right_edit = &ctx.props().right_edit;

		html! {
			<div class="body">
				{ Self::display_value_row("Title", &left_edit.title, &right_edit.title, UpdateValue::Title, self.edits.title.is_none(), ctx) }
				{ Self::display_value_row("Description", &left_edit.description, &right_edit.description, UpdateValue::Description, self.edits.description.is_none(), ctx) }
				{ Self::display_value_row("Rating", &left_edit.rating, &right_edit.rating, UpdateValue::Rating, self.edits.rating.is_none(), ctx) }
				{ Self::display_value_row("ISBN 10", &left_edit.isbn_10, &right_edit.isbn_10, UpdateValue::Isbn10, self.edits.isbn_10.is_none(), ctx) }
				{ Self::display_value_row("ISBN 13", &left_edit.isbn_13, &right_edit.isbn_13, UpdateValue::Isbn13, self.edits.isbn_13.is_none(), ctx) }
				{ Self::display_value_row("Available At", &left_edit.available_at, &right_edit.available_at, UpdateValue::AvailableAt, self.edits.available_at.is_none(), ctx) }

				{ Self::display_image_row("Images", &left_edit.added_images, &right_edit.added_images, UpdateValue::AddedImages, self.edits.added_images.is_none(), ctx) }
			</div>
		}
	}

	fn display_value_row<V: Clone + Default + fmt::Display + PartialEq + fmt::Debug>(
		title: &'static str,
		current: &Option<V>,
		new: &Option<V>,
		updating: UpdateValue,
		is_old: bool,
		ctx: &Context<Self>,
	) -> Html {
		let old_selected = is_old.then(|| "selected");
		let new_selected = (!is_old).then(|| "selected");

		match (current, new) {
			(Some(old_value), Some(new_value)) if old_value != new_value || ctx.props().show_equal_rows => {
				html! {
					<div class="comparison-row">
						<div class="row-title"><span>{ title }</span></div>
						<div class={ classes!("row-grow", old_selected) } onclick={ ctx.link().callback(move |_| Msg::UpdateNew(updating, false)) }><div class="label">{ old_value.clone() }</div></div>
						<div class={ classes!("row-grow", new_selected) } onclick={ ctx.link().callback(move |_| Msg::UpdateNew(updating, true)) }><div class="label">{ new_value.clone() }</div></div>
					</div>
				}
			}

			(None, Some(new_value)) => {
				html! {
					<div class="comparison-row">
						<div class="row-title"><span>{ title }</span></div>
						<div class={ classes!("row-grow", old_selected) } onclick={ ctx.link().callback(move |_| Msg::UpdateNew(updating, false)) }><div class="label">{ "(Empty)" }</div></div>
						<div class={ classes!("row-grow", new_selected) } onclick={ ctx.link().callback(move |_| Msg::UpdateNew(updating, true)) }><div class="label">{ new_value.clone() }</div></div>
					</div>
				}
			}

			_ => html! {},
		}
	}

	fn display_image_row(
		title: &'static str,
		current: &Option<Vec<NewOrCachedImage>>,
		new: &Option<Vec<NewOrCachedImage>>,
		updating: UpdateValue,
		is_old: bool,
		ctx: &Context<Self>,
	) -> Html {
		let old_selected = is_old.then(|| "selected");
		let new_selected = (!is_old).then(|| "selected");

		match (current, new) {
			(Some(old_images), Some(new_images)) if old_images != new_images || ctx.props().show_equal_rows => {
				html! {
					<div class="comparison-row">
						<div class="row-title"><span>{ title }</span></div>
						<div class={ classes!("row-grow", old_selected) } onclick={ ctx.link().callback(move |_| Msg::UpdateNew(updating, false)) }>
							{
								for old_images.iter().map(|v| {
									html! {
										<div class="label">
											<div class="poster">
												<img src={ v.as_url().into_owned() } />
											</div>
										</div>
									}
								})
							}
						</div>
						<div class={ classes!("row-grow", new_selected) } onclick={ ctx.link().callback(move |_| Msg::UpdateNew(updating, true)) }>
						{
							for new_images.iter().map(|v| {
								html! {
									<div class="label">
										<div class="poster">
											<img src={ v.as_url().into_owned() } />
										</div>
									</div>
								}
							})
						}
						</div>
					</div>
				}
			}

			(None, Some(new_images)) => {
				html! {
					<div class="comparison-row">
						<div class="row-title"><span>{ title }</span></div>
						<div class={ classes!("row-grow", old_selected) } onclick={ ctx.link().callback(move |_| Msg::UpdateNew(updating, false)) }>
							<div class="label">{ "(Empty)" }</div>
						</div>
						<div class={ classes!("row-grow", new_selected) } onclick={ ctx.link().callback(move |_| Msg::UpdateNew(updating, true)) }>
						{
							for new_images.iter().map(|v| {
								html! {
									<div class="label">
										<div class="poster">
											<img src={ v.as_url().into_owned() } />
										</div>
									</div>
								}
							})
						}
						</div>
					</div>
				}
			}

			_ => html! {},
		}
	}
}


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UpdateValue {
	Title,
	Description,
	Rating,
	Isbn10,
	Isbn13,
	AvailableAt,

	AddedImages,
}