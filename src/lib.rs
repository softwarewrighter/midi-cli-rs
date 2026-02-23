//! midi-cli-rs: MIDI generation library for AI coding agents
//!
//! This library provides programmatic MIDI generation with support for
//! note sequences, instrument selection, and mood presets.

pub mod midi;
pub mod preset;
#[cfg(feature = "server")]
pub mod server;

pub use midi::note::NoteError;
pub use midi::sequence::{
    INSTRUMENT_MAP, JsonNoteInput, JsonSequenceInput, JsonTrackInput, resolve_instrument,
};
pub use midi::writer::{MidiWriteError, write_midi, write_midi_single};
pub use midi::{Note, NoteSequence};
pub use preset::{Key, Mood, MoodGenerator, PresetConfig, generate_mood};

// Re-export plugin-related types and functions for CLI use
#[cfg(feature = "server")]
pub use server::api::{get_moods_dir, lookup_plugin_mood, PluginMoodInfo};
