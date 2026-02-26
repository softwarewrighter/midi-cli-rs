//! MIDI CLI Web UI - Yew frontend application.

mod api;
mod components;

mod version_info {
    include!(concat!(env!("OUT_DIR"), "/version_info.rs"));
}

use api::{AbcImportRequest, ApiClient, MelodyRequest, MoodPackInfo, PresetRequest, SavedMelody, SavedPreset};
use components::{AbcImport, MelodyEditor, MelodyList, PluginManager, PresetEditor, PresetList};
use std::collections::HashMap;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tab {
    Presets,
    Melodies,
    Plugins,
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
    // ABC Import
    abc_importing: bool,
    abc_import_error: Option<String>,
    // Plugins
    plugins: Vec<MoodPackInfo>,
    plugins_loading: bool,
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
    // ABC Import/Export
    ImportAbcMelody(AbcImportRequest),
    AbcImported(SavedMelody),
    AbcImportError(String),
    ExportMelodyAbc(String),
    // Plugins
    LoadPlugins,
    PluginsLoaded(Vec<MoodPackInfo>),
    UploadPlugin(String),
    PluginUploaded(MoodPackInfo),
    DeletePlugin(String),
    PluginDeleted(String),
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
        ctx.link().send_message(Msg::LoadPlugins);
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
            Msg::PresetSaved(preset) => {
                // Clear audio URL - user must regenerate after edits
                self.state.preset_audio_urls.remove(&preset.id);
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
            Msg::MelodySaved(melody) => {
                // Clear audio URL - user must regenerate after edits
                self.state.melody_audio_urls.remove(&melody.id);
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

            // ABC Import/Export handlers
            Msg::ImportAbcMelody(req) => {
                self.state.abc_importing = true;
                self.state.abc_import_error = None;
                let link = ctx.link().clone();
                spawn_local(async move {
                    match ApiClient::import_abc_melody(&req).await {
                        Ok(melody) => link.send_message(Msg::AbcImported(melody)),
                        Err(e) => link.send_message(Msg::AbcImportError(e)),
                    }
                });
                true
            }
            Msg::AbcImported(melody) => {
                self.state.abc_importing = false;
                self.state.abc_import_error = None;
                self.state.melodies.insert(0, melody);
                true
            }
            Msg::AbcImportError(error) => {
                self.state.abc_importing = false;
                self.state.abc_import_error = Some(error);
                true
            }
            Msg::ExportMelodyAbc(id) => {
                let link = ctx.link().clone();
                spawn_local(async move {
                    match ApiClient::export_melody_abc(&id).await {
                        Ok(abc) => {
                            // Copy to clipboard
                            if let Some(window) = web_sys::window() {
                                let clipboard = window.navigator().clipboard();
                                let _ = clipboard.write_text(&abc);
                            }
                        }
                        Err(e) => link.send_message(Msg::Error(e)),
                    }
                });
                true
            }

            // Plugin handlers
            Msg::LoadPlugins => {
                self.state.plugins_loading = true;
                let link = ctx.link().clone();
                spawn_local(async move {
                    match ApiClient::list_plugins().await {
                        Ok(plugins) => link.send_message(Msg::PluginsLoaded(plugins)),
                        Err(e) => {
                            link.send_message(Msg::PluginsLoaded(vec![]));
                            link.send_message(Msg::Error(e));
                        }
                    }
                });
                true
            }
            Msg::PluginsLoaded(plugins) => {
                self.state.plugins = plugins;
                self.state.plugins_loading = false;
                true
            }
            Msg::UploadPlugin(content) => {
                let link = ctx.link().clone();
                spawn_local(async move {
                    match ApiClient::upload_plugin(&content).await {
                        Ok(plugin) => link.send_message(Msg::PluginUploaded(plugin)),
                        Err(e) => link.send_message(Msg::Error(e)),
                    }
                });
                true
            }
            Msg::PluginUploaded(_plugin) => {
                ctx.link().send_message(Msg::LoadPlugins);
                true
            }
            Msg::DeletePlugin(name) => {
                let link = ctx.link().clone();
                let name_clone = name.clone();
                spawn_local(async move {
                    match ApiClient::delete_plugin(&name_clone).await {
                        Ok(()) => link.send_message(Msg::PluginDeleted(name_clone)),
                        Err(e) => link.send_message(Msg::Error(e)),
                    }
                });
                true
            }
            Msg::PluginDeleted(name) => {
                self.state.plugins.retain(|p| p.name != name);
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
        let on_tab_plugins = ctx.link().callback(|_| Msg::SwitchTab(Tab::Plugins));

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
                        <button
                            class={if active_tab == Tab::Plugins { "tab active" } else { "tab" }}
                            onclick={on_tab_plugins}
                        >
                            {"Plugins"}
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
                        Tab::Plugins => self.view_plugins_tab(ctx),
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
        let on_export_abc = ctx.link().callback(Msg::ExportMelodyAbc);
        let on_abc_import = ctx.link().callback(Msg::ImportAbcMelody);
        let on_abc_imported = ctx.link().callback(Msg::AbcImported);

        html! {
            <main class="main-content">
                <AbcImport
                    on_import={on_abc_import}
                    on_imported={on_abc_imported}
                    importing={self.state.abc_importing}
                    error={self.state.abc_import_error.clone()}
                />
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
                    on_export_abc={on_export_abc}
                    generating={self.state.generating_melody.clone()}
                    audio_urls={self.state.melody_audio_urls.clone()}
                />
            </main>
        }
    }

    fn view_plugins_tab(&self, ctx: &Context<Self>) -> Html {
        let on_refresh = ctx.link().callback(|_| Msg::LoadPlugins);
        let on_delete = ctx.link().callback(Msg::DeletePlugin);
        let on_upload = ctx.link().callback(Msg::UploadPlugin);

        html! {
            <main class="main-content plugins-layout">
                <PluginManager
                    plugins={self.state.plugins.clone()}
                    on_refresh={on_refresh}
                    on_delete={on_delete}
                    on_upload={on_upload}
                    loading={self.state.plugins_loading}
                />
            </main>
        }
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
