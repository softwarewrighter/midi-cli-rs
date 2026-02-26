//! ABC notation parser
//!
//! Parses ABC notation files into ImportedMelody structures.
//! ABC notation is a text-based music notation widely used for folk tunes.
//!
//! # Supported Features
//! - Header fields: X, T, M, L, K, Q
//! - Notes: A-G, a-g with octave markers (' and ,)
//! - Accidentals: ^ (sharp), _ (flat), = (natural)
//! - Duration modifiers: /2, 2, 3/2, etc.
//! - Rests: z, Z
//! - Bar lines: |, ||, |], [|
//! - Ties: - (combines note durations)
//!
//! # Limitations
//! - Chords [CEG] - only first note is used
//! - Decorations !trill! - ignored
//! - Slurs () - ignored (notes kept separate)
//! - Multi-voice - only first voice parsed

use super::{ImportError, ImportedMelody, ImportedNote, MelodyImporter};
use std::path::Path;

/// ABC notation parser
pub struct AbcParser;

impl MelodyImporter for AbcParser {
    fn parse_file(path: &Path) -> Result<ImportedMelody, ImportError> {
        if !path.exists() {
            return Err(ImportError::FileNotFound(path.display().to_string()));
        }
        let content = std::fs::read_to_string(path)?;
        Self::parse_string(&content)
    }

    fn parse_string(content: &str) -> Result<ImportedMelody, ImportError> {
        let mut melody = ImportedMelody::new();
        let mut unit_length = 1.0 / 8.0; // L:1/8 default (eighth note = 0.5 beats)
        let mut key_accidentals: [i8; 7] = [0; 7]; // C D E F G A B sharps/flats from key signature
        let mut in_body = false;
        let mut current_offset = 0.0;
        let mut pending_tie = false;

        for line in content.lines() {
            let line = line.trim();

            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('%') {
                continue;
            }

            // Parse header fields
            if line.len() >= 2 && line.chars().nth(1) == Some(':') {
                let field = line.chars().next().unwrap();
                let value = line[2..].trim();

                match field {
                    'X' => {
                        // Reference number - marks start of tune
                        in_body = false;
                    }
                    'T' => {
                        // Title
                        if melody.title.is_none() {
                            melody.title = Some(value.to_string());
                        }
                    }
                    'M' => {
                        // Time signature (e.g., "4/4", "6/8", "C")
                        melody.time_signature = parse_time_signature(value);
                    }
                    'L' => {
                        // Unit note length (e.g., "1/8", "1/4")
                        unit_length = parse_fraction(value).unwrap_or(1.0 / 8.0);
                    }
                    'K' => {
                        // Key signature - marks end of header, start of body
                        melody.key = Some(value.to_string());
                        key_accidentals = key_signature_accidentals(value);
                        in_body = true;
                    }
                    'Q' => {
                        // Tempo (e.g., "1/4=120", "120")
                        melody.tempo = parse_tempo(value);
                    }
                    _ => {}
                }
                continue;
            }

            // Parse body (notes)
            if in_body {
                let (notes, new_offset, tie) =
                    parse_abc_line(line, unit_length, &key_accidentals, current_offset, pending_tie)?;
                melody.notes.extend(notes);
                current_offset = new_offset;
                pending_tie = tie;
            }
        }

        if melody.notes.is_empty() {
            return Err(ImportError::NoNotes);
        }

        Ok(melody)
    }
}

impl AbcParser {
    /// Parse ABC notation from a file
    pub fn parse_file(path: &Path) -> Result<ImportedMelody, ImportError> {
        <Self as MelodyImporter>::parse_file(path)
    }

    /// Parse ABC notation from a string
    pub fn parse_string(content: &str) -> Result<ImportedMelody, ImportError> {
        <Self as MelodyImporter>::parse_string(content)
    }
}

/// Parse time signature string to (numerator, denominator)
fn parse_time_signature(s: &str) -> (u8, u8) {
    let s = s.trim();
    if s == "C" {
        return (4, 4);
    }
    if s == "C|" {
        return (2, 2);
    }

    if let Some((num, denom)) = s.split_once('/') {
        let num: u8 = num.trim().parse().unwrap_or(4);
        let denom: u8 = denom.trim().parse().unwrap_or(4);
        (num, denom)
    } else {
        (4, 4)
    }
}

/// Parse a fraction string like "1/8" to a float
fn parse_fraction(s: &str) -> Option<f64> {
    let s = s.trim();
    if let Some((num, denom)) = s.split_once('/') {
        let num: f64 = num.trim().parse().ok()?;
        let denom: f64 = denom.trim().parse().ok()?;
        if denom != 0.0 {
            Some(num / denom)
        } else {
            None
        }
    } else {
        s.parse().ok()
    }
}

/// Parse tempo string like "1/4=120" or just "120"
fn parse_tempo(s: &str) -> Option<u16> {
    let s = s.trim();
    if let Some(eq_pos) = s.find('=') {
        // Format: "1/4=120"
        s[eq_pos + 1..].trim().parse().ok()
    } else {
        // Format: "120"
        s.parse().ok()
    }
}

/// Get key signature accidentals for each note (C=0, D=1, E=2, F=3, G=4, A=5, B=6)
/// Returns array of -1 (flat), 0 (natural), or 1 (sharp) for each note
fn key_signature_accidentals(key: &str) -> [i8; 7] {
    let key = key.trim().to_uppercase();
    let mut accidentals = [0i8; 7];

    // Sharp keys: G D A E B F# C#
    // Flat keys: F Bb Eb Ab Db Gb Cb
    // Order of sharps: F C G D A E B
    // Order of flats: B E A D G C F

    let (root, is_minor) = if key.ends_with('M') {
        (&key[..key.len() - 1], true)
    } else if key.contains("MIN") {
        (key.split("MIN").next().unwrap_or(&key), true)
    } else {
        (key.as_str(), false)
    };

    // Get effective major key (for minor, use relative major)
    let major_key = if is_minor {
        // Minor key - convert to relative major (up a minor third)
        match root {
            "A" => "C",
            "E" => "G",
            "B" => "D",
            "F#" | "F" => "A", // F#m -> A, Fm -> Ab (simplified)
            "C#" | "C" => "E", // C#m -> E, Cm -> Eb (simplified)
            "G#" | "G" => "B", // G#m -> B, Gm -> Bb (simplified)
            "D#" | "D" => "F#", // D#m -> F#, Dm -> F
            _ => "C",
        }
    } else {
        root
    };

    // Apply sharps/flats based on key
    match major_key {
        // Sharp keys
        "G" => {
            accidentals[3] = 1; // F#
        }
        "D" => {
            accidentals[3] = 1; // F#
            accidentals[0] = 1; // C#
        }
        "A" => {
            accidentals[3] = 1; // F#
            accidentals[0] = 1; // C#
            accidentals[4] = 1; // G#
        }
        "E" => {
            accidentals[3] = 1; // F#
            accidentals[0] = 1; // C#
            accidentals[4] = 1; // G#
            accidentals[1] = 1; // D#
        }
        "B" => {
            accidentals[3] = 1;
            accidentals[0] = 1;
            accidentals[4] = 1;
            accidentals[1] = 1;
            accidentals[5] = 1; // A#
        }
        "F#" => {
            accidentals[3] = 1;
            accidentals[0] = 1;
            accidentals[4] = 1;
            accidentals[1] = 1;
            accidentals[5] = 1;
            accidentals[2] = 1; // E#
        }
        // Flat keys
        "F" => {
            accidentals[6] = -1; // Bb
        }
        "BB" => {
            // Bb major has 2 flats
            accidentals[6] = -1; // Bb
            accidentals[2] = -1; // Eb
        }
        "EB" => {
            accidentals[6] = -1; // Bb
            accidentals[2] = -1; // Eb
            accidentals[5] = -1; // Ab
        }
        "AB" => {
            accidentals[6] = -1;
            accidentals[2] = -1;
            accidentals[5] = -1;
            accidentals[1] = -1; // Db
        }
        _ => {} // C major or unrecognized - no accidentals
    }

    accidentals
}

/// Parse a line of ABC notation body
/// Returns (notes, new_offset, pending_tie)
fn parse_abc_line(
    line: &str,
    unit_length: f64,
    key_accidentals: &[i8; 7],
    start_offset: f64,
    mut pending_tie: bool,
) -> Result<(Vec<ImportedNote>, f64, bool), ImportError> {
    let mut notes: Vec<ImportedNote> = Vec::new();
    let mut offset = start_offset;
    let mut chars = line.chars().peekable();
    let mut local_accidentals: [Option<i8>; 7] = [None; 7]; // Per-bar accidentals

    while let Some(&c) = chars.peek() {
        match c {
            // Skip whitespace
            ' ' | '\t' => {
                chars.next();
            }

            // Bar lines reset local accidentals
            '|' | ']' | ':' => {
                chars.next();
                // Reset bar-local accidentals
                local_accidentals = [None; 7];
            }

            // Accidentals
            '^' | '_' | '=' => {
                chars.next();
                let accidental = match c {
                    '^' => 1i8,  // sharp
                    '_' => -1i8, // flat
                    '=' => 0i8,  // natural
                    _ => 0,
                };

                // Double sharp/flat
                let accidental = if chars.peek() == Some(&c) {
                    chars.next();
                    accidental * 2
                } else {
                    accidental
                };

                // Parse the note that follows
                if let Some((note, dur)) =
                    parse_note(&mut chars, unit_length, key_accidentals, &mut local_accidentals, Some(accidental))?
                {
                    if pending_tie && !notes.is_empty() {
                        // Extend previous note
                        if let Some(last) = notes.last_mut() {
                            last.duration += dur;
                        }
                        pending_tie = false;
                    } else {
                        notes.push(ImportedNote::new(note, dur, offset));
                        offset += dur;
                    }
                }
            }

            // Notes (uppercase = octave 4, lowercase = octave 5)
            'A'..='G' | 'a'..='g' => {
                if let Some((note, dur)) =
                    parse_note(&mut chars, unit_length, key_accidentals, &mut local_accidentals, None)?
                {
                    if pending_tie {
                        // Extend previous note
                        if let Some(last) = notes.last_mut() {
                            last.duration += dur;
                        }
                        pending_tie = false;
                    } else {
                        notes.push(ImportedNote::new(note, dur, offset));
                        offset += dur;
                    }
                }
            }

            // Rests
            'z' | 'Z' | 'x' | 'X' => {
                chars.next();
                let dur = parse_duration(&mut chars, unit_length);
                notes.push(ImportedNote::rest(dur, offset));
                offset += dur;
            }

            // Ties
            '-' => {
                chars.next();
                pending_tie = true;
            }

            // Chords - take first note only
            '[' => {
                chars.next();
                let mut first_note = None;
                while let Some(&ch) = chars.peek() {
                    if ch == ']' {
                        chars.next();
                        break;
                    }
                    if ch.is_ascii_alphabetic() && first_note.is_none() {
                        first_note =
                            parse_note(&mut chars, unit_length, key_accidentals, &mut local_accidentals, None)?;
                    } else {
                        chars.next();
                    }
                }
                if let Some((note, dur)) = first_note {
                    notes.push(ImportedNote::new(note, dur, offset));
                    offset += dur;
                }
            }

            // Slurs and decorations - skip
            '(' | ')' | '!' | '+' | '~' | 'H' | 'T' | '.' | '>' | '<' => {
                chars.next();
            }

            // Quotes (dynamics) - skip
            '"' => {
                chars.next();
                while let Some(&ch) = chars.peek() {
                    chars.next();
                    if ch == '"' {
                        break;
                    }
                }
            }

            // Line continuation
            '\\' => {
                chars.next();
            }

            // Numbers after notes are handled in parse_note
            '0'..='9' | '/' => {
                chars.next();
            }

            // Skip anything else
            _ => {
                chars.next();
            }
        }
    }

    Ok((notes, offset, pending_tie))
}

/// Parse a single note (pitch + duration)
/// Returns (MIDI pitch, duration in beats)
fn parse_note(
    chars: &mut std::iter::Peekable<std::str::Chars>,
    unit_length: f64,
    key_accidentals: &[i8; 7],
    local_accidentals: &mut [Option<i8>; 7],
    explicit_accidental: Option<i8>,
) -> Result<Option<(u8, f64)>, ImportError> {
    let c = match chars.next() {
        Some(c) => c,
        None => return Ok(None),
    };

    // Get base note (C=0, D=1, E=2, F=3, G=4, A=5, B=6)
    let (note_index, base_octave) = match c {
        'C' => (0, 4),
        'D' => (1, 4),
        'E' => (2, 4),
        'F' => (3, 4),
        'G' => (4, 4),
        'A' => (5, 4),
        'B' => (6, 4),
        'c' => (0, 5),
        'd' => (1, 5),
        'e' => (2, 5),
        'f' => (3, 5),
        'g' => (4, 5),
        'a' => (5, 5),
        'b' => (6, 5),
        _ => return Ok(None),
    };

    // Semitones from C for each note
    let semitones = [0, 2, 4, 5, 7, 9, 11]; // C D E F G A B

    // Determine accidental: explicit > local > key signature
    let accidental = if let Some(acc) = explicit_accidental {
        // Store in local accidentals
        local_accidentals[note_index] = Some(acc);
        acc
    } else if let Some(acc) = local_accidentals[note_index] {
        acc
    } else {
        key_accidentals[note_index]
    };

    // Parse octave modifiers (' raises, , lowers)
    let mut octave = base_octave;
    while let Some(&ch) = chars.peek() {
        match ch {
            '\'' => {
                octave += 1;
                chars.next();
            }
            ',' => {
                octave -= 1;
                chars.next();
            }
            _ => break,
        }
    }

    // Calculate MIDI pitch: (octave + 1) * 12 + semitone + accidental
    let midi_pitch = ((octave as i16 + 1) * 12 + semitones[note_index] as i16 + accidental as i16)
        .clamp(0, 127) as u8;

    // Parse duration
    let duration = parse_duration(chars, unit_length);

    Ok(Some((midi_pitch, duration)))
}

/// Parse duration modifier after a note
fn parse_duration(chars: &mut std::iter::Peekable<std::str::Chars>, unit_length: f64) -> f64 {
    let mut multiplier = 1.0;
    let mut has_number = false;

    // Check for number multiplier (e.g., C2 = double length)
    let mut num_str = String::new();
    while let Some(&ch) = chars.peek() {
        if ch.is_ascii_digit() {
            num_str.push(ch);
            chars.next();
        } else {
            break;
        }
    }

    if !num_str.is_empty() {
        multiplier = num_str.parse::<f64>().unwrap_or(1.0);
        has_number = true;
    }

    // Check for fraction (e.g., C/2 = half length, C3/2 = 1.5x)
    if chars.peek() == Some(&'/') {
        chars.next();
        let mut denom_str = String::new();
        while let Some(&ch) = chars.peek() {
            if ch.is_ascii_digit() {
                denom_str.push(ch);
                chars.next();
            } else {
                break;
            }
        }
        let denom = if denom_str.is_empty() {
            2.0 // C/ means C/2
        } else {
            denom_str.parse::<f64>().unwrap_or(2.0)
        };
        if denom != 0.0 {
            multiplier = if has_number { multiplier / denom } else { 1.0 / denom };
        }
    }

    // Check for broken rhythm (> or <)
    // > means this note is 1.5x, next is 0.5x (dotted)
    // < means this note is 0.5x, next is 1.5x
    if chars.peek() == Some(&'>') {
        chars.next();
        multiplier *= 1.5;
    } else if chars.peek() == Some(&'<') {
        chars.next();
        multiplier *= 0.5;
    }

    // Convert unit length to beats (quarter note = 1.0)
    // unit_length is fraction of a whole note
    // So L:1/8 = 0.125, which is 0.5 beats (eighth note)
    let beats_per_unit = unit_length * 4.0;

    beats_per_unit * multiplier
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_time_signature() {
        assert_eq!(parse_time_signature("4/4"), (4, 4));
        assert_eq!(parse_time_signature("3/4"), (3, 4));
        assert_eq!(parse_time_signature("6/8"), (6, 8));
        assert_eq!(parse_time_signature("C"), (4, 4));
        assert_eq!(parse_time_signature("C|"), (2, 2));
    }

    #[test]
    fn test_parse_fraction() {
        assert_eq!(parse_fraction("1/8"), Some(0.125));
        assert_eq!(parse_fraction("1/4"), Some(0.25));
        assert_eq!(parse_fraction("3/8"), Some(0.375));
    }

    #[test]
    fn test_parse_tempo() {
        assert_eq!(parse_tempo("120"), Some(120));
        assert_eq!(parse_tempo("1/4=120"), Some(120));
        assert_eq!(parse_tempo("1/8=160"), Some(160));
    }

    #[test]
    fn test_simple_melody() {
        let abc = r#"
X: 1
T: Test
M: 4/4
L: 1/4
K: C
C D E F | G A B c |
"#;
        let melody = AbcParser::parse_string(abc).unwrap();
        assert_eq!(melody.title, Some("Test".to_string()));
        assert_eq!(melody.time_signature, (4, 4));
        assert_eq!(melody.note_count(), 8);

        // Check pitches: C4, D4, E4, F4, G4, A4, B4, C5
        let pitches: Vec<u8> = melody.notes.iter().filter_map(|n| n.pitch).collect();
        assert_eq!(pitches, vec![60, 62, 64, 65, 67, 69, 71, 72]);
    }

    #[test]
    fn test_octave_modifiers() {
        let abc = r#"
X: 1
T: Octaves
M: 4/4
L: 1/4
K: C
C, C c c' |
"#;
        let melody = AbcParser::parse_string(abc).unwrap();
        let pitches: Vec<u8> = melody.notes.iter().filter_map(|n| n.pitch).collect();
        // C, = C3 (48), C = C4 (60), c = C5 (72), c' = C6 (84)
        assert_eq!(pitches, vec![48, 60, 72, 84]);
    }

    #[test]
    fn test_accidentals() {
        let abc = r#"
X: 1
T: Accidentals
M: 4/4
L: 1/4
K: C
C ^C _D =D |
"#;
        let melody = AbcParser::parse_string(abc).unwrap();
        let pitches: Vec<u8> = melody.notes.iter().filter_map(|n| n.pitch).collect();
        // C4 (60), C#4 (61), Db4 (61), D4 (62)
        assert_eq!(pitches, vec![60, 61, 61, 62]);
    }

    #[test]
    fn test_key_signature() {
        let abc = r#"
X: 1
T: G Major
M: 4/4
L: 1/4
K: G
F |
"#;
        let melody = AbcParser::parse_string(abc).unwrap();
        let pitches: Vec<u8> = melody.notes.iter().filter_map(|n| n.pitch).collect();
        // In G major, F should be F# (66)
        assert_eq!(pitches, vec![66]);
    }

    #[test]
    fn test_durations() {
        let abc = r#"
X: 1
T: Durations
M: 4/4
L: 1/8
K: C
C C2 C/2 C4 |
"#;
        let melody = AbcParser::parse_string(abc).unwrap();
        let durations: Vec<f64> = melody.notes.iter().map(|n| n.duration).collect();
        // L:1/8 = 0.5 beats
        // C = 0.5, C2 = 1.0, C/2 = 0.25, C4 = 2.0
        assert_eq!(durations, vec![0.5, 1.0, 0.25, 2.0]);
    }

    #[test]
    fn test_rests() {
        let abc = r#"
X: 1
T: Rests
M: 4/4
L: 1/4
K: C
C z D z2 |
"#;
        let melody = AbcParser::parse_string(abc).unwrap();
        assert_eq!(melody.notes.len(), 4);
        assert_eq!(melody.notes[0].pitch, Some(60)); // C
        assert_eq!(melody.notes[1].pitch, None); // rest
        assert_eq!(melody.notes[2].pitch, Some(62)); // D
        assert_eq!(melody.notes[3].pitch, None); // rest
    }

    #[test]
    fn test_happy_birthday() {
        let abc = r#"
X: 1
T:Happy Birthday to You
M:3/4
L:1/8
K:D
A>A | B2 A2 d2 | c4
"#;
        let melody = AbcParser::parse_string(abc).unwrap();
        assert_eq!(melody.title, Some("Happy Birthday to You".to_string()));
        assert_eq!(melody.key, Some("D".to_string()));
        assert!(melody.note_count() >= 5);
    }
}
