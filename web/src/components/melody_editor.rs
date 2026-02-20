//! Melody editor component with keyboard-driven note editing.

use crate::api::{MelodyNote, MelodyRequest, SavedMelody};
use wasm_bindgen::JsCast;
use web_sys::{HtmlInputElement, KeyboardEvent};
use yew::prelude::*;

const KEYS: &[&str] = &[
    "C", "Cm", "D", "Dm", "Eb", "E", "Em", "F", "Fm", "G", "Gm", "A", "Am", "Bb", "B", "Bm",
];

/// Instruments with default octaves - must match names in INSTRUMENT_MAP (src/midi/sequence.rs)
const INSTRUMENTS: &[(&str, u8)] = &[
    // Pianos (octave 4)
    ("piano", 4),
    ("acoustic_piano", 4),
    ("bright_piano", 4),
    ("electric_piano", 4),
    // Strings (varied octaves)
    ("strings", 4),
    ("violin", 4),
    ("viola", 4),
    ("cello", 3),
    ("contrabass", 2),
    ("tremolo_strings", 4),
    ("pizzicato_strings", 4),
    ("harp", 4),
    // Woodwinds (octave 4-5)
    ("flute", 5),
    ("oboe", 4),
    ("clarinet", 4),
    ("bassoon", 2),
    // Brass (octave 3-4)
    ("trumpet", 4),
    ("trombone", 3),
    ("french_horn", 3),
    ("tuba", 2),
    // Synth (octave 3-4)
    ("synth_pad", 3),
    ("synth_lead", 4),
    ("pad_warm", 3),
    ("pad_choir", 3),
    // Ambient (octave 3-4)
    ("atmosphere", 3),
    ("soundtrack", 4),
    // Guitar/Bass (octave 2-3)
    ("acoustic_guitar", 3),
    ("electric_guitar", 3),
    ("bass", 2),
    ("electric_bass", 2),
    // Bells/Percussion (octave 4-5)
    ("vibraphone", 4),
    ("marimba", 4),
    ("xylophone", 5),
    ("tubular_bells", 4),
    ("glockenspiel", 5),
    ("celesta", 5),
];

const DURATIONS: &[(f64, &str)] = &[
    (0.25, "1/16"),
    (0.5, "1/8"),
    (0.75, "3/16"),
    (1.0, "1/4"),
    (1.5, "3/8"),
    (2.0, "1/2"),
    (3.0, "3/4"),
    (4.0, "1"),
];

#[derive(Properties, PartialEq)]
pub struct MelodyEditorProps {
    pub on_save: Callback<MelodyRequest>,
    pub editing: Option<SavedMelody>,
    pub on_clear: Callback<()>,
}

#[derive(Clone, Debug)]
struct EditorState {
    name: String,
    notes: Vec<MelodyNote>,
    key: String,
    tempo: u16,
    instrument: String,
    attack: u8,
    decay: u8,
    selected_note: usize,
    insert_mode: bool,
    undo_stack: Vec<Vec<MelodyNote>>,
    redo_stack: Vec<Vec<MelodyNote>>,
}

impl Default for EditorState {
    fn default() -> Self {
        let instrument = "piano".to_string();
        let octave = default_octave_for_instrument(&instrument);
        Self {
            name: String::new(),
            notes: vec![MelodyNote {
                pitch: format!("C{}", octave),
                duration: 1.0,
                velocity: 80,
            }],
            key: "C".to_string(),
            tempo: 120,
            instrument,
            attack: 0,
            decay: 64,
            selected_note: 0,
            insert_mode: false,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }
}

impl EditorState {
    /// Create a new note with the appropriate octave for the current instrument.
    fn new_note(&self) -> MelodyNote {
        let octave = default_octave_for_instrument(&self.instrument);
        MelodyNote {
            pitch: format!("C{}", octave),
            duration: 1.0,
            velocity: 80,
        }
    }

    fn from_melody(melody: &SavedMelody) -> Self {
        let octave = default_octave_for_instrument(&melody.instrument);
        Self {
            name: melody.name.clone(),
            notes: if melody.notes.is_empty() {
                vec![MelodyNote {
                    pitch: format!("C{}", octave),
                    duration: 1.0,
                    velocity: 80,
                }]
            } else {
                melody.notes.clone()
            },
            key: melody.key.clone(),
            tempo: melody.tempo,
            instrument: melody.instrument.clone(),
            attack: melody.attack,
            decay: melody.decay,
            selected_note: 0,
            insert_mode: false,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    fn to_request(&self) -> MelodyRequest {
        MelodyRequest {
            name: self.name.clone(),
            notes: self.notes.clone(),
            key: self.key.clone(),
            tempo: self.tempo,
            instrument: self.instrument.clone(),
            attack: self.attack,
            decay: self.decay,
        }
    }

    fn push_undo(&mut self) {
        self.undo_stack.push(self.notes.clone());
        self.redo_stack.clear();
        // Limit undo stack size
        if self.undo_stack.len() > 50 {
            self.undo_stack.remove(0);
        }
    }

    fn undo(&mut self) {
        if let Some(prev) = self.undo_stack.pop() {
            self.redo_stack.push(self.notes.clone());
            self.notes = prev;
            self.selected_note = self.selected_note.min(self.notes.len().saturating_sub(1));
        }
    }

    fn redo(&mut self) {
        if let Some(next) = self.redo_stack.pop() {
            self.undo_stack.push(self.notes.clone());
            self.notes = next;
            self.selected_note = self.selected_note.min(self.notes.len().saturating_sub(1));
        }
    }
}

#[function_component(MelodyEditor)]
pub fn melody_editor(props: &MelodyEditorProps) -> Html {
    let state = use_state(|| {
        props
            .editing
            .as_ref()
            .map(EditorState::from_melody)
            .unwrap_or_default()
    });
    let note_grid_focused = use_state(|| false);

    // Update state when editing prop changes
    {
        let state = state.clone();
        let editing = props.editing.clone();
        use_effect_with(editing, move |editing| {
            state.set(
                editing
                    .as_ref()
                    .map(EditorState::from_melody)
                    .unwrap_or_default(),
            );
        });
    }

    let on_keydown = {
        let state = state.clone();
        Callback::from(move |e: KeyboardEvent| {
            // Skip handling if event originated from an input or select element
            if let Some(target) = e.target() {
                if let Ok(element) = target.dyn_into::<web_sys::Element>() {
                    let tag = element.tag_name().to_uppercase();
                    if tag == "INPUT" || tag == "SELECT" || tag == "TEXTAREA" {
                        return;
                    }
                }
            }

            let key = e.key();
            let shift = e.shift_key();
            let ctrl = e.ctrl_key() || e.meta_key();
            let mut s = (*state).clone();

            match key.as_str() {
                // Navigation
                "Tab" if !shift => {
                    e.prevent_default();
                    if s.selected_note < s.notes.len() - 1 {
                        s.selected_note += 1;
                    }
                }
                "Tab" if shift => {
                    e.prevent_default();
                    if s.selected_note > 0 {
                        s.selected_note -= 1;
                    }
                }
                "ArrowRight" => {
                    e.prevent_default();
                    if s.selected_note < s.notes.len() - 1 {
                        s.selected_note += 1;
                    }
                }
                "ArrowLeft" => {
                    e.prevent_default();
                    if s.selected_note > 0 {
                        s.selected_note -= 1;
                    }
                }

                // Note input (a-g)
                "a" | "b" | "c" | "d" | "e" | "f" | "g" | "A" | "B" | "C" | "D" | "E" | "F"
                | "G" => {
                    e.prevent_default();
                    s.push_undo();
                    let note_name = key.to_uppercase();
                    let octave = if s.notes.get(s.selected_note).is_some() {
                        extract_octave(&s.notes[s.selected_note].pitch)
                    } else {
                        4
                    };
                    let new_pitch = format!("{}{}", note_name, octave);
                    if s.insert_mode {
                        s.notes.insert(
                            s.selected_note + 1,
                            MelodyNote {
                                pitch: new_pitch,
                                duration: 1.0,
                                velocity: 80,
                            },
                        );
                        s.selected_note += 1;
                    } else if let Some(note) = s.notes.get_mut(s.selected_note) {
                        note.pitch = new_pitch;
                    }
                }

                // Rest
                "r" | "R" => {
                    e.prevent_default();
                    s.push_undo();
                    if s.insert_mode {
                        s.notes.insert(s.selected_note + 1, MelodyNote::rest(1.0));
                        s.selected_note += 1;
                    } else if let Some(note) = s.notes.get_mut(s.selected_note) {
                        *note = MelodyNote::rest(note.duration);
                    }
                }

                // Octave up/down
                "=" | "+" => {
                    e.prevent_default();
                    s.push_undo();
                    if let Some(note) = s.notes.get_mut(s.selected_note) {
                        if !note.is_rest() {
                            let octave = extract_octave(&note.pitch);
                            if octave < 8 {
                                note.pitch = set_octave(&note.pitch, octave + 1);
                            }
                        }
                    }
                }
                "-" | "_" => {
                    e.prevent_default();
                    s.push_undo();
                    if let Some(note) = s.notes.get_mut(s.selected_note) {
                        if !note.is_rest() {
                            let octave = extract_octave(&note.pitch);
                            if octave > 0 {
                                note.pitch = set_octave(&note.pitch, octave - 1);
                            }
                        }
                    }
                }

                // Scale movement (up/down arrow with shift)
                "ArrowUp" if shift => {
                    e.prevent_default();
                    s.push_undo();
                    if let Some(note) = s.notes.get_mut(s.selected_note) {
                        if !note.is_rest() {
                            note.pitch = move_scale_step(&note.pitch, 1, &s.key);
                        }
                    }
                }
                "ArrowDown" if shift => {
                    e.prevent_default();
                    s.push_undo();
                    if let Some(note) = s.notes.get_mut(s.selected_note) {
                        if !note.is_rest() {
                            note.pitch = move_scale_step(&note.pitch, -1, &s.key);
                        }
                    }
                }

                // Velocity adjustment
                "ArrowUp" if !shift => {
                    e.prevent_default();
                    s.push_undo();
                    if let Some(note) = s.notes.get_mut(s.selected_note) {
                        note.velocity = (note.velocity + 10).min(127);
                    }
                }
                "ArrowDown" if !shift => {
                    e.prevent_default();
                    s.push_undo();
                    if let Some(note) = s.notes.get_mut(s.selected_note) {
                        note.velocity = note.velocity.saturating_sub(10);
                    }
                }

                // Duration ([ and ])
                "[" => {
                    e.prevent_default();
                    s.push_undo();
                    if let Some(note) = s.notes.get_mut(s.selected_note) {
                        note.duration = prev_duration(note.duration);
                    }
                }
                "]" => {
                    e.prevent_default();
                    s.push_undo();
                    if let Some(note) = s.notes.get_mut(s.selected_note) {
                        note.duration = next_duration(note.duration);
                    }
                }

                // Delete
                "Delete" | "Backspace" => {
                    e.prevent_default();
                    if s.notes.len() > 1 {
                        s.push_undo();
                        s.notes.remove(s.selected_note);
                        if s.selected_note >= s.notes.len() {
                            s.selected_note = s.notes.len() - 1;
                        }
                    }
                }

                // Insert mode toggle
                "i" | "I" if !ctrl => {
                    e.prevent_default();
                    s.insert_mode = !s.insert_mode;
                }

                // Undo/Redo
                "z" | "Z" if ctrl && !shift => {
                    e.prevent_default();
                    s.undo();
                }
                "z" | "Z" if ctrl && shift => {
                    e.prevent_default();
                    s.redo();
                }
                "y" | "Y" if ctrl => {
                    e.prevent_default();
                    s.redo();
                }

                // Add note at end
                "Enter" => {
                    e.prevent_default();
                    s.push_undo();
                    s.notes.push(s.new_note());
                    s.selected_note = s.notes.len() - 1;
                }

                // Escape to blur/exit note editor
                "Escape" => {
                    e.prevent_default();
                    if let Some(target) = e.target() {
                        if let Ok(element) = target.dyn_into::<web_sys::HtmlElement>() {
                            let _ = element.blur();
                        }
                    }
                }

                _ => {}
            }

            state.set(s);
        })
    };

    let on_name_change = {
        let state = state.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            let mut s = (*state).clone();
            s.name = input.value();
            state.set(s);
        })
    };

    let on_key_change = {
        let state = state.clone();
        Callback::from(move |e: Event| {
            let select: web_sys::HtmlSelectElement = e.target_unchecked_into();
            let mut s = (*state).clone();
            s.key = select.value();
            state.set(s);
        })
    };

    let on_tempo_change = {
        let state = state.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            let mut s = (*state).clone();
            s.tempo = input.value().parse().unwrap_or(120);
            state.set(s);
        })
    };

    let on_instrument_change = {
        let state = state.clone();
        Callback::from(move |e: Event| {
            let select: web_sys::HtmlSelectElement = e.target_unchecked_into();
            let mut s = (*state).clone();
            let old_octave = default_octave_for_instrument(&s.instrument);
            let new_instrument = select.value();
            let new_octave = default_octave_for_instrument(&new_instrument);

            // Transpose all notes to the new instrument's range
            if old_octave != new_octave {
                let shift = new_octave as i8 - old_octave as i8;
                for note in &mut s.notes {
                    if !note.is_rest() {
                        let current_octave = extract_octave(&note.pitch) as i8;
                        let target_octave = (current_octave + shift).clamp(0, 8) as u8;
                        note.pitch = set_octave(&note.pitch, target_octave);
                    }
                }
            }

            s.instrument = new_instrument;
            state.set(s);
        })
    };

    let on_attack_change = {
        let state = state.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            let mut s = (*state).clone();
            s.attack = input.value().parse().unwrap_or(0);
            state.set(s);
        })
    };

    let on_decay_change = {
        let state = state.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            let mut s = (*state).clone();
            s.decay = input.value().parse().unwrap_or(64);
            state.set(s);
        })
    };

    let on_note_click = {
        let state = state.clone();
        move |idx: usize| {
            let state = state.clone();
            Callback::from(move |_| {
                let mut s = (*state).clone();
                s.selected_note = idx;
                state.set(s);
            })
        }
    };

    let on_submit = {
        let state = state.clone();
        let on_save = props.on_save.clone();
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            on_save.emit(state.to_request());
        })
    };

    let on_clear = {
        let on_clear = props.on_clear.clone();
        Callback::from(move |_| on_clear.emit(()))
    };

    let on_grid_focus = {
        let note_grid_focused = note_grid_focused.clone();
        Callback::from(move |_: FocusEvent| {
            note_grid_focused.set(true);
        })
    };

    let on_grid_blur = {
        let note_grid_focused = note_grid_focused.clone();
        Callback::from(move |_: FocusEvent| {
            note_grid_focused.set(false);
        })
    };

    let is_editing = props.editing.is_some();
    let grid_focused = *note_grid_focused;

    let grid_class = if grid_focused {
        "note-grid note-grid-focused"
    } else {
        "note-grid"
    };

    html! {
        <div class="card melody-editor">
            <h2>{ if is_editing { "Edit Melody" } else { "New Melody" } }</h2>

            { if grid_focused {
                html! {
                    <div class="mode-indicator mode-note-editing">
                        {"NOTE EDITING MODE"}
                        <small>{" — Press Esc to exit"}</small>
                    </div>
                }
            } else {
                html! {
                    <div class="mode-indicator mode-form">
                        {"FORM MODE"}
                        <small>{" — Click notes to edit"}</small>
                    </div>
                }
            }}

            { if grid_focused {
                html! {
                    <div class="keyboard-hint">
                        <small>
                            {"Keys: "}
                            <kbd>{"a-g"}</kbd>{" note | "}
                            <kbd>{"r"}</kbd>{" rest | "}
                            <kbd>{"Tab"}</kbd>{" next | "}
                            <kbd>{"+/-"}</kbd>{" octave | "}
                            <kbd>{"[/]"}</kbd>{" duration | "}
                            <kbd>{"↑↓"}</kbd>{" velocity | "}
                            <kbd>{"Shift+↑↓"}</kbd>{" scale | "}
                            <kbd>{"i"}</kbd>{" insert | "}
                            <kbd>{"Del"}</kbd>{" delete | "}
                            <kbd>{"Ctrl+Z/Y"}</kbd>{" undo/redo | "}
                            <kbd>{"Esc"}</kbd>{" exit"}
                        </small>
                    </div>
                }
            } else {
                html! {}
            }}

            <form onsubmit={on_submit}>
                <div class="form-group">
                    <label for="melody-name">{"Name"}</label>
                    <input
                        type="text"
                        id="melody-name"
                        value={state.name.clone()}
                        oninput={on_name_change}
                        placeholder="My Melody"
                        required=true
                    />
                </div>

                <div
                    class={grid_class}
                    tabindex="0"
                    onkeydown={on_keydown}
                    onfocus={on_grid_focus}
                    onblur={on_grid_blur}
                >
                    { for state.notes.iter().enumerate().map(|(idx, note)| {
                        let selected = idx == state.selected_note;
                        let class = if selected {
                            if state.insert_mode { "note-cell selected insert-mode" } else { "note-cell selected" }
                        } else {
                            "note-cell"
                        };
                        let is_rest = note.is_rest();

                        html! {
                            <div
                                class={class}
                                onclick={on_note_click(idx)}
                                title={format!("Velocity: {}", note.velocity)}
                            >
                                <div class="note-pitch">
                                    { if is_rest { "—".to_string() } else { note.pitch.clone() } }
                                </div>
                                <div class="note-duration">
                                    { duration_label(note.duration) }
                                </div>
                                <div class="note-velocity-bar" style={format!("width: {}%", note.velocity as f32 / 127.0 * 100.0)}></div>
                            </div>
                        }
                    })}
                </div>

                { if state.insert_mode {
                    html! { <div class="insert-mode-indicator">{"INSERT MODE"}</div> }
                } else {
                    html! {}
                }}

                <div class="form-row">
                    <div class="form-group">
                        <label for="melody-key">{"Key"}</label>
                        <select id="melody-key" onchange={on_key_change} value={state.key.clone()}>
                            { for KEYS.iter().map(|k| html! {
                                <option value={*k} selected={state.key == *k}>{k}</option>
                            })}
                        </select>
                    </div>

                    <div class="form-group">
                        <label for="melody-instrument">{"Instrument"}</label>
                        <select id="melody-instrument" onchange={on_instrument_change} value={state.instrument.clone()}>
                            { for INSTRUMENTS.iter().map(|(name, _)| html! {
                                <option value={*name} selected={state.instrument == *name}>{name}</option>
                            })}
                        </select>
                    </div>
                </div>

                <div class="form-row">
                    <div class="form-group">
                        <label for="melody-tempo">{"Tempo (BPM)"}</label>
                        <input
                            type="number"
                            id="melody-tempo"
                            min="40"
                            max="240"
                            value={state.tempo.to_string()}
                            oninput={on_tempo_change}
                        />
                    </div>

                    <div class="form-group">
                        <label for="melody-attack">{"Attack"}</label>
                        <input
                            type="range"
                            id="melody-attack"
                            min="0"
                            max="127"
                            value={state.attack.to_string()}
                            oninput={on_attack_change}
                        />
                    </div>

                    <div class="form-group">
                        <label for="melody-decay">{"Decay"}</label>
                        <input
                            type="range"
                            id="melody-decay"
                            min="0"
                            max="127"
                            value={state.decay.to_string()}
                            oninput={on_decay_change}
                        />
                    </div>
                </div>

                <div class="button-row">
                    <button type="submit" class="btn-primary">
                        { if is_editing { "Update Melody" } else { "Save Melody" } }
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

// Helper functions

fn extract_octave(pitch: &str) -> u8 {
    pitch
        .chars()
        .filter(|c| c.is_ascii_digit())
        .collect::<String>()
        .parse()
        .unwrap_or(4)
}

fn set_octave(pitch: &str, octave: u8) -> String {
    let note_part: String = pitch.chars().filter(|c| !c.is_ascii_digit()).collect();
    format!("{}{}", note_part, octave)
}

fn duration_label(duration: f64) -> &'static str {
    for (d, label) in DURATIONS {
        if (*d - duration).abs() < 0.01 {
            return label;
        }
    }
    "?"
}

fn prev_duration(current: f64) -> f64 {
    for (i, (d, _)) in DURATIONS.iter().enumerate() {
        if (*d - current).abs() < 0.01 && i > 0 {
            return DURATIONS[i - 1].0;
        }
    }
    current
}

fn next_duration(current: f64) -> f64 {
    for (i, (d, _)) in DURATIONS.iter().enumerate() {
        if (*d - current).abs() < 0.01 && i < DURATIONS.len() - 1 {
            return DURATIONS[i + 1].0;
        }
    }
    current
}

fn move_scale_step(pitch: &str, steps: i32, _key: &str) -> String {
    // Simplified chromatic movement for now
    let note_names = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
    let octave = extract_octave(pitch);
    let note_part: String = pitch.chars().filter(|c| !c.is_ascii_digit()).collect();

    let current_idx = note_names
        .iter()
        .position(|&n| n == note_part)
        .unwrap_or(0) as i32;

    let new_idx = (current_idx + steps).rem_euclid(12);
    let octave_adjust = (current_idx + steps) / 12;
    let new_octave = (octave as i32 + octave_adjust).clamp(0, 8) as u8;

    format!("{}{}", note_names[new_idx as usize], new_octave)
}

/// Get the default octave for an instrument (bass instruments play lower).
fn default_octave_for_instrument(instrument: &str) -> u8 {
    for (name, octave) in INSTRUMENTS {
        if *name == instrument {
            return *octave;
        }
    }
    4 // default to middle octave
}
