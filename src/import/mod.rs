//! Melody import from ABC notation and MusicXML files
//!
//! This module provides parsers for importing melodies from common music notation formats:
//! - ABC notation (.abc) - Simple text-based format used by folk music archives
//! - MusicXML (.musicxml, .mxl) - Standard interchange format for music notation
//!
//! # Example
//!
//! ```ignore
//! use midi_cli_rs::import::{AbcParser, MelodyImporter};
//!
//! let melody = AbcParser::parse_file("tune.abc")?;
//! let sequences = melody.to_sequences("piano", 120)?;
//! ```

mod abc;
mod error;
mod musicxml;

pub use abc::AbcParser;
pub use error::ImportError;
pub use musicxml::MusicXmlParser;

use crate::midi::sequence::{resolve_instrument, NoteSequence};
use crate::midi::Note;

/// A single imported note with optional rest support
#[derive(Debug, Clone)]
pub struct ImportedNote {
    /// MIDI pitch (0-127), None indicates a rest
    pub pitch: Option<u8>,
    /// Duration in beats (quarter note = 1.0)
    pub duration: f64,
    /// Velocity (0-127), default 80
    pub velocity: u8,
    /// Start time in beats from sequence start
    pub offset: f64,
}

impl ImportedNote {
    /// Create a new note
    pub fn new(pitch: u8, duration: f64, offset: f64) -> Self {
        Self {
            pitch: Some(pitch),
            duration,
            velocity: 80,
            offset,
        }
    }

    /// Create a rest
    pub fn rest(duration: f64, offset: f64) -> Self {
        Self {
            pitch: None,
            duration,
            velocity: 0,
            offset,
        }
    }
}

/// Imported melody with metadata
#[derive(Debug, Clone)]
pub struct ImportedMelody {
    /// Notes in the melody
    pub notes: Vec<ImportedNote>,
    /// Musical key (e.g., "G", "Am", "F#m")
    pub key: Option<String>,
    /// Tempo in BPM if specified in the file
    pub tempo: Option<u16>,
    /// Time signature (numerator, denominator)
    pub time_signature: (u8, u8),
    /// Title from the file
    pub title: Option<String>,
}

impl ImportedMelody {
    /// Create a new empty melody
    pub fn new() -> Self {
        Self {
            notes: Vec::new(),
            key: None,
            tempo: None,
            time_signature: (4, 4),
            title: None,
        }
    }

    /// Convert to NoteSequence for MIDI generation
    ///
    /// # Arguments
    /// * `instrument` - Instrument name or GM program number
    /// * `tempo` - Override tempo (uses file tempo if None)
    ///
    /// # Returns
    /// A vector containing one NoteSequence
    pub fn to_sequences(
        &self,
        instrument: &str,
        tempo: Option<u16>,
    ) -> Result<Vec<NoteSequence>, ImportError> {
        let instrument_num = resolve_instrument(instrument).unwrap_or(0);
        let final_tempo = tempo.or(self.tempo).unwrap_or(120);

        // Filter out rests and convert to Note
        let notes: Vec<Note> = self
            .notes
            .iter()
            .filter_map(|n| {
                n.pitch.map(|pitch| Note {
                    pitch,
                    duration: n.duration,
                    velocity: n.velocity,
                    offset: n.offset,
                })
            })
            .collect();

        if notes.is_empty() {
            return Err(ImportError::NoNotes);
        }

        Ok(vec![NoteSequence::new(notes, instrument_num, final_tempo)])
    }

    /// Get total duration in beats
    pub fn duration_beats(&self) -> f64 {
        self.notes
            .iter()
            .map(|n| n.offset + n.duration)
            .fold(0.0, f64::max)
    }

    /// Get note count (excluding rests)
    pub fn note_count(&self) -> usize {
        self.notes.iter().filter(|n| n.pitch.is_some()).count()
    }
}

impl Default for ImportedMelody {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert MIDI pitch number to note name (e.g., 60 -> "C4")
pub fn midi_pitch_to_name(pitch: u8) -> String {
    const NOTE_NAMES: [&str; 12] = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
    let octave = (pitch / 12) as i32 - 1;
    let note_index = (pitch % 12) as usize;
    format!("{}{}", NOTE_NAMES[note_index], octave)
}

/// Convert note name to MIDI pitch number (e.g., "C4" -> 60)
pub fn name_to_midi_pitch(name: &str) -> Option<u8> {
    if name.eq_ignore_ascii_case("rest") {
        return None;
    }

    let name = name.trim();
    if name.is_empty() {
        return None;
    }

    // Parse note letter and accidental
    let mut chars = name.chars().peekable();
    let note_letter = chars.next()?.to_ascii_uppercase();

    let base_pitch = match note_letter {
        'C' => 0,
        'D' => 2,
        'E' => 4,
        'F' => 5,
        'G' => 7,
        'A' => 9,
        'B' => 11,
        _ => return None,
    };

    // Check for accidental
    let mut pitch_offset = 0i8;
    if let Some(&c) = chars.peek() {
        match c {
            '#' => {
                pitch_offset = 1;
                chars.next();
            }
            'b' => {
                pitch_offset = -1;
                chars.next();
            }
            _ => {}
        }
    }

    // Parse octave
    let octave_str: String = chars.collect();
    let octave: i32 = octave_str.parse().ok()?;

    // Calculate MIDI pitch
    let midi_pitch = (octave + 1) * 12 + base_pitch as i32 + pitch_offset as i32;

    if midi_pitch >= 0 && midi_pitch <= 127 {
        Some(midi_pitch as u8)
    } else {
        None
    }
}

/// A melody note in server format (pitch as string)
#[derive(Debug, Clone)]
pub struct ServerMelodyNote {
    pub pitch: String,
    pub duration: f64,
    pub velocity: u8,
}

impl ImportedMelody {
    /// Convert to server MelodyNote format (pitch as string like "C4")
    pub fn to_melody_notes(&self) -> Vec<ServerMelodyNote> {
        self.notes
            .iter()
            .map(|n| ServerMelodyNote {
                pitch: n.pitch.map(midi_pitch_to_name).unwrap_or_else(|| "rest".to_string()),
                duration: n.duration,
                velocity: n.velocity,
            })
            .collect()
    }
}

/// Convert a list of melody notes to ABC notation
pub fn notes_to_abc(
    notes: &[(String, f64, u8)], // (pitch, duration, velocity)
    title: &str,
    key: &str,
    tempo: u16,
) -> String {
    let mut abc = String::new();

    // Header
    abc.push_str("X:1\n");
    abc.push_str(&format!("T:{}\n", title));
    abc.push_str("M:4/4\n");
    abc.push_str("L:1/4\n"); // Default note length is quarter note
    abc.push_str(&format!("Q:1/4={}\n", tempo));
    abc.push_str(&format!("K:{}\n", key));

    // Notes
    let mut bar_position = 0.0;
    for (i, (pitch, duration, _velocity)) in notes.iter().enumerate() {
        // Convert pitch to ABC notation
        let abc_note = pitch_to_abc(pitch, *duration);
        abc.push_str(&abc_note);

        bar_position += duration;

        // Add bar line every 4 beats
        if bar_position >= 4.0 {
            bar_position -= 4.0;
            if i < notes.len() - 1 {
                abc.push_str(" | ");
            }
        } else {
            abc.push(' ');
        }
    }

    // End bar
    abc.push_str("|\n");

    abc
}

/// Convert a pitch string and duration to ABC notation
fn pitch_to_abc(pitch: &str, duration: f64) -> String {
    if pitch == "rest" {
        return duration_to_abc("z", duration);
    }

    let pitch = pitch.trim();
    if pitch.is_empty() {
        return "z".to_string();
    }

    let mut chars = pitch.chars().peekable();
    let note_letter = chars.next().unwrap_or('C').to_ascii_uppercase();

    // Check for accidental
    let mut accidental = String::new();
    if let Some(&c) = chars.peek() {
        match c {
            '#' => {
                accidental = "^".to_string();
                chars.next();
            }
            'b' => {
                accidental = "_".to_string();
                chars.next();
            }
            _ => {}
        }
    }

    // Parse octave
    let octave_str: String = chars.collect();
    let octave: i32 = octave_str.parse().unwrap_or(4);

    // Convert to ABC octave notation
    // ABC: C = C4, c = C5, C, = C3, c' = C6
    let abc_note = if octave < 4 {
        // Lower octave - use uppercase with commas
        let commas = ",".repeat((4 - octave) as usize);
        format!("{}{}{}", accidental, note_letter, commas)
    } else if octave == 4 {
        // Middle octave - uppercase
        format!("{}{}", accidental, note_letter)
    } else {
        // Higher octave - lowercase with apostrophes
        let apostrophes = "'".repeat((octave - 5) as usize);
        format!("{}{}{}", accidental, note_letter.to_ascii_lowercase(), apostrophes)
    };

    duration_to_abc(&abc_note, duration)
}

/// Add duration suffix to ABC note
fn duration_to_abc(note: &str, duration: f64) -> String {
    // Duration is in quarter notes, L:1/4 means default is quarter
    if (duration - 1.0).abs() < 0.01 {
        note.to_string()
    } else if (duration - 2.0).abs() < 0.01 {
        format!("{}2", note)
    } else if (duration - 0.5).abs() < 0.01 {
        format!("{}/2", note)
    } else if (duration - 0.25).abs() < 0.01 {
        format!("{}/4", note)
    } else if (duration - 3.0).abs() < 0.01 {
        format!("{}3", note)
    } else if (duration - 4.0).abs() < 0.01 {
        format!("{}4", note)
    } else if (duration - 1.5).abs() < 0.01 {
        format!("{}3/2", note)
    } else {
        // For other durations, try to express as fraction
        let numerator = (duration * 4.0).round() as u32;
        if numerator == 4 {
            note.to_string()
        } else if numerator > 4 {
            format!("{}{}/4", note, numerator)
        } else {
            format!("{}/{}", note, 4 / numerator)
        }
    }
}

/// Trait for melody importers
pub trait MelodyImporter {
    /// Parse a file and return an ImportedMelody
    fn parse_file(path: &std::path::Path) -> Result<ImportedMelody, ImportError>;

    /// Parse from a string
    fn parse_string(content: &str) -> Result<ImportedMelody, ImportError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_imported_note_creation() {
        let note = ImportedNote::new(60, 1.0, 0.0);
        assert_eq!(note.pitch, Some(60));
        assert_eq!(note.duration, 1.0);
        assert_eq!(note.velocity, 80);
        assert_eq!(note.offset, 0.0);
    }

    #[test]
    fn test_imported_rest() {
        let rest = ImportedNote::rest(0.5, 1.0);
        assert_eq!(rest.pitch, None);
        assert_eq!(rest.duration, 0.5);
        assert_eq!(rest.offset, 1.0);
    }

    #[test]
    fn test_melody_to_sequences() {
        let mut melody = ImportedMelody::new();
        melody.notes.push(ImportedNote::new(60, 1.0, 0.0));
        melody.notes.push(ImportedNote::new(64, 1.0, 1.0));
        melody.notes.push(ImportedNote::rest(0.5, 2.0)); // rest should be filtered
        melody.notes.push(ImportedNote::new(67, 1.0, 2.5));

        let sequences = melody.to_sequences("piano", Some(100)).unwrap();
        assert_eq!(sequences.len(), 1);
        assert_eq!(sequences[0].notes.len(), 3); // 3 notes, rest excluded
        assert_eq!(sequences[0].tempo, 100);
        assert_eq!(sequences[0].instrument, 0);
    }

    #[test]
    fn test_melody_duration() {
        let mut melody = ImportedMelody::new();
        melody.notes.push(ImportedNote::new(60, 1.0, 0.0));
        melody.notes.push(ImportedNote::new(64, 2.0, 1.0));
        assert_eq!(melody.duration_beats(), 3.0);
    }

    #[test]
    fn test_midi_pitch_to_name() {
        assert_eq!(midi_pitch_to_name(60), "C4");
        assert_eq!(midi_pitch_to_name(61), "C#4");
        assert_eq!(midi_pitch_to_name(69), "A4");
        assert_eq!(midi_pitch_to_name(72), "C5");
        assert_eq!(midi_pitch_to_name(48), "C3");
        assert_eq!(midi_pitch_to_name(21), "A0");
    }

    #[test]
    fn test_name_to_midi_pitch() {
        assert_eq!(name_to_midi_pitch("C4"), Some(60));
        assert_eq!(name_to_midi_pitch("C#4"), Some(61));
        assert_eq!(name_to_midi_pitch("Db4"), Some(61));
        assert_eq!(name_to_midi_pitch("A4"), Some(69));
        assert_eq!(name_to_midi_pitch("C5"), Some(72));
        assert_eq!(name_to_midi_pitch("rest"), None);
        assert_eq!(name_to_midi_pitch("REST"), None);
    }

    #[test]
    fn test_to_melody_notes() {
        let mut melody = ImportedMelody::new();
        melody.notes.push(ImportedNote::new(60, 1.0, 0.0));
        melody.notes.push(ImportedNote::rest(0.5, 1.0));
        melody.notes.push(ImportedNote::new(64, 2.0, 1.5));

        let notes = melody.to_melody_notes();
        assert_eq!(notes.len(), 3);
        assert_eq!(notes[0].pitch, "C4");
        assert_eq!(notes[0].duration, 1.0);
        assert_eq!(notes[1].pitch, "rest");
        assert_eq!(notes[2].pitch, "E4");
    }

    #[test]
    fn test_notes_to_abc() {
        let notes = vec![
            ("C4".to_string(), 1.0, 80u8),
            ("E4".to_string(), 1.0, 80),
            ("G4".to_string(), 2.0, 80),
        ];

        let abc = notes_to_abc(&notes, "Test Tune", "C", 120);
        assert!(abc.contains("X:1"));
        assert!(abc.contains("T:Test Tune"));
        assert!(abc.contains("K:C"));
        assert!(abc.contains("Q:1/4=120"));
        assert!(abc.contains("C E G2"));
    }

    #[test]
    fn test_pitch_to_abc_octaves() {
        // Middle C (C4) should be uppercase C
        assert_eq!(pitch_to_abc("C4", 1.0), "C");
        // C5 should be lowercase c
        assert_eq!(pitch_to_abc("C5", 1.0), "c");
        // C6 should be c'
        assert_eq!(pitch_to_abc("C6", 1.0), "c'");
        // C3 should be C,
        assert_eq!(pitch_to_abc("C3", 1.0), "C,");
        // C2 should be C,,
        assert_eq!(pitch_to_abc("C2", 1.0), "C,,");
    }

    #[test]
    fn test_pitch_to_abc_accidentals() {
        assert_eq!(pitch_to_abc("C#4", 1.0), "^C");
        assert_eq!(pitch_to_abc("Db4", 1.0), "_D");
    }

    #[test]
    fn test_pitch_to_abc_durations() {
        assert_eq!(pitch_to_abc("C4", 0.5), "C/2");
        assert_eq!(pitch_to_abc("C4", 2.0), "C2");
        assert_eq!(pitch_to_abc("C4", 0.25), "C/4");
    }
}
