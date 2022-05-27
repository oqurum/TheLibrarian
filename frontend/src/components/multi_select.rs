use std::rc::Rc;

use gloo_timers::future::TimeoutFuture;
use web_sys::HtmlInputElement;
use yew::{prelude::*, virtual_dom::VChild, html::Scope};


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

	OnHover(usize),

	OnKeyDown(KeyboardEvent),
	OnPressEnter,
	OnInputChange(KeyboardEvent),
}


pub struct MultiselectModule {
	input_ref: NodeRef,
	// On focus
	is_focused: bool,
	// Dropdown Opened
	is_opened: bool,

	selected_index: usize,
}

impl Component for MultiselectModule {
	type Message = Msg;
	type Properties = Property;

	fn create(_ctx: &Context<Self>) -> Self {
		Self {
			input_ref: NodeRef::default(),
			is_focused: false,
			is_opened: false,
			selected_index: 0,
		}
	}

	fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
		match msg {
			Msg::Update => (),
			Msg::Ignore => return false,

			Msg::OnHover(id) => {
				if id != 0 {
					if let Some(index) = self.get_child_index_by_id(id, ctx) {
						self.selected_index = index;
					}
				} else {
					self.selected_index = self.viewable_children_count(ctx);
				}
			}

			Msg::OnInputChange(event) => {
				let key = event.key();

				if key != "ArrowUp" && key != "ArrowDown" {
					self.selected_index = 0;
				}
			}

			Msg::OnPressEnter => {
				let child_count = self.viewable_children_count(ctx);

				if self.selected_index < child_count {
					let value = self.get_selected_child_id(ctx).expect("Couldn't get child value");

					return self.update(ctx, Msg::OnSelectItem(value));
				} else {
					return self.update(ctx, Msg::OnCreate);
				}
			}

			Msg::OnKeyDown(event) => {
				match event.key().as_str() {
					"ArrowUp" => if self.selected_index != 0 {
						self.selected_index -= 1;
					},

					"ArrowDown" => {
						let child_count = self.viewable_children_count(ctx);

						// Correct child count for if statement. If we're not displaying create value, minus one from child count.
						let child_count = if self.create_button_value().is_some() {
							child_count
						} else { // TODO: Remove this if statement. It complicates it.
							child_count.saturating_sub(1)
						};

						if child_count > self.selected_index {
							self.selected_index += 1;
						} else {
							self.selected_index = child_count;
						}
					},

					_ => ()
				}
			}

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
				self.selected_index = 0;
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
						onkeyup={ctx.link().callback(|e: KeyboardEvent| if e.key() == "Enter" { Msg::OnPressEnter } else { Msg::OnInputChange(e) })}
						onkeydown={ctx.link().callback(Msg::OnKeyDown)}
						type="text"
						placeholder="Enter Something"
					/>
				</div>
				<div class="dropdown">
					<div class="dropdown-list">
						{ for ctx.props().children.iter()
							.filter(|v| self.filter_viewable_child(v))
							.enumerate()
							.map(|(index, mut item)| {
								let mut props = Rc::make_mut(&mut item.props);

								props.hovering = index == self.selected_index;

								if props.callback.is_none() {
									props.callback = Some(ctx.link().clone());
								}

								item
							})
						}

						{
							if let Some(value) = self.create_button_value() {
								let children_count = self.viewable_children_count(ctx);

								html! {
									<div
										class={classes!("list-item", Some("hovering").filter(|_| children_count == self.selected_index))}
										onclick={ctx.link().callback(|_| Msg::OnCreate)}
										onmouseover={ctx.link().callback(|_| Msg::OnHover(0))}
									>
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
	fn create_button_value(&self) -> Option<String> {
		self.input_ref.cast::<HtmlInputElement>().map(|v| v.value().trim().to_string()).filter(|v| !v.is_empty())
	}

	fn create_selected_pill(ctx: &Context<Self>, props: &Rc<MultiselectItemProps>) -> Html {
		let item_id = props.id;

		html! {
			<div class="chosen-item" onmousedown={ctx.link().callback(move |_| Msg::OnUnselectItem(item_id))}>
				{ &props.name }
			</div>
		}
	}

	fn filter_viewable_child(&self, item: &VChild<MultiselectItem>) -> bool {
		let input_val_lc = self.input_ref.cast::<HtmlInputElement>().map(|v| v.value().to_lowercase());

		if let Some(v) = input_val_lc.as_deref() {
			if !item.props.name.to_lowercase().contains(v) {
				return false;
			}
		}

		!item.props.selected
	}

	fn viewable_children_count(&self, ctx: &Context<Self>) -> usize {
		ctx.props()
			.children
			.iter()
			.filter(|v| self.filter_viewable_child(v))
			.count()
	}

	fn get_selected_child_id(&self, ctx: &Context<Self>) -> Option<usize> {
		ctx.props()
			.children
			.iter()
			.filter(|v| self.filter_viewable_child(v))
			.enumerate()
			.find_map(|(index, item)| {
				if index == self.selected_index {
					Some(item.props.id)
				} else {
					None
				}
			})
	}

	fn get_child_index_by_id(&self, id: usize, ctx: &Context<Self>) -> Option<usize> {
		ctx.props()
			.children
			.iter()
			.filter(|v| self.filter_viewable_child(v))
			.enumerate()
			.find_map(|(index, item)| {
				if id == item.props.id {
					Some(index)
				} else {
					None
				}
			})
	}
}



#[derive(Clone)]
pub struct MultiselectNewItem {
	pub name: String,
	/// Registers the new item in the Multi Select Component
	pub register: Callback<usize>,
}











#[derive(Clone, Properties)]
pub struct MultiselectItemProps {
	pub id: usize, // TODO: Change to allow PartialEq
	pub name: String,

	pub callback: Option<Scope<MultiselectModule>>,

	#[prop_or_default]
	pub selected: bool,

	#[prop_or_default]
	hovering: bool, // TODO: Better name
}

impl PartialEq for MultiselectItemProps {
	fn eq(&self, other: &Self) -> bool {
		self.id == other.id &&
		self.name == other.name &&
		self.selected == other.selected &&
		self.hovering == other.hovering
	}
}


#[derive(Clone, PartialEq)]
pub enum MultiselectItemMessage {
	Selected,
}



pub struct MultiselectItem;

impl Component for MultiselectItem {
	type Message = MultiselectItemMessage;
	type Properties = MultiselectItemProps;

	fn create(_ctx: &Context<Self>) -> Self {
		Self
	}

	fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
		match msg {
			MultiselectItemMessage::Selected => {
				let props = ctx.props();
				props.callback.as_ref().unwrap().send_message(Msg::OnSelectItem(props.id));
			}
		}

		true
	}

	fn view(&self, ctx: &Context<Self>) -> Html {
		let cb = ctx.props().callback.clone().unwrap();
		let id = ctx.props().id;

		html! {
			<div
				class={classes!("list-item", Some("hovering").filter(|_| ctx.props().hovering))}
				onclick={ctx.link().callback(|_| MultiselectItemMessage::Selected)}
				onmouseover={move |_| cb.send_message(Msg::OnHover(id))}
			>
				{ &ctx.props().name }
			</div>
		}
	}
}