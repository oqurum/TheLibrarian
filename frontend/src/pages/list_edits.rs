use chrono::Utc;
use librarian_common::{api, item::edit::*, edit::*, get_language_name};
use yew::{prelude::*, html::Scope};

use crate::{request, get_member_self};


#[derive(Properties, PartialEq)]
pub struct Property {
}

#[derive(Clone)]
pub enum Msg {
	// Requests
	RequestEdits,

	// Results
	EditListResults(api::GetEditListResponse),

	EditItemUpdate(api::PostEditResponse),

	Ignore,
}

pub struct EditListPage {
	items_resp: Option<api::GetEditListResponse>,
}

impl Component for EditListPage {
	type Message = Msg;
	type Properties = Property;

	fn create(ctx: &Context<Self>) -> Self {
		ctx.link().send_message(Msg::RequestEdits);

		Self {
			items_resp: None,
		}
	}

	fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
		match msg {
			Msg::RequestEdits => {
				ctx.link()
				.send_future(async move {
					Msg::EditListResults(request::get_edit_list(None, None).await)
				});
			}

			Msg::EditListResults(resp) => {
				self.items_resp = Some(resp);
			}

			Msg::EditItemUpdate(mut item) => {
				if let Some(my_vote) = item.vote.take() {
					if let Some(all_edit_items) = self.items_resp.as_mut() {
						if let Some(edit_model) = all_edit_items.items.iter_mut().find(|v| v.id == my_vote.edit_id) {
							if let Some(votes) = edit_model.votes.as_mut() {
								// Insert or Update Vote
								if let Some(curr_vote_pos) = votes.items.iter_mut().position(|v| v.id == my_vote.id) {
									if votes.items[curr_vote_pos].vote == my_vote.vote {
										votes.items.remove(curr_vote_pos);
									} else {
										let _ = std::mem::replace(&mut votes.items[curr_vote_pos], my_vote);
									}
								} else {
									votes.items.push(my_vote);
								}
							}
						}
					}
				}

			}

			Msg::Ignore => return false,
		}

		true
	}

	fn view(&self, ctx: &Context<Self>) -> Html {
		if let Some(resp) = self.items_resp.as_ref() {
			html! {
				<div class="main-content-view edit-list-view-container">

					<div class="list-items">
						{ for resp.items.iter().map(|item| self.render_item(item, ctx.link())) }
					</div>
				</div>
			}
		} else {
			html! {
				<h1>{ "Loading..." }</h1>
			}
		}
	}
}

impl EditListPage {
	fn render_item(&self, item: &SharedEditModel, scope: &Scope<Self>) -> Html {
		let id = item.id;

		let status_color = match item.status {
			EditStatus::Accepted |
			EditStatus::ForceAccepted => "green",
			EditStatus::Pending => "yellow",
			EditStatus::Rejected |
			EditStatus::Failed |
			EditStatus::Cancelled |
			EditStatus::ForceRejected => "red",
		};

		let my_vote = self.get_my_vote(item);

		html! {
			<div class="editing-item-card">
				<div class="header">
					// Left
					<div class="aligned-left">
						<h5>{ item.operation.get_name() } { " " } { item.type_of.get_name() }</h5>
						<div>
							<b>{ "Member: " }</b>
							{
								if let Some(member) = item.member.as_ref() {
									html! {
										<span>{ member.name.clone() }</span>
									}
								} else {
									html! {}
								}
							}
						</div>
						<div>
							<b>{ "Created: " }</b>
							<span>{ item.created_at.format("%b %e, %Y %T %p").to_string() }</span>
						</div>
					</div>

					// Right
					<div class="aligned-right">
						<div>
							<b>{ "Status: " }</b>
							<span class={classes!("label", status_color)}>{ item.status.get_name() }</span>
						</div>

						{
							if let Some(expires_at) = item.expires_at {
								html! {
									<div title={ expires_at.format("%b %e, %Y %T %p").to_string() }>
										<b>{ "Closes: " }</b>
										<span>{ expires_at.date().signed_duration_since(Utc::today()).num_days() } { " days" }</span>
									</div>
								}
							} else {
								html! {
									<div>
										<b>{ "Closes: " }</b>
										<span>{ "Never" }</span>
									</div>
								}
							}
						}

						<div>
							<b>{ "Vote Count: " }</b>
							<span>{ item.vote_count }</span>
						</div>
					</div>
				</div>

				<hr class="transparent" />

				<div class="body">
					{
						match &item.data {
							EditData::Book(v) => Self::generate_book_rows(v, scope),
							_ => todo!(),
						}
					}
				</div>

				<hr class="transparent" />

				<div class="footer">
					<div class="aligned-left">
						{
							if let Some(is_selected) = my_vote.map(|v| !v).or(Some(false)) {
								html! {
									<button
										class={ if is_selected || my_vote.is_none() { "red" } else { "disabled" }}
										title="Downvote"
										onclick={scope.callback_future(move |_| async move {
											let resp = request::update_edit_item(id, &UpdateEditModel {
												vote: Some(-1),
												.. UpdateEditModel::default()
											}).await;

											Msg::EditItemUpdate(resp)
										})}
									><span class="material-icons">{ "keyboard_arrow_down" }</span></button>
								}
							} else {
								html! {}
							}
						}

						{
							if let Some(is_selected) = my_vote.or(Some(false)) {
								html! {
									<button
									class={ if is_selected || my_vote.is_none() { "green" } else { "disabled" }}
										title="Upvote"
										onclick={scope.callback_future(move |_| async move {
											let resp = request::update_edit_item(id, &UpdateEditModel {
												vote: Some(1),
												.. UpdateEditModel::default()
											}).await;

											Msg::EditItemUpdate(resp)
										})}
									><span class="material-icons">{ "keyboard_arrow_up" }</span></button>
								}
							} else {
								html! {}
							}
						}
					</div>
					<div class="aligned-right">
						<button
							class="red"
							onclick={scope.callback_future(move |_| async move {
								let resp = request::update_edit_item(id, &UpdateEditModel {
									status: Some(EditStatus::ForceRejected),
									.. UpdateEditModel::default()
								}).await;

								Msg::EditItemUpdate(resp)
							})}
						>{ "Force Reject" }</button>

						<button
							class="green"
							onclick={scope.callback_future(move |_| async move {
								let resp = request::update_edit_item(id, &UpdateEditModel {
									status: Some(EditStatus::ForceAccepted),
									.. UpdateEditModel::default()
								}).await;

								Msg::EditItemUpdate(resp)
							})}
						>{ "Force Accept" }</button>
					</div>
				</div>
			</div>
		}
	}

	fn generate_book_rows(book_edit_data: &BookEditData, _scope: &Scope<Self>) -> Html {
		let (new_data, old_data) = match (&book_edit_data.new, &book_edit_data.old) {
			(Some(a), Some(b)) => (a, b),
			_ => return html! {},
		};

		html! {
			<>
				{ // Title
					if let Some((new, old)) = join_new_old(&new_data.title, &old_data.title) {
						html! {
							<div class="row">
								<div class="row-title"><span>{ "Title" }</span></div>
								<div class="row-old"><div class="label red">{ old.clone().unwrap_or_default() }</div></div>
								<div class="row-new"><div class="label green">{ new.clone() }</div></div>
							</div>
						}
					} else {
						html! {}
					}
				}

				{ // Clean Title
					if let Some((new, old)) = join_new_old(&new_data.clean_title, &old_data.clean_title) {
						html! {
							<div class="row">
								<div class="row-title"><span>{ "Clean Title" }</span></div>
								<div class="row-old"><div class="label red">{ old.clone().unwrap_or_default() }</div></div>
								<div class="row-new"><div class="label green">{ new.clone() }</div></div>
							</div>
						}
					} else {
						html! {}
					}
				}

				{ // Description
					if let Some((new, old)) = join_new_old(&new_data.description, &old_data.description) {
						html! {
							<div class="row">
								<div class="row-title"><span>{ "Description" }</span></div>
								<div class="row-old"><div class="label red">{ old.clone().unwrap_or_default() }</div></div>
								<div class="row-new"><div class="label green">{ new.clone() }</div></div>
							</div>
						}
					} else {
						html! {}
					}
				}

				{ // Rating
					if let Some((new, old)) = join_new_old(&new_data.rating, &old_data.rating) {
						html! {
							<div class="row">
								<div class="row-title"><span>{ "Rating" }</span></div>
								<div class="row-old"><div class="label red">{ old.unwrap_or_default() }</div></div>
								<div class="row-new"><div class="label green">{ new }</div></div>
							</div>
						}
					} else {
						html! {}
					}
				}

				{ // ISBN 10
					if let Some((new, old)) = join_new_old(&new_data.isbn_10, &old_data.isbn_10) {
						html! {
							<div class="row">
								<div class="row-title"><span>{ "ISBN 10" }</span></div>
								<div class="row-old"><div class="label red">{ old.clone().unwrap_or_default() }</div></div>
								<div class="row-new"><div class="label green">{ new.clone() }</div></div>
							</div>
						}
					} else {
						html! {}
					}
				}

				{ // ISBN 13
					if let Some((new, old)) = join_new_old(&new_data.isbn_13, &old_data.isbn_13) {
						html! {
							<div class="row">
								<div class="row-title"><span>{ "ISBN 13" }</span></div>
								<div class="row-old"><div class="label red">{ old.clone().unwrap_or_default() }</div></div>
								<div class="row-new"><div class="label green">{ new.clone() }</div></div>
							</div>
						}
					} else {
						html! {}
					}
				}

				{ // Publicity
					if let Some((new, old)) = join_new_old(&new_data.is_public, &old_data.is_public) {
						html! {
							<div class="row">
								<div class="row-title"><span>{ "Is Public" }</span></div>
								<div class="row-old"><div class="label red">{ old.unwrap_or_default() }</div></div>
								<div class="row-new"><div class="label green">{ new }</div></div>
							</div>
						}
					} else {
						html! {}
					}
				}

				{ // Available At
					if let Some((new, old)) = join_new_old(&new_data.available_at, &old_data.available_at) {
						html! {
							<div class="row">
								<div class="row-title"><span>{ "Available At" }</span></div>
								<div class="row-old"><div class="label red">{ old.clone().unwrap_or_default() }</div></div>
								<div class="row-new"><div class="label green">{ new.clone() }</div></div>
							</div>
						}
					} else {
						html! {}
					}
				}

				{ // Language
					if let Some((&new, &old)) = join_new_old(&new_data.language, &old_data.language) {
						html! {
							<div class="row">
								<div class="row-title"><span>{ "Language" }</span></div>
								<div class="row-old"><div class="label red">{ old.and_then(get_language_name).unwrap_or_default() }</div></div>
								<div class="row-new"><div class="label green">{ get_language_name(new).unwrap_or_default() }</div></div>
							</div>
						}
					} else {
						html! {}
					}
				}

				{ // Publisher
					if let Some((new, old)) = join_new_old(&new_data.publisher, &old_data.publisher) {
						html! {
							<div class="row">
								<div class="row-title"><span>{ "Publisher" }</span></div>
								<div class="row-old"><div class="label red">{ old.clone().unwrap_or_default() }</div></div>
								<div class="row-new"><div class="label green">{ new.clone() }</div></div>
							</div>
						}
					} else {
						html! {}
					}
				}

				// TODO: People, Tags, Images
			</>
		}
	}

	fn get_my_vote(&self, item: &SharedEditModel) -> Option<bool> {
		let votes = item.votes.as_ref()?;

		votes.items.iter().find_map(|v| if v.member_id? == get_member_self()?.id { Some(v.vote) } else { None })
	}
}


fn join_new_old<'a, V>(new: &'a Option<V>, old: &'a Option<V>) -> Option<(&'a V, &'a Option<V>)> {
	match (new, old) {
		(Some(n), o) => Some((n, o)),
		_ => None,
	}
}