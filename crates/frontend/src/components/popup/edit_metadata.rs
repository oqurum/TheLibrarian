use common::{
    api::WrappingResponse,
    component::popup::{Popup, PopupType},
    Either, ImageIdType,
};
use common_local::api;
use yew::prelude::*;

use crate::request;

#[derive(Clone, Copy)]
pub enum TabDisplay {
    General,
    Poster,
    Info,
}

#[derive(Properties, PartialEq)]
pub struct Property {
    #[prop_or_default]
    pub classes: Classes,

    pub on_close: Callback<()>,

    pub media_resp: api::MediaViewResponse,
}

pub enum Msg {
    RetrievePostersResponse(WrappingResponse<api::GetPostersResponse>),

    // Events
    SwitchTab(TabDisplay),

    UpdatedPoster,

    Ignore,
}

pub struct PopupEditMetadata {
    tab_display: TabDisplay,

    cached_posters: Option<WrappingResponse<api::GetPostersResponse>>,
}

impl Component for PopupEditMetadata {
    type Message = Msg;
    type Properties = Property;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            tab_display: TabDisplay::General,
            cached_posters: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Ignore => {
                return false;
            }

            Msg::SwitchTab(value) => {
                self.tab_display = value;
                self.cached_posters = None;
            }

            Msg::RetrievePostersResponse(resp) => {
                self.cached_posters = Some(resp);
            }

            Msg::UpdatedPoster => {
                let book_id = ctx.props().media_resp.metadata.id;

                ctx.link().send_future(async move {
                    Msg::RetrievePostersResponse(
                        request::get_posters_for_meta(ImageIdType::new_book(book_id), None).await,
                    )
                });

                return false;
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
                <div class="modal-header">
                    <h2 class="modal-title">{"Edit"}</h2>
                </div>

                <div class="tab-bar">
                    <div class="tab-bar-item" onclick={ctx.link().callback(|_| Msg::SwitchTab(TabDisplay::General))}>{ "General" }</div>
                    <div class="tab-bar-item" onclick={ctx.link().callback(|_| Msg::SwitchTab(TabDisplay::Poster))}>{ "Poster" }</div>
                    <div class="tab-bar-item" onclick={ctx.link().callback(|_| Msg::SwitchTab(TabDisplay::Info))}>{ "Info" }</div>
                </div>

                { self.render_tab_contents(ctx) }

                <div class="modal-footer">
                    <button class="btn btn-danger">{ "Cancel" }</button>
                    <button class="btn btn-success">{ "Save" }</button>
                </div>
            </Popup>
        }
    }
}

impl PopupEditMetadata {
    fn render_tab_contents(&self, ctx: &Context<Self>) -> Html {
        match self.tab_display {
            TabDisplay::General => self.render_tab_general(ctx.props()),
            TabDisplay::Poster => {
                if self.cached_posters.is_none() {
                    let book_id = ctx.props().media_resp.metadata.id;

                    ctx.link().send_future(async move {
                        Msg::RetrievePostersResponse(
                            request::get_posters_for_meta(ImageIdType::new_book(book_id), None)
                                .await,
                        )
                    });
                }

                self.render_tab_poster(ctx)
            }
            TabDisplay::Info => self.render_tab_info(ctx.props()),
        }
    }

    fn render_tab_general(&self, props: &<Self as Component>::Properties) -> Html {
        let resp = &props.media_resp;

        html! {
            <div class="modal-body">
                <label for="input-title">{ "Title" }</label>
                <input class="form-control" type="text" id="input-title" value={ resp.metadata.title.clone().unwrap_or_default() } />

                <label for="input-orig-title">{ "Original Title" }</label>
                <input class="form-control" type="text" id="input-orig-title" value={ resp.metadata.clean_title.clone().unwrap_or_default() } />

                <label for="input-descr">{ "Description" }</label>
                <textarea class="form-control" type="text" id="input-descr" rows="5" value={ resp.metadata.description.clone().unwrap_or_default() } />
            </div>
        }
    }

    fn render_tab_poster(&self, ctx: &Context<Self>) -> Html {
        if let Some(resp) = self.cached_posters.as_ref() {
            let resp = crate::continue_or_html_err!(resp);

            html! {
                <div class="modal-body edit-posters">
                    <div class="drop-container">
                        <h4>{ "Drop File To Upload" }</h4>
                    </div>
                    <div class="poster-list">
                        {
                            for resp.items.iter().map(|poster| {
                                let book_id = ctx.props().media_resp.metadata.id;
                                let url_or_id = poster.id.map(Either::Right).unwrap_or_else(|| Either::Left(poster.path.clone()));
                                let is_selected = poster.selected;

                                html_nested! {
                                    <div
                                        class={ classes!("poster", "normal", is_selected.then_some("selected")) }
                                        onclick={ctx.link().callback_future(move |_| {
                                            let url_or_id = url_or_id.clone();

                                            async move {
                                                if is_selected {
                                                    Msg::Ignore
                                                } else {
                                                    request::change_poster_for_meta(ImageIdType::new_book(book_id), url_or_id).await;

                                                    Msg::UpdatedPoster
                                                }
                                            }
                                        })}
                                    >
                                        <img src={poster.path.clone()} />
                                    </div>
                                }
                            })
                        }
                    </div>
                </div>
            }
        } else {
            html! {
                <div class="modal-body edit-posters">
                    <h3>{ "Loading Posters..." }</h3>
                </div>
            }
        }
    }

    fn render_tab_info(&self, _props: &<Self as Component>::Properties) -> Html {
        html! {
            <div class="modal-body"></div>
        }
    }
}
