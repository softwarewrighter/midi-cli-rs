//! Preset editor form component.

use crate::api::{PresetRequest, SavedPreset};
use web_sys::HtmlInputElement;
use yew::prelude::*;

/// Available musical keys for the dropdown.
const KEYS: &[&str] = &[
    "C", "Cm", "D", "Dm", "Eb", "E", "Em", "F", "Fm", "G", "Gm", "A", "Am", "Bb", "B", "Bm",
];

/// Available moods for the dropdown.
const MOODS: &[&str] = &["suspense", "eerie", "upbeat", "calm", "ambient", "jazz"];

/// Props for the PresetEditor component.
#[derive(Properties, PartialEq)]
pub struct PresetEditorProps {
    /// Callback when save is clicked.
    pub on_save: Callback<PresetRequest>,
    /// Current preset being edited (None for new preset).
    pub editing: Option<SavedPreset>,
    /// Callback to clear the editor.
    pub on_clear: Callback<()>,
}

/// Form state for editing a preset.
#[derive(Clone, Debug, Default)]
struct FormState {
    name: String,
    mood: String,
    duration: f64,
    key: String,
    intensity: u8,
    tempo: u16,
    seed: i64,
}

impl FormState {
    fn new() -> Self {
        Self {
            name: String::new(),
            mood: "jazz".to_string(),
            duration: 5.0,
            key: String::new(),
            intensity: 50,
            tempo: 90,
            seed: 0, // 0 = random (will generate actual seed on save)
        }
    }

    fn from_preset(preset: &SavedPreset) -> Self {
        Self {
            name: preset.name.clone(),
            mood: preset.mood.clone(),
            duration: preset.duration,
            key: preset.key.clone().unwrap_or_default(),
            intensity: preset.intensity,
            tempo: preset.tempo,
            seed: preset.seed,
        }
    }

    fn to_request(&self) -> PresetRequest {
        // If seed is 0 (random), generate an actual random seed for reproducibility
        let seed = if self.seed == 0 {
            generate_random_seed()
        } else {
            self.seed
        };

        PresetRequest {
            name: self.name.clone(),
            mood: self.mood.clone(),
            duration: self.duration,
            key: if self.key.is_empty() {
                None
            } else {
                Some(self.key.clone())
            },
            intensity: self.intensity,
            tempo: self.tempo,
            seed,
        }
    }
}

/// Generate a random seed using JavaScript's Math.random()
fn generate_random_seed() -> i64 {
    let window = web_sys::window().unwrap();
    let crypto = window.crypto().unwrap();
    let mut bytes = [0u8; 8];
    crypto.get_random_values_with_u8_array(&mut bytes).unwrap();
    // Use first 6 bytes for a reasonable seed range (positive i64)
    let seed = i64::from_le_bytes(bytes) & 0x7FFF_FFFF_FFFF_FFFF;
    seed.max(1) // Ensure non-zero
}

/// Preset editor form component.
#[function_component(PresetEditor)]
pub fn preset_editor(props: &PresetEditorProps) -> Html {
    let form = use_state(|| {
        props
            .editing
            .as_ref()
            .map(FormState::from_preset)
            .unwrap_or_else(FormState::new)
    });

    // Update form when editing prop changes
    {
        let form = form.clone();
        let editing = props.editing.clone();
        use_effect_with(editing, move |editing| {
            form.set(
                editing
                    .as_ref()
                    .map(FormState::from_preset)
                    .unwrap_or_else(FormState::new),
            );
        });
    }

    let on_name_change = {
        let form = form.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            let mut state = (*form).clone();
            state.name = input.value();
            form.set(state);
        })
    };

    let on_mood_change = {
        let form = form.clone();
        Callback::from(move |e: Event| {
            let select: web_sys::HtmlSelectElement = e.target_unchecked_into();
            let mut state = (*form).clone();
            state.mood = select.value();
            form.set(state);
        })
    };

    let on_duration_change = {
        let form = form.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            let mut state = (*form).clone();
            state.duration = input.value().parse().unwrap_or(5.0);
            form.set(state);
        })
    };

    let on_key_change = {
        let form = form.clone();
        Callback::from(move |e: Event| {
            let select: web_sys::HtmlSelectElement = e.target_unchecked_into();
            let mut state = (*form).clone();
            state.key = select.value();
            form.set(state);
        })
    };

    let on_intensity_change = {
        let form = form.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            let mut state = (*form).clone();
            state.intensity = input.value().parse().unwrap_or(50);
            form.set(state);
        })
    };

    let on_tempo_change = {
        let form = form.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            let mut state = (*form).clone();
            state.tempo = input.value().parse().unwrap_or(90);
            form.set(state);
        })
    };

    let on_seed_change = {
        let form = form.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            let mut state = (*form).clone();
            state.seed = input.value().parse().unwrap_or(1);
            form.set(state);
        })
    };

    let on_time_seed = {
        let form = form.clone();
        Callback::from(move |_: MouseEvent| {
            let mut state = (*form).clone();
            // Use milliseconds since epoch as seed
            let now = js_sys::Date::now() as i64;
            state.seed = now;
            form.set(state);
        })
    };

    let on_submit = {
        let form = form.clone();
        let on_save = props.on_save.clone();
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            on_save.emit(form.to_request());
        })
    };

    let on_clear = {
        let on_clear = props.on_clear.clone();
        Callback::from(move |_| {
            on_clear.emit(());
        })
    };

    let is_editing = props.editing.is_some();

    html! {
        <div class="card">
            <h2>{ if is_editing { "Edit Preset" } else { "New Preset" } }</h2>
            <form onsubmit={on_submit}>
                <div class="form-group">
                    <label for="name">{"Name"}</label>
                    <input
                        type="text"
                        id="name"
                        value={form.name.clone()}
                        oninput={on_name_change}
                        placeholder="My Cool Preset"
                        required=true
                    />
                </div>

                <div class="form-row">
                    <div class="form-group">
                        <label for="mood">{"Mood"}</label>
                        <select id="mood" key={form.mood.clone()} onchange={on_mood_change}>
                            { for MOODS.iter().map(|m| {
                                let is_selected = form.mood == *m;
                                html! {
                                    <option value={*m} selected={is_selected}>{m}</option>
                                }
                            })}
                        </select>
                    </div>

                    <div class="form-group">
                        <label for="key">{"Key (optional)"}</label>
                        <select id="key" key={form.key.clone()} onchange={on_key_change}>
                            <option value="" selected={form.key.is_empty()}>{"Default"}</option>
                            { for KEYS.iter().map(|k| {
                                let is_selected = form.key == *k;
                                html! {
                                    <option value={*k} selected={is_selected}>{k}</option>
                                }
                            })}
                        </select>
                    </div>
                </div>

                <div class="form-group">
                    <label for="duration">{"Duration (seconds)"}</label>
                    <div class="slider-group">
                        <input
                            type="range"
                            id="duration"
                            min="1"
                            max="30"
                            step="0.5"
                            value={form.duration.to_string()}
                            oninput={on_duration_change}
                        />
                        <span class="slider-value">{format!("{:.1}s", form.duration)}</span>
                    </div>
                </div>

                <div class="form-group">
                    <label for="intensity">{"Intensity"}</label>
                    <div class="slider-group">
                        <input
                            type="range"
                            id="intensity"
                            min="0"
                            max="100"
                            value={form.intensity.to_string()}
                            oninput={on_intensity_change}
                        />
                        <span class="slider-value">{form.intensity}</span>
                    </div>
                </div>

                <div class="form-row">
                    <div class="form-group">
                        <label for="tempo">{"Tempo (BPM)"}</label>
                        <input
                            type="number"
                            id="tempo"
                            min="40"
                            max="200"
                            value={form.tempo.to_string()}
                            oninput={on_tempo_change}
                        />
                    </div>

                    <div class="form-group">
                        <label for="seed">{"Seed"}</label>
                        <div class="seed-input-row">
                            <input
                                type="number"
                                id="seed"
                                value={form.seed.to_string()}
                                oninput={on_seed_change}
                            />
                            <button
                                type="button"
                                class="btn-icon seed-now-btn"
                                onclick={on_time_seed}
                                title="Time-based seed"
                            >
                                {"🕐"}
                            </button>
                        </div>
                    </div>
                </div>

                <div class="button-row">
                    <button type="submit" class="btn-primary">
                        { if is_editing { "Update Preset" } else { "Save Preset" } }
                    </button>
                    { if is_editing {
                        html! {
                            <button type="button" class="btn-secondary" onclick={on_clear}>
                                {"Cancel"}
                            </button>
                        }
                    } else {
                        html! {}
                    }}
                </div>
            </form>
        </div>
    }
}
