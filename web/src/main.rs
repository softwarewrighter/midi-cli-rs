//! MIDI CLI Web UI - Yew frontend application.

mod api;
mod components;

// Include generated version info
mod version_info {
    include!(concat!(env!("OUT_DIR"), "/version_info.rs"));
}

use api::{ApiClient, PresetRequest, SavedPreset};
use components::{PresetEditor, PresetList};
use std::collections::HashMap;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

/// Main application state.
#[derive(Default)]
struct AppState {
    presets: Vec<SavedPreset>,
    editing: Option<SavedPreset>,
    generating: Option<String>,
    audio_urls: HashMap<String, String>,
    error: Option<String>,
    loading: bool,
}

/// Application messages for state updates.
enum Msg {
    /// Load presets from the server.
    LoadPresets,
    /// Presets loaded successfully.
    PresetsLoaded(Vec<SavedPreset>),
    /// Start editing a preset.
    EditPreset(SavedPreset),
    /// Clear the editor (cancel editing).
    ClearEditor,
    /// Save a new or updated preset.
    SavePreset(PresetRequest),
    /// Preset saved successfully.
    PresetSaved(SavedPreset),
    /// Delete a preset.
    DeletePreset(String),
    /// Preset deleted successfully.
    PresetDeleted(String),
    /// Generate audio for a preset.
    GenerateAudio(String),
    /// Audio generation completed.
    GenerationComplete(String, String),
    /// An error occurred.
    Error(String),
    /// Clear the error message.
    ClearError,
}

/// Main application component.
struct App {
    state: AppState,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        // Load presets on startup
        ctx.link().send_message(Msg::LoadPresets);
        Self {
            state: AppState {
                loading: true,
                ..Default::default()
            },
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::LoadPresets => {
                self.state.loading = true;
                let link = ctx.link().clone();
                spawn_local(async move {
                    match ApiClient::list_presets().await {
                        Ok(presets) => link.send_message(Msg::PresetsLoaded(presets)),
                        Err(e) => link.send_message(Msg::Error(e)),
                    }
                });
                true
            }

            Msg::PresetsLoaded(presets) => {
                self.state.presets = presets;
                self.state.loading = false;
                true
            }

            Msg::EditPreset(preset) => {
                self.state.editing = Some(preset);
                true
            }

            Msg::ClearEditor => {
                self.state.editing = None;
                true
            }

            Msg::SavePreset(req) => {
                let link = ctx.link().clone();
                let editing_id = self.state.editing.as_ref().map(|p| p.id.clone());

                spawn_local(async move {
                    let result = if let Some(id) = editing_id {
                        ApiClient::update_preset(&id, &req).await
                    } else {
                        ApiClient::create_preset(&req).await
                    };

                    match result {
                        Ok(preset) => link.send_message(Msg::PresetSaved(preset)),
                        Err(e) => link.send_message(Msg::Error(e)),
                    }
                });
                true
            }

            Msg::PresetSaved(_preset) => {
                self.state.editing = None;
                // Reload presets to get fresh data
                ctx.link().send_message(Msg::LoadPresets);
                true
            }

            Msg::DeletePreset(id) => {
                let link = ctx.link().clone();
                let id_clone = id.clone();
                spawn_local(async move {
                    match ApiClient::delete_preset(&id_clone).await {
                        Ok(()) => link.send_message(Msg::PresetDeleted(id_clone)),
                        Err(e) => link.send_message(Msg::Error(e)),
                    }
                });
                true
            }

            Msg::PresetDeleted(id) => {
                self.state.presets.retain(|p| p.id != id);
                self.state.audio_urls.remove(&id);
                if self.state.editing.as_ref().is_some_and(|p| p.id == id) {
                    self.state.editing = None;
                }
                true
            }

            Msg::GenerateAudio(id) => {
                self.state.generating = Some(id.clone());
                let link = ctx.link().clone();
                spawn_local(async move {
                    match ApiClient::generate_audio(&id).await {
                        Ok(response) => {
                            link.send_message(Msg::GenerationComplete(id, response.audio_url))
                        }
                        Err(e) => {
                            link.send_message(Msg::Error(e));
                            // Clear generating state even on error
                            link.send_message(Msg::GenerationComplete(id, String::new()));
                        }
                    }
                });
                true
            }

            Msg::GenerationComplete(id, audio_url) => {
                self.state.generating = None;
                if !audio_url.is_empty() {
                    self.state.audio_urls.insert(id, audio_url);
                }
                true
            }

            Msg::Error(error) => {
                self.state.error = Some(error);
                self.state.loading = false;
                true
            }

            Msg::ClearError => {
                self.state.error = None;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let on_save = ctx.link().callback(Msg::SavePreset);
        let on_clear = ctx.link().callback(|_| Msg::ClearEditor);
        let on_edit = ctx.link().callback(Msg::EditPreset);
        let on_delete = ctx.link().callback(Msg::DeletePreset);
        let on_generate = ctx.link().callback(Msg::GenerateAudio);
        let on_clear_error = ctx.link().callback(|_| Msg::ClearError);

        html! {
            <>
                // GitHub corner octocat
                <a href={version_info::REPO_URL} class="github-corner" aria-label="View source on GitHub" target="_blank" rel="noopener">
                    <svg viewBox="0 0 250 250" aria-hidden="true">
                        <path d="M0,0 L115,115 L130,115 L142,142 L250,250 L250,0 Z"></path>
                        <path d="M128.3,109.0 C113.8,99.7 119.0,89.6 119.0,89.6 C122.0,82.7 120.5,78.6 120.5,78.6 C119.2,72.0 123.4,76.3 123.4,76.3 C127.3,80.9 125.5,87.3 125.5,87.3 C122.9,97.6 130.6,101.9 134.4,103.2" fill="currentColor" style="transform-origin: 130px 106px;" class="octo-arm"></path>
                        <path d="M115.0,115.0 C114.9,115.1 118.7,116.5 119.8,115.4 L133.7,101.6 C136.9,99.2 139.9,98.4 142.2,98.6 C133.8,88.0 127.5,74.4 143.8,58.0 C148.5,53.4 154.0,51.2 159.7,51.0 C160.3,49.4 163.2,43.6 171.4,40.1 C171.4,40.1 176.1,42.5 178.8,56.2 C183.1,58.6 187.2,61.8 190.9,65.4 C194.5,69.0 197.7,73.2 200.1,77.6 C213.8,80.2 216.3,84.9 216.3,84.9 C212.7,93.1 206.9,96.0 205.4,96.6 C205.1,102.4 203.0,107.8 198.3,112.5 C181.9,128.9 168.3,122.5 157.7,114.1 C157.9,116.9 156.7,120.9 152.7,124.9 L141.0,136.5 C139.8,137.7 141.6,141.9 141.8,141.8 Z" fill="currentColor" class="octo-body"></path>
                    </svg>
                </a>

                <div class="app-container">
                    <header>
                        <h1>{"midi-cli-rs"}</h1>
                        <span class="separator">{"|"}</span>
                        <span class="subtitle">{"AI Music Preset Manager"}</span>
                    </header>

                    { if let Some(ref error) = self.state.error {
                        html! {
                            <div class="error-message" onclick={on_clear_error}>
                                {error}
                                <span style="float: right; cursor: pointer;">{"[x]"}</span>
                            </div>
                        }
                    } else {
                        html! {}
                    }}

                    <main class="main-content">
                        <PresetEditor
                            on_save={on_save}
                            editing={self.state.editing.clone()}
                            on_clear={on_clear}
                        />

                        <PresetList
                            presets={self.state.presets.clone()}
                            on_edit={on_edit}
                            on_delete={on_delete}
                            on_generate={on_generate}
                            generating={self.state.generating.clone()}
                            audio_urls={self.state.audio_urls.clone()}
                        />
                    </main>
                </div>

                <footer>
                    <div class="footer-content">
                        <div class="footer-left">
                            <span>{version_info::COPYRIGHT}</span>
                            <span>{"|"}</span>
                            <a href={version_info::LICENSE_URL} target="_blank" rel="noopener">
                                {format!("{} License", version_info::LICENSE)}
                            </a>
                        </div>
                        <div class="footer-right">
                            <span class="build-info">
                                {format!("v{} ({}) built {}",
                                    version_info::VERSION,
                                    version_info::GIT_COMMIT_SHORT,
                                    &version_info::BUILD_TIME[..10]  // Just the date
                                )}
                            </span>
                        </div>
                    </div>
                </footer>
            </>
        }
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
