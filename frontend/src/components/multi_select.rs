use std::rc::Rc;

use web_sys::HtmlInputElement;
use yew::prelude::*;


#[derive(Properties)]
pub struct Property {
	pub children: ChildrenWithProps<MultiselectItem>,

	pub on_create_item: Option<Callback<MultiselectNewItem>>,
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

	selected: Vec<usize>,
}

impl Component for MultiselectModule {
	type Message = Msg;
	type Properties = Property;

	fn create(_ctx: &Context<Self>) -> Self {
		Self {
			input_ref: NodeRef::default(),
			is_focused: false,
			is_opened: false,
			selected: Vec::new(),
		}
	}

	fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
		match msg {
			Msg::Update => (),
			Msg::Ignore => return false,

			Msg::OnSelectItem(id) => {
				if !self.selected.contains(&id) {
					self.selected.push(id);
				}
			}

			Msg::OnUnselectItem(id) => {
				if let Some(index) = self.selected.iter().position(|v| *v == id) {
					self.selected.swap_remove(index);
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
		html! {
			<div class={classes!("multi-selection", Some("focused").filter(|_| self.is_focused), Some("opened").filter(|_| self.is_opened))}>
				<div class="input" onclick={ctx.link().callback(|_| Msg::SetFocus)}>
					<div class="chosen-list">
						{ for self.selected.iter().copied().filter_map(|i| Some(Self::create_selected_pill(ctx, &ctx.props().children.iter().find(|v| v.props.id == i)?.props))) }
					</div>
					<input
						ref={self.input_ref.clone()}
						onfocusin={ctx.link().callback(|_| Msg::OnFocus)}
						onfocusout={ctx.link().callback(|_| Msg::OnUnfocus)}
						onkeyup={ctx.link().callback(|e: KeyboardEvent| if e.key() == "Enter" { Msg::OnCreate } else { Msg::Update })}
						type="text"
						placeholder="Enter Something"
					/>
				</div>
				<div class="dropdown">
					<div class="dropdown-list">
						{ for ctx.props().children.iter().filter_map(|mut item| {
							let mut props = Rc::make_mut(&mut item.props);

							if self.selected.contains(&props.id) {
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
			<div class="chosen-item" onclick={ctx.link().callback(move |_| Msg::OnUnselectItem(item_id))}>
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