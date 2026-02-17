//! Note representation and parsing
//!
//! Notes are specified in format: `PITCH:DURATION:VELOCITY[@OFFSET]`
//!
//! Examples:
//! - `C4:1:80` - Middle C, 1 beat, velocity 80
//! - `F#3:0.5:100@2` - F# below middle C, half beat, velocity 100, starting at beat 2

use std::str::FromStr;
use thiserror::Error;

/// Errors that can occur when parsing notes
#[derive(Debug, Error, PartialEq)]
pub enum NoteError {
    #[error("Bad format: {0}. Expected PITCH:DURATION:VELOCITY[@OFFSET]")]
    BadFormat(String),

    #[error(
        "Bad pitch: {0}. Expected note name (A-G) with optional accidental (#/b) and octave (0-10)"
    )]
    BadPitch(String),

    #[error("Bad duration: {0}. Expected positive number")]
    BadDuration(String),

    #[error("Bad velocity: {0}. Expected 0-127")]
    BadVelocity(String),

    #[error("Bad offset: {0}. Expected non-negative number")]
    BadOffset(String),
}

/// A single MIDI note with pitch, duration, velocity, and timing
#[derive(Debug, Clone, PartialEq)]
pub struct Note {
    /// MIDI pitch (0-127, 60 = C4 = middle C)
    pub pitch: u8,

    /// Duration in beats (1.0 = quarter note)
    pub duration: f64,

    /// Velocity/volume (0-127)
    pub velocity: u8,

    /// Start time in beats from sequence start
    pub offset: f64,
}

impl Note {
    /// Create a new note
    pub fn new(pitch: u8, duration: f64, velocity: u8, offset: f64) -> Self {
        Self {
            pitch,
            duration,
            velocity,
            offset,
        }
    }

    /// Parse a note name (like "C4", "F#3", "Bb5") to MIDI pitch number
    pub fn parse_pitch(pitch_str: &str) -> Result<u8, NoteError> {
        let pitch_str = pitch_str.trim();
        if pitch_str.is_empty() {
            return Err(NoteError::BadPitch(pitch_str.to_string()));
        }

        let mut chars = pitch_str.chars().peekable();

        // Parse note name (A-G)
        let note_char = chars
            .next()
            .ok_or_else(|| NoteError::BadPitch(pitch_str.to_string()))?;
        let semitone = match note_char.to_ascii_uppercase() {
            'C' => 0,
            'D' => 2,
            'E' => 4,
            'F' => 5,
            'G' => 7,
            'A' => 9,
            'B' => 11,
            _ => return Err(NoteError::BadPitch(pitch_str.to_string())),
        };

        // Parse optional accidental (# or b)
        let accidental = match chars.peek() {
            Some('#') => {
                chars.next();
                1i8
            }
            Some('b') => {
                chars.next();
                -1i8
            }
            _ => 0i8,
        };

        // Parse octave number
        let octave_str: String = chars.collect();
        let octave: i8 = octave_str
            .parse()
            .map_err(|_| NoteError::BadPitch(pitch_str.to_string()))?;

        if !(0..=10).contains(&octave) {
            return Err(NoteError::BadPitch(pitch_str.to_string()));
        }

        // Calculate MIDI pitch: (octave + 1) * 12 + semitone + accidental
        // C4 = (4 + 1) * 12 + 0 = 60
        let midi_pitch = (octave as i16 + 1) * 12 + semitone as i16 + accidental as i16;

        if !(0..=127).contains(&midi_pitch) {
            return Err(NoteError::BadPitch(pitch_str.to_string()));
        }

        Ok(midi_pitch as u8)
    }

    /// Parse a note from string format: "PITCH:DURATION:VELOCITY[@OFFSET]"
    pub fn parse(s: &str) -> Result<Self, NoteError> {
        let s = s.trim();

        // Split on @ to get offset if present
        let (main_part, offset) = if let Some(at_pos) = s.find('@') {
            let offset_str = &s[at_pos + 1..];
            let offset: f64 = offset_str
                .parse()
                .map_err(|_| NoteError::BadOffset(offset_str.to_string()))?;
            if offset < 0.0 {
                return Err(NoteError::BadOffset(offset_str.to_string()));
            }
            (&s[..at_pos], offset)
        } else {
            (s, 0.0)
        };

        // Split main part by colon
        let parts: Vec<&str> = main_part.split(':').collect();
        if parts.len() != 3 {
            return Err(NoteError::BadFormat(s.to_string()));
        }

        let pitch = Self::parse_pitch(parts[0])?;

        let duration: f64 = parts[1]
            .parse()
            .map_err(|_| NoteError::BadDuration(parts[1].to_string()))?;
        if duration <= 0.0 {
            return Err(NoteError::BadDuration(parts[1].to_string()));
        }

        let velocity: u8 = parts[2]
            .parse()
            .map_err(|_| NoteError::BadVelocity(parts[2].to_string()))?;
        if velocity > 127 {
            return Err(NoteError::BadVelocity(parts[2].to_string()));
        }

        Ok(Self::new(pitch, duration, velocity, offset))
    }

    /// Parse multiple notes from comma-separated string
    pub fn parse_many(s: &str) -> Result<Vec<Self>, NoteError> {
        s.split(',')
            .map(|note_str| Self::parse(note_str.trim()))
            .collect()
    }
}

impl FromStr for Note {
    type Err = NoteError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===================
    // Pitch Parsing Tests
    // ===================

    #[test]
    fn test_parse_pitch_c4_middle_c() {
        assert_eq!(Note::parse_pitch("C4").unwrap(), 60);
    }

    #[test]
    fn test_parse_pitch_a4_concert_a() {
        assert_eq!(Note::parse_pitch("A4").unwrap(), 69);
    }

    #[test]
    fn test_parse_pitch_c0_lowest_c() {
        assert_eq!(Note::parse_pitch("C0").unwrap(), 12);
    }

    #[test]
    fn test_parse_pitch_sharp() {
        assert_eq!(Note::parse_pitch("F#3").unwrap(), 54);
        assert_eq!(Note::parse_pitch("C#4").unwrap(), 61);
    }

    #[test]
    fn test_parse_pitch_flat() {
        assert_eq!(Note::parse_pitch("Bb3").unwrap(), 58);
        assert_eq!(Note::parse_pitch("Eb4").unwrap(), 63);
    }

    #[test]
    fn test_parse_pitch_all_notes_octave_4() {
        assert_eq!(Note::parse_pitch("C4").unwrap(), 60);
        assert_eq!(Note::parse_pitch("D4").unwrap(), 62);
        assert_eq!(Note::parse_pitch("E4").unwrap(), 64);
        assert_eq!(Note::parse_pitch("F4").unwrap(), 65);
        assert_eq!(Note::parse_pitch("G4").unwrap(), 67);
        assert_eq!(Note::parse_pitch("A4").unwrap(), 69);
        assert_eq!(Note::parse_pitch("B4").unwrap(), 71);
    }

    #[test]
    fn test_parse_pitch_lowercase() {
        assert_eq!(Note::parse_pitch("c4").unwrap(), 60);
        assert_eq!(Note::parse_pitch("a4").unwrap(), 69);
    }

    #[test]
    fn test_parse_pitch_invalid_note_name() {
        assert!(Note::parse_pitch("X4").is_err());
        assert!(Note::parse_pitch("H4").is_err());
    }

    #[test]
    fn test_parse_pitch_invalid_octave() {
        assert!(Note::parse_pitch("C11").is_err());
        assert!(Note::parse_pitch("C-1").is_err());
    }

    #[test]
    fn test_parse_pitch_empty() {
        assert!(Note::parse_pitch("").is_err());
    }

    // ==================
    // Note Parsing Tests
    // ==================

    #[test]
    fn test_parse_note_basic() {
        let note = Note::parse("C4:1:80").unwrap();
        assert_eq!(note.pitch, 60);
        assert_eq!(note.duration, 1.0);
        assert_eq!(note.velocity, 80);
        assert_eq!(note.offset, 0.0);
    }

    #[test]
    fn test_parse_note_with_offset() {
        let note = Note::parse("E4:0.5:100@2").unwrap();
        assert_eq!(note.pitch, 64);
        assert_eq!(note.duration, 0.5);
        assert_eq!(note.velocity, 100);
        assert_eq!(note.offset, 2.0);
    }

    #[test]
    fn test_parse_note_fractional_offset() {
        let note = Note::parse("G4:0.25:90@1.5").unwrap();
        assert_eq!(note.pitch, 67);
        assert_eq!(note.duration, 0.25);
        assert_eq!(note.velocity, 90);
        assert_eq!(note.offset, 1.5);
    }

    #[test]
    fn test_parse_note_with_accidental() {
        let note = Note::parse("F#3:2:60").unwrap();
        assert_eq!(note.pitch, 54);
        assert_eq!(note.duration, 2.0);
        assert_eq!(note.velocity, 60);
    }

    #[test]
    fn test_parse_note_invalid_format_too_few_parts() {
        assert!(Note::parse("C4:1").is_err());
    }

    #[test]
    fn test_parse_note_invalid_format_too_many_parts() {
        assert!(Note::parse("C4:1:80:extra").is_err());
    }

    #[test]
    fn test_parse_note_zero_duration_invalid() {
        assert!(Note::parse("C4:0:80").is_err());
    }

    #[test]
    fn test_parse_note_negative_duration_invalid() {
        assert!(Note::parse("C4:-1:80").is_err());
    }

    #[test]
    fn test_parse_note_velocity_too_high() {
        assert!(Note::parse("C4:1:200").is_err());
    }

    #[test]
    fn test_parse_note_zero_velocity_valid() {
        let note = Note::parse("C4:1:0").unwrap();
        assert_eq!(note.velocity, 0);
    }

    #[test]
    fn test_parse_note_max_velocity() {
        let note = Note::parse("C4:1:127").unwrap();
        assert_eq!(note.velocity, 127);
    }

    // ====================
    // Multi-Note Parsing
    // ====================

    #[test]
    fn test_parse_many_notes() {
        let notes = Note::parse_many("C4:1:80,E4:0.5:100,G4:0.5:100").unwrap();
        assert_eq!(notes.len(), 3);
        assert_eq!(notes[0].pitch, 60);
        assert_eq!(notes[1].pitch, 64);
        assert_eq!(notes[2].pitch, 67);
    }

    #[test]
    fn test_parse_many_with_spaces() {
        let notes = Note::parse_many("C4:1:80, E4:0.5:100, G4:0.5:100").unwrap();
        assert_eq!(notes.len(), 3);
    }

    #[test]
    fn test_parse_many_single_note() {
        let notes = Note::parse_many("C4:1:80").unwrap();
        assert_eq!(notes.len(), 1);
    }

    #[test]
    fn test_parse_many_with_offsets() {
        let notes = Note::parse_many("C4:1:80@0,E4:1:80@1,G4:1:80@2").unwrap();
        assert_eq!(notes[0].offset, 0.0);
        assert_eq!(notes[1].offset, 1.0);
        assert_eq!(notes[2].offset, 2.0);
    }

    // ================
    // FromStr Trait
    // ================

    #[test]
    fn test_from_str() {
        let note: Note = "C4:1:80".parse().unwrap();
        assert_eq!(note.pitch, 60);
    }
}
