//! Preset list component showing saved presets.

use crate::api::SavedPreset;
use crate::components::AudioPlayer;
use yew::prelude::*;

/// Props for the PresetList component.
#[derive(Properties, PartialEq)]
pub struct PresetListProps {
    /// List of saved presets.
    pub presets: Vec<SavedPreset>,
    /// Callback when edit is clicked.
    pub on_edit: Callback<SavedPreset>,
    /// Callback when delete is clicked.
    pub on_delete: Callback<String>,
    /// Callback when generate is clicked.
    pub on_generate: Callback<String>,
    /// ID of preset currently being generated (if any).
    pub generating: Option<String>,
    /// Map of preset IDs to their generated audio URLs.
    pub audio_urls: std::collections::HashMap<String, String>,
}

/// Component displaying the list of saved presets.
#[function_component(PresetList)]
pub fn preset_list(props: &PresetListProps) -> Html {
    if props.presets.is_empty() {
        return html! {
            <div class="card">
                <h2>{"Saved Presets"}</h2>
                <div class="empty-state">
                    <p>{"No presets saved yet."}</p>
                    <p>{"Create one using the form!"}</p>
                </div>
            </div>
        };
    }

    html! {
        <div class="card">
            <h2>{"Saved Presets"}</h2>
            <div class="preset-list">
                { for props.presets.iter().map(|preset| {
                    let preset_id = preset.id.clone();
                    let preset_for_edit = preset.clone();
                    let is_generating = props.generating.as_ref() == Some(&preset_id);
                    let audio_url = props.audio_urls.get(&preset_id).cloned();

                    let on_edit = {
                        let on_edit = props.on_edit.clone();
                        Callback::from(move |_| on_edit.emit(preset_for_edit.clone()))
                    };

                    let on_delete = {
                        let on_delete = props.on_delete.clone();
                        let id = preset_id.clone();
                        Callback::from(move |_| on_delete.emit(id.clone()))
                    };

                    let on_generate = {
                        let on_generate = props.on_generate.clone();
                        let id = preset_id.clone();
                        Callback::from(move |_| on_generate.emit(id.clone()))
                    };

                    html! {
                        <div class="preset-item" key={preset_id.clone()}>
                            <div class="preset-item-header">
                                <span class="preset-item-name">{&preset.name}</span>
                                <span class="preset-item-mood">{&preset.mood}</span>
                            </div>
                            <div class="preset-item-details">
                                {format!(
                                    "{:.1}s | {} BPM | Intensity {} | Seed {}",
                                    preset.duration,
                                    preset.tempo,
                                    preset.intensity,
                                    preset.seed
                                )}
                                { if let Some(ref key) = preset.key {
                                    html! { <>{format!(" | Key {}", key)}</> }
                                } else {
                                    html! {}
                                }}
                            </div>
                            <div class="preset-item-actions">
                                <button
                                    class="btn-primary btn-small"
                                    onclick={on_generate}
                                    disabled={is_generating}
                                >
                                    { if is_generating {
                                        html! { <span class="loading"></span> }
                                    } else {
                                        html! { "Generate" }
                                    }}
                                </button>
                                <button class="btn-secondary btn-small" onclick={on_edit}>
                                    {"Edit"}
                                </button>
                                <button class="btn-danger btn-small" onclick={on_delete}>
                                    {"Delete"}
                                </button>
                            </div>
                            { if let Some(url) = audio_url {
                                html! { <AudioPlayer src={url} /> }
                            } else {
                                html! {
                                    <div class="audio-player audio-player-disabled" title="Click Generate to create audio">
                                        <audio controls=true disabled=true>
                                            {"Audio not generated"}
                                        </audio>
                                    </div>
                                }
                            }}
                        </div>
                    }
                })}
            </div>
        </div>
    }
}
