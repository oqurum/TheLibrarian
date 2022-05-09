use std::rc::Rc;

use gloo_timers::future::TimeoutFuture;
use web_sys::HtmlInputElement;
use yew::prelude::*;


#[derive(Properties)]
pub struct Property {
	pub children: ChildrenWithProps<MultiselectItem>,

	pub on_create_item: Option<Callback<MultiselectNewItem>>,
	pub on_toggle_item: Option<Callback<(bool, usize)>>,
}

impl PartialEq for Property {
	fn eq(&self, other: &Self) -> bool {
		self.children == other.children
	}
}


pub enum Msg {
	Update,
	Ignore,

	OnUnfocus,
	OnFocus,
	SetFocus,

	OnSelectItem(usize),
	OnUnselectItem(usize),
	OnCreate,
}


pub struct MultiselectModule {
	input_ref: NodeRef,
	// On focus
	is_focused: bool,
	// Dropdown Opened
	is_opened: bool,
}

impl Component for MultiselectModule {
	type Message = Msg;
	type Properties = Property;

	fn create(_ctx: &Context<Self>) -> Self {
		Self {
			input_ref: NodeRef::default(),
			is_focused: false,
			is_opened: false,
		}
	}

	fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
		match msg {
			Msg::Update => (),
			Msg::Ignore => return false,

			Msg::OnSelectItem(id) => {
				log::info!("-- Selected: {}", id);

				if let Some(mut item) = ctx.props().children.iter().find(|v| v.props.id == id) {
					let mut props = Rc::make_mut(&mut item.props);
					props.selected = true;

					if let Some(cb) = ctx.props().on_toggle_item.as_ref() {
						cb.emit((true, props.id));
					}
				}
			}

			Msg::OnUnselectItem(id) => {
				if let Some(mut item) = ctx.props().children.iter().find(|v| v.props.id == id) {
					let mut props = Rc::make_mut(&mut item.props);
					props.selected = false;

					if let Some(cb) = ctx.props().on_toggle_item.as_ref() {
						cb.emit((false, props.id));
					}
				}
			}

			Msg::OnCreate => {
				if let Some(input) = self.input_ref.cast::<HtmlInputElement>() {
					let value = input.value().trim().to_string();

					if !value.is_empty() {
						if let Some(cb) = ctx.props().on_create_item.as_ref() {
							input.set_value("");

							cb.emit(MultiselectNewItem {
								name: value,
								register: ctx.link().callback(Msg::OnSelectItem),
							});
						}
					}
				}
			}


			// Focus

			Msg::SetFocus => {
				if let Some(input) = self.input_ref.cast::<HtmlInputElement>() {
					let _ = input.focus();
				}
			}

			Msg::OnFocus => {
				self.is_focused = true;
			}

			Msg::OnUnfocus => {
				self.is_focused = false;
			}
		}

		self.is_opened = self.is_focused && (
			!ctx.props().children.is_empty() ||
			self.input_ref.cast::<HtmlInputElement>().map(|v| !v.value().trim().is_empty()).unwrap_or_default()
		);

		true
	}

	fn view(&self, ctx: &Context<Self>) -> Html {
		let input_val_lc = self.input_ref.cast::<HtmlInputElement>().map(|v| v.value().to_lowercase());

		html! {
			<div class={classes!("multi-selection", Some("focused").filter(|_| self.is_focused), Some("opened").filter(|_| self.is_opened))}>
				<div class="input" onclick={ctx.link().callback(|_| Msg::SetFocus)}>
					<div class="chosen-list">
						{ for ctx.props().children.iter().filter(|v| v.props.selected).map(|child| Self::create_selected_pill(ctx, &child.props)) }
					</div>
					<input
						ref={self.input_ref.clone()}
						onfocusin={ctx.link().callback(|_| Msg::OnFocus)}
						onfocusout={ctx.link().callback_future(|_| async {
							// TODO: Fix. Used since we unfocus when we click the dropdown. This provides enough time for the onmousedown event to fire.
							TimeoutFuture::new(100).await;
							Msg::OnUnfocus
						})}
						onkeyup={ctx.link().callback(|e: KeyboardEvent| if e.key() == "Enter" { Msg::OnCreate } else { Msg::Update })}
						type="text"
						placeholder="Enter Something"
					/>
				</div>
				<div class="dropdown">
					<div class="dropdown-list">
						{ for ctx.props().children.iter().filter_map(|mut item| {
							let mut props = Rc::make_mut(&mut item.props);

							if let Some(v) = input_val_lc.as_deref() {
								if !props.name.to_lowercase().contains(v) {
									return None;
								}
							}

							if props.selected {
								None
							} else {
								if props.on_click.is_none() {
									props.on_click = Some(ctx.link().callback(Msg::OnSelectItem));
								}

								Some(item)
							}
						}) }

						{
							if let Some(value) = self.input_ref.cast::<HtmlInputElement>().map(|v| v.value()).filter(|v| !v.trim().is_empty()) {
								html! {
									<div class="list-item" onclick={ctx.link().callback(|_| Msg::OnCreate)}>
										{ format!(r#"Create "{value}""#) }
									</div>
								}
							} else {
								html! {}
							}
						}
					</div>
				</div>
			</div>
		}
	}
}

impl MultiselectModule {
	fn create_selected_pill(ctx: &Context<Self>, props: &Rc<MultiselectItemProps>) -> Html {
		let item_id = props.id;

		html! {
			<div class="chosen-item" onmousedown={ctx.link().callback(move |_| Msg::OnUnselectItem(item_id))}>
				{ &props.name }
			</div>
		}
	}
}



#[derive(Clone)]
pub struct MultiselectNewItem {
	pub name: String,
	/// Registers the new item in the Multi Select Component
	pub register: Callback<usize>,
}











#[derive(Clone, Properties, PartialEq)]
pub struct MultiselectItemProps {
	pub id: usize,
	pub name: String,
	pub on_click: Option<Callback<usize>>,

	#[prop_or_default]
	pub selected: bool,
}

pub struct MultiselectItem;

impl Component for MultiselectItem {
	type Message = bool;
	type Properties = MultiselectItemProps;

	fn create(_ctx: &Context<Self>) -> Self {
		Self
	}

	fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
		if msg {
			let props = ctx.props();
			props.on_click.as_ref().unwrap().emit(props.id);
		}

		msg
	}

	fn view(&self, ctx: &Context<Self>) -> Html {
		html! {
			<div class="list-item" onclick={ctx.link().callback(|_| true)}>
				{ &ctx.props().name }
			</div>
		}
	}
}