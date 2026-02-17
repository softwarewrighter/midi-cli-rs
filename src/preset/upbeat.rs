//! Upbeat mood preset
//!
//! Characteristics: Major key, rhythmic, energetic, clear pulse

use super::{MoodGenerator, PresetConfig, create_rng};
use crate::midi::{Note, NoteSequence};
use rand::Rng;

/// Upbeat mood generator
pub struct UpbeatPreset;

impl MoodGenerator for UpbeatPreset {
    fn generate(&self, config: &PresetConfig) -> Vec<NoteSequence> {
        let mut rng = create_rng(config.seed);
        let mut sequences = Vec::new();

        let beats = config.duration_secs * config.tempo as f64 / 60.0;

        // Layer 1: Rhythmic chord pattern (piano)
        sequences.push(generate_rhythm_pattern(config, beats, &mut rng));

        // Layer 2: Bass line
        sequences.push(generate_bass_line(config, beats, &mut rng));

        // Layer 3: Melody hint (if intensity > 50)
        if config.intensity > 50 {
            sequences.push(generate_melody_hint(config, beats, &mut rng));
        }

        sequences
    }

    fn name(&self) -> &'static str {
        "upbeat"
    }

    fn description(&self) -> &'static str {
        "Energetic, happy mood with rhythmic patterns and major harmony"
    }
}

/// Generate rhythmic chord pattern
fn generate_rhythm_pattern(config: &PresetConfig, beats: f64, rng: &mut impl Rng) -> NoteSequence {
    let chord = config.key.chord_tones();
    let mut notes = Vec::new();

    // Syncopated rhythm pattern
    let pattern = [0.0, 0.5, 1.0, 1.5, 2.5, 3.0, 3.5];
    let pattern_len = 4.0; // One bar

    let mut t = 0.0;
    while t < beats {
        for &offset in &pattern {
            let pos = t + offset;
            if pos >= beats {
                break;
            }

            let velocity = if offset == 0.0 || offset == 2.5 {
                80 + rng.gen_range(0..20) // Accented
            } else {
                60 + rng.gen_range(0..15)
            };

            let duration = if offset == 2.5 { 0.75 } else { 0.25 };

            // Play chord
            for &pitch in &chord {
                notes.push(Note::new(pitch, duration, velocity, pos));
            }
        }
        t += pattern_len;
    }

    // Bright piano (1)
    NoteSequence::new(notes, 1, config.tempo)
}

/// Generate bass line
fn generate_bass_line(config: &PresetConfig, beats: f64, rng: &mut impl Rng) -> NoteSequence {
    let root = config.key.root();
    let fifth = root + 7;
    let mut notes = Vec::new();

    // Simple root-fifth bass pattern
    let mut t = 0.0;
    let mut is_root = true;

    while t < beats {
        let pitch = if is_root { root - 12 } else { fifth - 12 };
        let velocity = 90 + rng.gen_range(0..10);
        let duration = 0.5;

        notes.push(Note::new(pitch, duration, velocity, t));

        is_root = !is_root;
        t += 0.5;
    }

    // Electric bass (33)
    NoteSequence::new(notes, 33, config.tempo)
}

/// Generate melody hint
fn generate_melody_hint(config: &PresetConfig, beats: f64, rng: &mut impl Rng) -> NoteSequence {
    let scale = config.key.scale_intervals();
    let root = config.key.root();
    let mut notes = Vec::new();

    // Short melodic fragment: 3-5 notes
    let num_notes = rng.gen_range(3..=5);
    let start_pos = beats * 0.6; // Start in last third

    for i in 0..num_notes {
        let interval = scale[rng.gen_range(0..scale.len())];
        let octave = if rng.gen_bool(0.3) { 12 } else { 0 };
        let pitch = root + interval + octave;

        let pos = start_pos + (i as f64 * 0.25);
        if pos >= beats {
            break;
        }

        let velocity = 70 + rng.gen_range(0..20);
        let duration = if i == num_notes - 1 { 0.5 } else { 0.25 };

        notes.push(Note::new(pitch, duration, velocity, pos));
    }

    // Synth lead (80)
    NoteSequence::new(notes, 80, config.tempo)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_upbeat_generates_sequences() {
        let config = PresetConfig {
            key: super::super::Key::C,
            tempo: 140, // Fast tempo
            ..Default::default()
        };
        let sequences = UpbeatPreset.generate(&config);
        assert!(!sequences.is_empty());
    }

    #[test]
    fn test_upbeat_uses_major_chord() {
        let config = PresetConfig {
            key: super::super::Key::C,
            ..Default::default()
        };
        let sequences = UpbeatPreset.generate(&config);

        // C major chord tones: C(60), E(64), G(67)
        let rhythm = &sequences[0];
        let pitches: Vec<u8> = rhythm.notes.iter().map(|n| n.pitch).collect();

        assert!(pitches.contains(&60)); // C
        assert!(pitches.contains(&64)); // E
        assert!(pitches.contains(&67)); // G
    }
}
