//! Note sequence representation
//!
//! A sequence is a collection of notes with instrument and tempo settings.

use super::Note;
use serde::Deserialize;

/// General MIDI instrument names mapped to program numbers
pub const INSTRUMENT_MAP: &[(&str, u8)] = &[
    // Pianos
    ("piano", 0),
    ("acoustic_piano", 0),
    ("bright_piano", 1),
    ("electric_piano", 4),
    // Strings
    ("strings", 48),
    ("violin", 40),
    ("viola", 41),
    ("cello", 42),
    ("contrabass", 43),
    ("tremolo_strings", 44),
    ("pizzicato_strings", 45),
    ("harp", 46),
    // Woodwinds
    ("flute", 73),
    ("oboe", 68),
    ("clarinet", 71),
    ("bassoon", 70),
    // Brass
    ("trumpet", 56),
    ("trombone", 57),
    ("french_horn", 60),
    ("tuba", 58),
    // Synth
    ("synth_pad", 88),
    ("synth_lead", 80),
    ("pad_warm", 89),
    ("pad_choir", 91),
    // Ambient
    ("atmosphere", 99),
    ("soundtrack", 97),
    // Guitar
    ("acoustic_guitar", 25),
    ("electric_guitar", 27),
    ("bass", 33),
    ("electric_bass", 33),
    // Bells/Percussion
    ("vibraphone", 11),
    ("marimba", 12),
    ("xylophone", 13),
    ("tubular_bells", 14),
    ("glockenspiel", 9),
    ("celesta", 8),
];

/// Resolve instrument name to GM program number
pub fn resolve_instrument(name: &str) -> Option<u8> {
    let name_lower = name.to_lowercase();

    // Try direct program number
    if let Ok(num) = name_lower.parse::<u8>()
        && num <= 127
    {
        return Some(num);
    }

    // Try named instrument
    INSTRUMENT_MAP
        .iter()
        .find(|(n, _)| *n == name_lower)
        .map(|(_, num)| *num)
}

/// A sequence of notes with instrument and tempo settings
#[derive(Debug, Clone)]
pub struct NoteSequence {
    /// Notes in the sequence
    pub notes: Vec<Note>,

    /// GM instrument program number (0-127)
    pub instrument: u8,

    /// MIDI channel (0-15, usually 0)
    pub channel: u8,

    /// Tempo in BPM
    pub tempo: u16,
}

impl NoteSequence {
    /// Create a new note sequence
    pub fn new(notes: Vec<Note>, instrument: u8, tempo: u16) -> Self {
        Self {
            notes,
            instrument,
            channel: 0,
            tempo,
        }
    }

    /// Create from notes with default settings
    pub fn from_notes(notes: Vec<Note>) -> Self {
        Self::new(notes, 0, 120)
    }

    /// Calculate total duration in beats
    pub fn duration_beats(&self) -> f64 {
        self.notes
            .iter()
            .map(|n| n.offset + n.duration)
            .fold(0.0, f64::max)
    }

    /// Calculate duration in seconds
    pub fn duration_seconds(&self) -> f64 {
        let beats = self.duration_beats();
        beats * 60.0 / self.tempo as f64
    }
}

/// JSON input format for note sequences
#[derive(Debug, Deserialize)]
pub struct JsonNoteInput {
    pub pitch: String,
    pub duration: f64,
    pub velocity: u8,
    #[serde(default)]
    pub offset: f64,
}

/// JSON input format for a single track
#[derive(Debug, Deserialize)]
pub struct JsonTrackInput {
    #[serde(default = "default_instrument")]
    pub instrument: String,
    #[serde(default)]
    pub channel: u8,
    pub notes: Vec<JsonNoteInput>,
}

/// JSON input format for full sequence
#[derive(Debug, Deserialize)]
pub struct JsonSequenceInput {
    #[serde(default = "default_tempo")]
    pub tempo: u16,
    #[serde(default = "default_instrument")]
    pub instrument: String,
    #[serde(default)]
    pub channel: u8,
    #[serde(default)]
    pub notes: Vec<JsonNoteInput>,
    #[serde(default)]
    pub tracks: Vec<JsonTrackInput>,
}

fn default_tempo() -> u16 {
    120
}

fn default_instrument() -> String {
    "piano".to_string()
}

impl JsonSequenceInput {
    /// Convert to NoteSequences
    pub fn to_sequences(&self) -> Result<Vec<NoteSequence>, super::note::NoteError> {
        let mut sequences = Vec::new();

        // If tracks are specified, use those
        if !self.tracks.is_empty() {
            for track in &self.tracks {
                let notes = track
                    .notes
                    .iter()
                    .map(|n| {
                        let pitch = Note::parse_pitch(&n.pitch)?;
                        Ok(Note::new(pitch, n.duration, n.velocity, n.offset))
                    })
                    .collect::<Result<Vec<_>, _>>()?;

                let instrument = resolve_instrument(&track.instrument).unwrap_or(0);
                let mut seq = NoteSequence::new(notes, instrument, self.tempo);
                seq.channel = track.channel;
                sequences.push(seq);
            }
        } else if !self.notes.is_empty() {
            // Use top-level notes
            let notes = self
                .notes
                .iter()
                .map(|n| {
                    let pitch = Note::parse_pitch(&n.pitch)?;
                    Ok(Note::new(pitch, n.duration, n.velocity, n.offset))
                })
                .collect::<Result<Vec<_>, _>>()?;

            let instrument = resolve_instrument(&self.instrument).unwrap_or(0);
            let mut seq = NoteSequence::new(notes, instrument, self.tempo);
            seq.channel = self.channel;
            sequences.push(seq);
        }

        Ok(sequences)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_instrument_by_name() {
        assert_eq!(resolve_instrument("piano"), Some(0));
        assert_eq!(resolve_instrument("violin"), Some(40));
        assert_eq!(resolve_instrument("strings"), Some(48));
    }

    #[test]
    fn test_resolve_instrument_case_insensitive() {
        assert_eq!(resolve_instrument("PIANO"), Some(0));
        assert_eq!(resolve_instrument("Piano"), Some(0));
    }

    #[test]
    fn test_resolve_instrument_by_number() {
        assert_eq!(resolve_instrument("0"), Some(0));
        assert_eq!(resolve_instrument("40"), Some(40));
        assert_eq!(resolve_instrument("127"), Some(127));
    }

    #[test]
    fn test_resolve_instrument_invalid() {
        assert_eq!(resolve_instrument("invalid"), None);
        assert_eq!(resolve_instrument("128"), None);
    }

    #[test]
    fn test_sequence_duration() {
        let notes = vec![
            Note::new(60, 1.0, 80, 0.0),
            Note::new(64, 1.0, 80, 1.0),
            Note::new(67, 1.0, 80, 2.0),
        ];
        let seq = NoteSequence::new(notes, 0, 120);
        assert_eq!(seq.duration_beats(), 3.0);
        assert_eq!(seq.duration_seconds(), 1.5); // 3 beats at 120 BPM = 1.5 seconds
    }

    #[test]
    fn test_json_parsing() {
        let json = r#"{
            "tempo": 100,
            "instrument": "strings",
            "notes": [
                {"pitch": "C4", "duration": 1.0, "velocity": 80, "offset": 0.0},
                {"pitch": "E4", "duration": 1.0, "velocity": 80, "offset": 1.0}
            ]
        }"#;

        let input: JsonSequenceInput = serde_json::from_str(json).unwrap();
        assert_eq!(input.tempo, 100);
        assert_eq!(input.notes.len(), 2);

        let sequences = input.to_sequences().unwrap();
        assert_eq!(sequences.len(), 1);
        assert_eq!(sequences[0].notes.len(), 2);
        assert_eq!(sequences[0].instrument, 48); // strings
    }

    #[test]
    fn test_json_multi_track() {
        let json = r#"{
            "tempo": 90,
            "tracks": [
                {
                    "instrument": "piano",
                    "notes": [{"pitch": "C4", "duration": 1.0, "velocity": 80}]
                },
                {
                    "instrument": "bass",
                    "notes": [{"pitch": "C2", "duration": 2.0, "velocity": 100}]
                }
            ]
        }"#;

        let input: JsonSequenceInput = serde_json::from_str(json).unwrap();
        let sequences = input.to_sequences().unwrap();
        assert_eq!(sequences.len(), 2);
        assert_eq!(sequences[0].instrument, 0); // piano
        assert_eq!(sequences[1].instrument, 33); // bass
    }
}
