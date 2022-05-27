use gloo_file::{FileList, Blob};
use gloo_utils::window;
use librarian_common::{Either, BookId, PersonId};
use wasm_bindgen::{JsCast, prelude::Closure, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{HtmlElement, RequestInit};
use yew::prelude::*;


const PREV_DEFAULT_FN_NAMES: [&str; 7] = ["drag", "dragstart", "dragend", "dragover", "dragenter", "dragleave", "drop"];
const DRAGOVER_EVENTS: [&str; 2] = ["dragover", "dragenter"];
const DRAGLEAVE_EVENTS: [&str; 3] = ["dragleave", "dragend", "drop"];

#[derive(Properties, PartialEq)]
pub struct Property {
	pub class: Classes,
	pub title: Option<String>,

	pub children: Children,

	pub id: Either<BookId, PersonId>,

	pub on_upload: Option<Callback<()>>
}


pub enum Msg {
	Ignore,

	OnDragOver,
	OnDragLeave,
	OnDrop(FileList),
}

pub struct UploadModule {
	#[allow(clippy::type_complexity)]
	events_prev_defs: Vec<(&'static str, Closure<dyn FnMut(Event)>)>,
	#[allow(clippy::type_complexity)]
	events: Vec<(&'static str, Closure<dyn FnMut()>)>,

	container_ref: NodeRef,

	is_drag_over: bool,
}

impl Component for UploadModule {
	type Message = Msg;
	type Properties = Property;

	fn create(_ctx: &Context<Self>) -> Self {
		Self {
			events_prev_defs: Vec::new(),
			events: Vec::new(),

			container_ref: NodeRef::default(),

			is_drag_over: false,
		}
	}

	fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
		match msg {
			Msg::Ignore => return false,

			Msg::OnDragOver => self.is_drag_over = true,
			Msg::OnDragLeave => self.is_drag_over = false,

			Msg::OnDrop(files) => {
				let cb = ctx.props().on_upload.clone();
				let id = ctx.props().id;

				ctx.link()
				.send_future(async move {
					for file in files.iter() {
						match id {
							Either::Left(id) => {
								let mut opts = RequestInit::new();
								opts.method("POST");
								opts.body(Some(&JsValue::from((file as &Blob).clone())));

								let _ = JsFuture::from(window().fetch_with_str_and_init(
									&format!("/api/v1/posters/{}/upload", id),
									&opts
								)).await;
							}

							Either::Right(_id) => {
								//
							}
						}
					}

					if let Some(cb) = cb {
						cb.emit(());
					}

					Msg::Ignore
				});
			}
		}

		true
	}

	fn view(&self, ctx: &Context<Self>) -> Html {
		html! {
			<div
				class={classes!(ctx.props().class.clone(), Some("dragging-over").filter(|_| self.is_drag_over))}
				title={ctx.props().title.clone()}
				ref={self.container_ref.clone()}
				ondrop={ctx.link().callback(|event: DragEvent| {
					if let Some(files) = event.data_transfer().and_then(|v| v.files()) {
						Msg::OnDrop(files.into())
					} else {
						Msg::Ignore
					}
				})}
			>
				{ for ctx.props().children.iter() }

				// <label for="file-input">Choose Photo/Video</label>
				// <input id="file-input" type="file" name="files" multiple="" accept=".jpg,.jpeg,.png,.gif,.apng,.tiff,.tif,.bmp,.xcf,.webp,.mp4,.mov,.avi,.webm,.mpeg,.flv,.mkv,.mpv,.wmv">
			</div>
		}
	}

	fn rendered(&mut self, ctx: &Context<Self>, _first_render: bool) {
		let cont = self.container_ref.cast::<HtmlElement>().unwrap();

		self.disable();

		{ // Prevent Defaults
			for name in PREV_DEFAULT_FN_NAMES {
				let evee = Closure::wrap(Box::new(move |event: Event| {
					event.prevent_default();
					event.stop_propagation();
				}) as Box<dyn FnMut(Event)>);

				let _ = cont.add_event_listener_with_callback(name, evee.as_ref().unchecked_ref());

				self.events_prev_defs.push((name, evee));
			}
		}

		{ // Drag Over
			for name in DRAGOVER_EVENTS {
				let link = ctx.link().clone();

				let evee = Closure::wrap(Box::new(move || link.send_message(Msg::OnDragOver)) as Box<dyn FnMut()>);
				let _ = cont.add_event_listener_with_callback(name, evee.as_ref().unchecked_ref());

				self.events.push((name, evee));
			}
		}

		{ // Drag Leave
			for name in DRAGLEAVE_EVENTS {
				let link = ctx.link().clone();

				let evee = Closure::wrap(Box::new(move || link.send_message(Msg::OnDragLeave)) as Box<dyn FnMut()>);
				let _ = cont.add_event_listener_with_callback(name, evee.as_ref().unchecked_ref());

				self.events.push((name, evee));
			}
		}
	}

	fn destroy(&mut self, _ctx: &Context<Self>) {
		self.disable();
	}
}

impl UploadModule {
	fn disable(&mut self) {
		let cont = self.container_ref.cast::<HtmlElement>().unwrap();

		for (name, func) in self.events_prev_defs.drain(..) {
			let _ = cont.remove_event_listener_with_callback(name, func.as_ref().unchecked_ref());
		}

		for (name, func) in self.events.drain(..) {
			let _ = cont.remove_event_listener_with_callback(name, func.as_ref().unchecked_ref());
		}
	}
}