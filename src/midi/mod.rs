//! MIDI generation module
//!
//! Provides note representation, sequence building, and MIDI file output.

pub mod note;
pub mod sequence;
pub mod writer;

pub use note::Note;
pub use sequence::NoteSequence;
pub use writer::write_midi;
