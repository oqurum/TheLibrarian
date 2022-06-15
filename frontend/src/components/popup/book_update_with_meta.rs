use std::fmt;

use librarian_common::{api::{MediaViewResponse, MetadataBookItem}, item::edit::BookEdit};
use yew::{prelude::*, html::Scope};

use super::{Popup, PopupType};



#[derive(Properties, PartialEq)]
pub struct Property {
	#[prop_or_default]
    pub classes: Classes,

	pub on_close: Callback<()>,
	pub on_submit: Callback<BookEdit>,

	pub book_resp: MediaViewResponse,
	pub metadata: MetadataBookItem,
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
		let MediaViewResponse { metadata, .. } = &ctx.props().book_resp;

		self.edits.title = metadata.title.clone();
		self.edits.description = metadata.description.clone();
		self.edits.isbn_10 = metadata.isbn_10.clone();
		self.edits.isbn_13 = metadata.isbn_13.clone();
		self.edits.available_at = metadata.available_at.clone();

		true
	}

	fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
		match msg {
			Msg::Ignore => return false,

			Msg::OnClose => ctx.props().on_close.emit(()),
			Msg::OnSubmit => ctx.props().on_submit.emit(self.edits.clone()),

			Msg::UpdateNew(value, is_set) => {
				let MediaViewResponse { metadata: old_meta, .. } = &ctx.props().book_resp;
				let new_meta = &ctx.props().metadata;

				// TODO: Use then_some after stable.
				match value {
					UpdateValue::Title => self.edits.title = is_set.then(|| 0).map_or_else(|| old_meta.title.clone(), |_| new_meta.title.clone()),
					UpdateValue::Description => self.edits.description = is_set.then(|| 0).map_or_else(|| old_meta.description.clone(), |_| new_meta.description.clone()),
					UpdateValue::Isbn10 => self.edits.isbn_10 = is_set.then(|| 0).map_or_else(|| old_meta.isbn_10.clone(), |_| new_meta.isbn_10.clone()),
					UpdateValue::Isbn13 => self.edits.isbn_13 = is_set.then(|| 0).map_or_else(|| old_meta.isbn_13.clone(), |_| new_meta.isbn_13.clone()),
					UpdateValue::AvailableAt => self.edits.available_at = is_set.then(|| 0).map_or_else(|| old_meta.available_at.clone(), |_| new_meta.available_at.clone()),
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
				classes={ classes!("popup-book-edit") }
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
		let MediaViewResponse { metadata, .. } = &ctx.props().book_resp;
		let new_meta = &ctx.props().metadata;

		html! {
			<div class="body">
				{ Self::display_row("Title", &metadata.title, &new_meta.title, UpdateValue::Title, ctx.link()) }
				{ Self::display_row("Description", &metadata.description, &new_meta.description, UpdateValue::Description, ctx.link()) }
				{ Self::display_row("ISBN 10", &metadata.isbn_10, &new_meta.isbn_10, UpdateValue::Isbn10, ctx.link()) }
				{ Self::display_row("ISBN 13", &metadata.isbn_13, &new_meta.isbn_13, UpdateValue::Isbn13, ctx.link()) }
				{ Self::display_row("Available At", &metadata.available_at, &new_meta.available_at, UpdateValue::AvailableAt, ctx.link()) }
			</div>
		}
	}

	fn display_row<V: Clone + Default + fmt::Display + PartialEq + fmt::Debug>(
		title: &'static str,
		current: &Option<V>,
		new: &Option<V>,
		updating: UpdateValue,
		scope: &Scope<Self>,
	) -> Html {
		match (current, new) {
			(Some(old_value), Some(new_value)) if old_value != new_value => {
				html! {
					<div class="comparison-row">
						<div class="row-title"><span>{ title }</span></div>
						<div class="row-grow" onclick={ scope.callback(move |_| Msg::UpdateNew(updating, false)) }><div class="label">{ old_value.clone() }</div></div>
						<div class="row-grow" onclick={ scope.callback(move |_| Msg::UpdateNew(updating, true)) }><div class="label">{ new_value.clone() }</div></div>
					</div>
				}
			}

			(None, Some(new_value)) => {
				html! {
					<div class="comparison-row">
						<div class="row-title"><span>{ title }</span></div>
						<div class="row-grow" onclick={ scope.callback(move |_| Msg::UpdateNew(updating, false)) }><div class="label">{ "(Empty)" }</div></div>
						<div class="row-grow" onclick={ scope.callback(move |_| Msg::UpdateNew(updating, true)) }><div class="label">{ new_value.clone() }</div></div>
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
	Isbn10,
	Isbn13,
	AvailableAt,
}