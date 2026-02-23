//! Plugin manager component for managing mood packs.

use crate::api::MoodPackInfo;
use wasm_bindgen::JsCast;
use web_sys::{FileReader, HtmlInputElement};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub plugins: Vec<MoodPackInfo>,
    pub on_refresh: Callback<()>,
    pub on_delete: Callback<String>,
    pub on_upload: Callback<String>,
    pub loading: bool,
}

pub enum Msg {
    FileSelected,
    FileLoaded(String),
    ToggleExpanded(String),
}

pub struct PluginManager {
    expanded: Option<String>,
    file_input_ref: NodeRef,
}

impl Component for PluginManager {
    type Message = Msg;
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            expanded: None,
            file_input_ref: NodeRef::default(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::FileSelected => {
                if let Some(input) = self.file_input_ref.cast::<HtmlInputElement>() {
                    if let Some(files) = input.files() {
                        if let Some(file) = files.get(0) {
                            let reader = FileReader::new().unwrap();
                            let reader_clone = reader.clone();
                            let link = ctx.link().clone();

                            let onload = wasm_bindgen::closure::Closure::wrap(Box::new(move || {
                                if let Ok(result) = reader_clone.result() {
                                    if let Some(text) = result.as_string() {
                                        link.send_message(Msg::FileLoaded(text));
                                    }
                                }
                            })
                                as Box<dyn FnMut()>);

                            reader.set_onload(Some(onload.as_ref().unchecked_ref()));
                            onload.forget();
                            let _ = reader.read_as_text(&file);
                        }
                    }
                }
                false
            }
            Msg::FileLoaded(content) => {
                ctx.props().on_upload.emit(content);
                // Clear the file input
                if let Some(input) = self.file_input_ref.cast::<HtmlInputElement>() {
                    input.set_value("");
                }
                false
            }
            Msg::ToggleExpanded(name) => {
                if self.expanded.as_ref() == Some(&name) {
                    self.expanded = None;
                } else {
                    self.expanded = Some(name);
                }
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let on_file_change = ctx.link().callback(|_| Msg::FileSelected);
        let on_refresh = {
            let callback = ctx.props().on_refresh.clone();
            Callback::from(move |_| callback.emit(()))
        };

        html! {
            <div class="plugin-manager">
                <div class="card">
                    <h2>{"Mood Plugins"}</h2>

                    <div class="plugin-actions">
                        <label class="btn btn-primary upload-btn">
                            {"Upload TOML"}
                            <input
                                type="file"
                                accept=".toml"
                                ref={self.file_input_ref.clone()}
                                onchange={on_file_change}
                                style="display: none;"
                            />
                        </label>
                        <button class="btn btn-secondary" onclick={on_refresh}>
                            {"Refresh"}
                        </button>
                    </div>

                    <div class="plugin-info">
                        <p class="text-muted">
                            {"Mood plugins are TOML files that define custom mood presets. "}
                            {"Place them in "}
                            <code>{"~/.midi-cli-rs/moods/"}</code>
                            {" or upload here."}
                        </p>
                    </div>

                    { if ctx.props().loading {
                        html! { <div class="loading-spinner"><div class="loading"></div></div> }
                    } else if ctx.props().plugins.is_empty() {
                        html! {
                            <div class="empty-state">
                                <p>{"No custom mood plugins installed."}</p>
                                <p class="text-muted">{"Upload a TOML file to add custom moods."}</p>
                            </div>
                        }
                    } else {
                        html! {
                            <div class="plugin-list">
                                { for ctx.props().plugins.iter().map(|plugin| {
                                    self.view_plugin(ctx, plugin)
                                })}
                            </div>
                        }
                    }}
                </div>

                <div class="card">
                    <h2>{"Built-in Moods"}</h2>
                    <div class="builtin-moods">
                        { Self::view_builtin_mood("suspense", "Am", "Tense low drone with tremolo and sparse hits") }
                        { Self::view_builtin_mood("eerie", "Dm", "Sparse unsettling atmosphere with bell accents") }
                        { Self::view_builtin_mood("upbeat", "C", "Energetic rhythmic feel with clear pulse") }
                        { Self::view_builtin_mood("calm", "G", "Peaceful sustained pads and gentle arpeggios") }
                        { Self::view_builtin_mood("ambient", "Em", "Evolving drone textures with sporadic tones") }
                        { Self::view_builtin_mood("jazz", "F", "Nightclub trio with walking bass and piano") }
                    </div>
                </div>
            </div>
        }
    }
}

impl PluginManager {
    fn view_plugin(&self, ctx: &Context<Self>, plugin: &MoodPackInfo) -> Html {
        let is_expanded = self.expanded.as_ref() == Some(&plugin.name);
        let name = plugin.name.clone();
        let name_for_toggle = name.clone();
        let name_for_delete = name.clone();

        let on_toggle = ctx
            .link()
            .callback(move |_| Msg::ToggleExpanded(name_for_toggle.clone()));
        let on_delete = {
            let callback = ctx.props().on_delete.clone();
            Callback::from(move |_| callback.emit(name_for_delete.clone()))
        };

        html! {
            <div class="plugin-item">
                <div class="plugin-header" onclick={on_toggle}>
                    <div class="plugin-title">
                        <span class="plugin-name">{&plugin.name}</span>
                        <span class="plugin-version">{format!("v{}", plugin.version)}</span>
                    </div>
                    <span class="plugin-mood-count">
                        {format!("{} mood{}", plugin.mood_count, if plugin.mood_count == 1 { "" } else { "s" })}
                    </span>
                    <span class="expand-icon">{if is_expanded { "▼" } else { "▶" }}</span>
                </div>

                { if is_expanded {
                    html! {
                        <div class="plugin-details">
                            { if let Some(ref desc) = plugin.description {
                                html! { <p class="plugin-description">{desc}</p> }
                            } else {
                                html! {}
                            }}

                            { if let Some(ref author) = plugin.author {
                                html! { <p class="plugin-author">{"Author: "}{author}</p> }
                            } else {
                                html! {}
                            }}

                            <div class="plugin-moods">
                                <h4>{"Moods in this pack:"}</h4>
                                <ul>
                                    { for plugin.moods.iter().map(|mood| {
                                        html! {
                                            <li>
                                                <code>{&mood.name}</code>
                                                {" - "}
                                                <span class="mood-key">{&mood.default_key}</span>
                                                { if let Some(ref desc) = mood.description {
                                                    html! { <span class="mood-desc">{" - "}{desc}</span> }
                                                } else {
                                                    html! {}
                                                }}
                                            </li>
                                        }
                                    })}
                                </ul>
                            </div>

                            <div class="plugin-actions">
                                <button class="btn btn-danger btn-small" onclick={on_delete}>
                                    {"Remove Plugin"}
                                </button>
                            </div>
                        </div>
                    }
                } else {
                    html! {}
                }}
            </div>
        }
    }

    fn view_builtin_mood(name: &str, key: &str, description: &str) -> Html {
        html! {
            <div class="builtin-mood-item">
                <code class="mood-name">{name}</code>
                <span class="mood-key">{key}</span>
                <span class="mood-description">{description}</span>
            </div>
        }
    }
}
