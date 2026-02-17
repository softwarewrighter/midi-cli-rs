//! midi-cli-rs: MIDI generation library for AI coding agents
//!
//! This library provides programmatic MIDI generation with support for
//! note sequences, instrument selection, and mood presets.

pub mod midi;

pub use midi::note::NoteError;
pub use midi::sequence::{
    INSTRUMENT_MAP, JsonNoteInput, JsonSequenceInput, JsonTrackInput, resolve_instrument,
};
pub use midi::writer::{MidiWriteError, write_midi, write_midi_single};
pub use midi::{Note, NoteSequence};
