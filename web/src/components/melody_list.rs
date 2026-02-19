//! Melody list component showing saved melodies.

use crate::api::SavedMelody;
use crate::components::AudioPlayer;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct MelodyListProps {
    pub melodies: Vec<SavedMelody>,
    pub on_edit: Callback<SavedMelody>,
    pub on_delete: Callback<String>,
    pub on_generate: Callback<String>,
    pub generating: Option<String>,
    pub audio_urls: std::collections::HashMap<String, String>,
}

#[function_component(MelodyList)]
pub fn melody_list(props: &MelodyListProps) -> Html {
    if props.melodies.is_empty() {
        return html! {
            <div class="card">
                <h2>{"Saved Melodies"}</h2>
                <div class="empty-state">
                    <p>{"No melodies saved yet."}</p>
                    <p>{"Create one using the editor!"}</p>
                </div>
            </div>
        };
    }

    html! {
        <div class="card">
            <h2>{"Saved Melodies"}</h2>
            <div class="preset-list">
                { for props.melodies.iter().map(|melody| {
                    let melody_id = melody.id.clone();
                    let melody_for_edit = melody.clone();
                    let is_generating = props.generating.as_ref() == Some(&melody_id);
                    let audio_url = props.audio_urls.get(&melody_id).cloned();

                    let on_edit = {
                        let on_edit = props.on_edit.clone();
                        Callback::from(move |_| on_edit.emit(melody_for_edit.clone()))
                    };

                    let on_delete = {
                        let on_delete = props.on_delete.clone();
                        let id = melody_id.clone();
                        Callback::from(move |_| on_delete.emit(id.clone()))
                    };

                    let on_generate = {
                        let on_generate = props.on_generate.clone();
                        let id = melody_id.clone();
                        Callback::from(move |_| on_generate.emit(id.clone()))
                    };

                    let note_preview: String = melody.notes
                        .iter()
                        .take(8)
                        .map(|n| if n.pitch == "rest" { "-".to_string() } else { n.pitch.clone() })
                        .collect::<Vec<_>>()
                        .join(" ");
                    let note_count = melody.notes.len();

                    html! {
                        <div class="preset-item" key={melody_id.clone()}>
                            <div class="preset-item-header">
                                <span class="preset-item-name">{&melody.name}</span>
                                <span class="preset-item-mood">{&melody.instrument}</span>
                            </div>
                            <div class="preset-item-details">
                                {format!("{} notes | {} BPM | Key {}", note_count, melody.tempo, melody.key)}
                            </div>
                            <div class="melody-notes-preview">
                                {note_preview}
                                { if note_count > 8 { "..." } else { "" } }
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
