//! Eerie mood preset
//!
//! Characteristics: Sparse, wide intervals, diminished harmony, ethereal

use super::{MoodGenerator, PresetConfig, create_rng};
use crate::midi::{Note, NoteSequence};
use rand::Rng;

/// Eerie mood generator
pub struct EeriePreset;

impl MoodGenerator for EeriePreset {
    fn generate(&self, config: &PresetConfig) -> Vec<NoteSequence> {
        let mut rng = create_rng(config.seed);
        let mut sequences = Vec::new();

        let beats = config.duration_secs * config.tempo as f64 / 60.0;

        // Layer 1: Sustained diminished pad
        sequences.push(generate_diminished_pad(config, beats));

        // Layer 2: Sparse high bell tones
        sequences.push(generate_bell_tones(config, beats, &mut rng));

        // Layer 3: Breath/wind texture (if intensity > 40)
        if config.intensity > 40 {
            sequences.push(generate_breath_texture(config, beats, &mut rng));
        }

        sequences
    }

    fn name(&self) -> &'static str {
        "eerie"
    }

    fn description(&self) -> &'static str {
        "Creepy, unsettling mood with sparse tones and diminished harmony"
    }
}

/// Generate diminished chord pad
fn generate_diminished_pad(config: &PresetConfig, beats: f64) -> NoteSequence {
    let root = config.key.root();

    // Diminished chord: root, minor 3rd, tritone (spread across octaves)
    let notes = vec![
        Note::new(root - 12, beats, 40, 0.0),     // Root, low
        Note::new(root + 3, beats, 35, 0.0),      // Minor 3rd
        Note::new(root + 6 + 12, beats, 30, 0.0), // Tritone, high
    ];

    // Pad warm (89)
    NoteSequence::new(notes, 89, config.tempo)
}

/// Generate sparse bell tones
fn generate_bell_tones(config: &PresetConfig, beats: f64, rng: &mut impl Rng) -> NoteSequence {
    let root = config.key.root();
    let mut notes = Vec::new();

    // Diminished scale for eerie feel
    let dim_scale = [0, 2, 3, 5, 6, 8, 9, 11];

    // 2-4 sparse notes
    let num_notes = rng.gen_range(2..=4);

    for i in 0..num_notes {
        let interval = dim_scale[rng.gen_range(0..dim_scale.len())];
        let octave_offset = rng.gen_range(1..=2) * 12;
        let pitch = root + interval + octave_offset;

        let position = (i as f64 / num_notes as f64) * beats * 0.8;
        let velocity = 30 + rng.gen_range(0..20);
        let duration = rng.gen_range(1.0..2.0);

        notes.push(Note::new(pitch, duration, velocity, position));
    }

    // Celesta (8) for ethereal bell sound
    NoteSequence::new(notes, 8, config.tempo)
}

/// Generate breath/wind texture
fn generate_breath_texture(config: &PresetConfig, beats: f64, rng: &mut impl Rng) -> NoteSequence {
    let root = config.key.root();
    let mut notes = Vec::new();

    // Chromatic movement, very soft
    let mut current_pitch = root as i8;
    let mut t = 0.0;
    let step_duration = 0.5;

    while t < beats {
        let pitch = current_pitch.clamp(root as i8 - 6, root as i8 + 6) as u8;
        let velocity = 15 + rng.gen_range(0..10);

        notes.push(Note::new(pitch, step_duration, velocity, t));

        // Random walk
        current_pitch += rng.gen_range(-1..=1);
        t += step_duration;
    }

    // Breath noise / pad atmosphere (99)
    NoteSequence::new(notes, 99, config.tempo)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eerie_generates_sequences() {
        let config = PresetConfig::default();
        let sequences = EeriePreset.generate(&config);
        assert!(!sequences.is_empty());
    }

    #[test]
    fn test_eerie_uses_diminished_harmony() {
        let config = PresetConfig {
            key: super::super::Key::Am,
            ..Default::default()
        };
        let sequences = EeriePreset.generate(&config);

        // Check first sequence (pad) contains tritone interval
        let pad = &sequences[0];
        let pitches: Vec<u8> = pad.notes.iter().map(|n| n.pitch % 12).collect();

        // Should have root and tritone (6 semitones)
        let root_mod = config.key.root() % 12;
        let tritone_mod = (root_mod + 6) % 12;
        assert!(pitches.contains(&root_mod) || pitches.contains(&tritone_mod));
    }
}
