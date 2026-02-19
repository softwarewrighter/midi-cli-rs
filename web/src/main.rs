//! MIDI CLI Web UI - Yew frontend application.

mod api;
mod components;

mod version_info {
    include!(concat!(env!("OUT_DIR"), "/version_info.rs"));
}

use api::{ApiClient, MelodyRequest, PresetRequest, SavedMelody, SavedPreset};
use components::{MelodyEditor, MelodyList, PresetEditor, PresetList};
use std::collections::HashMap;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tab {
    Presets,
    Melodies,
}

#[derive(Default)]
struct AppState {
    active_tab: Option<Tab>,
    // Presets
    presets: Vec<SavedPreset>,
    editing_preset: Option<SavedPreset>,
    generating_preset: Option<String>,
    preset_audio_urls: HashMap<String, String>,
    // Melodies
    melodies: Vec<SavedMelody>,
    editing_melody: Option<SavedMelody>,
    generating_melody: Option<String>,
    melody_audio_urls: HashMap<String, String>,
    // Common
    error: Option<String>,
    loading: bool,
}

impl AppState {
    fn new() -> Self {
        Self {
            active_tab: Some(Tab::Presets),
            loading: true,
            ..Default::default()
        }
    }
}

enum Msg {
    // Tab
    SwitchTab(Tab),
    // Presets
    LoadPresets,
    PresetsLoaded(Vec<SavedPreset>),
    EditPreset(SavedPreset),
    ClearPresetEditor,
    SavePreset(PresetRequest),
    PresetSaved(SavedPreset),
    DeletePreset(String),
    PresetDeleted(String),
    GeneratePresetAudio(String),
    PresetGenerationComplete(String, String),
    // Melodies
    LoadMelodies,
    MelodiesLoaded(Vec<SavedMelody>),
    EditMelody(SavedMelody),
    ClearMelodyEditor,
    SaveMelody(MelodyRequest),
    MelodySaved(SavedMelody),
    DeleteMelody(String),
    MelodyDeleted(String),
    GenerateMelodyAudio(String),
    MelodyGenerationComplete(String, String),
    // Common
    Error(String),
    ClearError,
}

struct App {
    state: AppState,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        ctx.link().send_message(Msg::LoadPresets);
        ctx.link().send_message(Msg::LoadMelodies);
        Self {
            state: AppState::new(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::SwitchTab(tab) => {
                self.state.active_tab = Some(tab);
                true
            }

            // Preset handlers
            Msg::LoadPresets => {
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
                self.state.editing_preset = Some(preset);
                true
            }
            Msg::ClearPresetEditor => {
                self.state.editing_preset = None;
                true
            }
            Msg::SavePreset(req) => {
                let link = ctx.link().clone();
                let editing_id = self.state.editing_preset.as_ref().map(|p| p.id.clone());
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
            Msg::PresetSaved(_) => {
                self.state.editing_preset = None;
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
                self.state.preset_audio_urls.remove(&id);
                if self.state.editing_preset.as_ref().is_some_and(|p| p.id == id) {
                    self.state.editing_preset = None;
                }
                true
            }
            Msg::GeneratePresetAudio(id) => {
                self.state.generating_preset = Some(id.clone());
                let link = ctx.link().clone();
                spawn_local(async move {
                    match ApiClient::generate_preset_audio(&id).await {
                        Ok(response) => {
                            link.send_message(Msg::PresetGenerationComplete(id, response.audio_url))
                        }
                        Err(e) => {
                            link.send_message(Msg::Error(e));
                            link.send_message(Msg::PresetGenerationComplete(id, String::new()));
                        }
                    }
                });
                true
            }
            Msg::PresetGenerationComplete(id, audio_url) => {
                self.state.generating_preset = None;
                if !audio_url.is_empty() {
                    self.state.preset_audio_urls.insert(id, audio_url);
                }
                true
            }

            // Melody handlers
            Msg::LoadMelodies => {
                let link = ctx.link().clone();
                spawn_local(async move {
                    match ApiClient::list_melodies().await {
                        Ok(melodies) => link.send_message(Msg::MelodiesLoaded(melodies)),
                        Err(e) => link.send_message(Msg::Error(e)),
                    }
                });
                true
            }
            Msg::MelodiesLoaded(melodies) => {
                self.state.melodies = melodies;
                true
            }
            Msg::EditMelody(melody) => {
                self.state.editing_melody = Some(melody);
                true
            }
            Msg::ClearMelodyEditor => {
                self.state.editing_melody = None;
                true
            }
            Msg::SaveMelody(req) => {
                let link = ctx.link().clone();
                let editing_id = self.state.editing_melody.as_ref().map(|m| m.id.clone());
                spawn_local(async move {
                    let result = if let Some(id) = editing_id {
                        ApiClient::update_melody(&id, &req).await
                    } else {
                        ApiClient::create_melody(&req).await
                    };
                    match result {
                        Ok(melody) => link.send_message(Msg::MelodySaved(melody)),
                        Err(e) => link.send_message(Msg::Error(e)),
                    }
                });
                true
            }
            Msg::MelodySaved(_) => {
                self.state.editing_melody = None;
                ctx.link().send_message(Msg::LoadMelodies);
                true
            }
            Msg::DeleteMelody(id) => {
                let link = ctx.link().clone();
                let id_clone = id.clone();
                spawn_local(async move {
                    match ApiClient::delete_melody(&id_clone).await {
                        Ok(()) => link.send_message(Msg::MelodyDeleted(id_clone)),
                        Err(e) => link.send_message(Msg::Error(e)),
                    }
                });
                true
            }
            Msg::MelodyDeleted(id) => {
                self.state.melodies.retain(|m| m.id != id);
                self.state.melody_audio_urls.remove(&id);
                if self.state.editing_melody.as_ref().is_some_and(|m| m.id == id) {
                    self.state.editing_melody = None;
                }
                true
            }
            Msg::GenerateMelodyAudio(id) => {
                self.state.generating_melody = Some(id.clone());
                let link = ctx.link().clone();
                spawn_local(async move {
                    match ApiClient::generate_melody_audio(&id).await {
                        Ok(response) => {
                            link.send_message(Msg::MelodyGenerationComplete(id, response.audio_url))
                        }
                        Err(e) => {
                            link.send_message(Msg::Error(e));
                            link.send_message(Msg::MelodyGenerationComplete(id, String::new()));
                        }
                    }
                });
                true
            }
            Msg::MelodyGenerationComplete(id, audio_url) => {
                self.state.generating_melody = None;
                if !audio_url.is_empty() {
                    self.state.melody_audio_urls.insert(id, audio_url);
                }
                true
            }

            // Common handlers
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
        let on_clear_error = ctx.link().callback(|_| Msg::ClearError);
        let active_tab = self.state.active_tab.unwrap_or(Tab::Presets);

        let on_tab_presets = ctx.link().callback(|_| Msg::SwitchTab(Tab::Presets));
        let on_tab_melodies = ctx.link().callback(|_| Msg::SwitchTab(Tab::Melodies));

        html! {
            <>
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
                        <span class="subtitle">{"AI Music Studio"}</span>
                    </header>

                    <div class="tabs">
                        <button
                            class={if active_tab == Tab::Presets { "tab active" } else { "tab" }}
                            onclick={on_tab_presets}
                        >
                            {"Presets"}
                        </button>
                        <button
                            class={if active_tab == Tab::Melodies { "tab active" } else { "tab" }}
                            onclick={on_tab_melodies}
                        >
                            {"Melodies"}
                        </button>
                    </div>

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

                    { match active_tab {
                        Tab::Presets => self.view_presets_tab(ctx),
                        Tab::Melodies => self.view_melodies_tab(ctx),
                    }}
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
                                    &version_info::BUILD_TIME[..10]
                                )}
                            </span>
                        </div>
                    </div>
                </footer>
            </>
        }
    }
}

impl App {
    fn view_presets_tab(&self, ctx: &Context<Self>) -> Html {
        let on_save = ctx.link().callback(Msg::SavePreset);
        let on_clear = ctx.link().callback(|_| Msg::ClearPresetEditor);
        let on_edit = ctx.link().callback(Msg::EditPreset);
        let on_delete = ctx.link().callback(Msg::DeletePreset);
        let on_generate = ctx.link().callback(Msg::GeneratePresetAudio);

        html! {
            <main class="main-content">
                <PresetEditor
                    on_save={on_save}
                    editing={self.state.editing_preset.clone()}
                    on_clear={on_clear}
                />
                <PresetList
                    presets={self.state.presets.clone()}
                    on_edit={on_edit}
                    on_delete={on_delete}
                    on_generate={on_generate}
                    generating={self.state.generating_preset.clone()}
                    audio_urls={self.state.preset_audio_urls.clone()}
                />
            </main>
        }
    }

    fn view_melodies_tab(&self, ctx: &Context<Self>) -> Html {
        let on_save = ctx.link().callback(Msg::SaveMelody);
        let on_clear = ctx.link().callback(|_| Msg::ClearMelodyEditor);
        let on_edit = ctx.link().callback(Msg::EditMelody);
        let on_delete = ctx.link().callback(Msg::DeleteMelody);
        let on_generate = ctx.link().callback(Msg::GenerateMelodyAudio);

        html! {
            <main class="main-content">
                <MelodyEditor
                    on_save={on_save}
                    editing={self.state.editing_melody.clone()}
                    on_clear={on_clear}
                />
                <MelodyList
                    melodies={self.state.melodies.clone()}
                    on_edit={on_edit}
                    on_delete={on_delete}
                    on_generate={on_generate}
                    generating={self.state.generating_melody.clone()}
                    audio_urls={self.state.melody_audio_urls.clone()}
                />
            </main>
        }
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
