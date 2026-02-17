//! Ambient mood preset
//!
//! Characteristics: Textural, non-rhythmic, drones, evolving, pentatonic

use super::{MoodGenerator, PresetConfig, create_rng};
use crate::midi::{Note, NoteSequence};
use rand::Rng;

/// Ambient mood generator
pub struct AmbientPreset;

impl MoodGenerator for AmbientPreset {
    fn generate(&self, config: &PresetConfig) -> Vec<NoteSequence> {
        let mut rng = create_rng(config.seed);
        let mut sequences = Vec::new();

        let beats = config.duration_secs * config.tempo as f64 / 60.0;

        // Layer 1: Multi-layered drone
        sequences.push(generate_drone_layer(config, beats, 0));
        sequences.push(generate_drone_layer(config, beats, 7)); // Fifth above

        // Layer 2: Sporadic pentatonic tones
        sequences.push(generate_sporadic_tones(config, beats, &mut rng));

        sequences
    }

    fn name(&self) -> &'static str {
        "ambient"
    }

    fn description(&self) -> &'static str {
        "Atmospheric, textural mood with drones and sparse pentatonic tones"
    }
}

/// Generate drone layer
fn generate_drone_layer(config: &PresetConfig, beats: f64, interval: u8) -> NoteSequence {
    let root = config.key.root();
    let pitch = root - 12 + interval; // One octave below middle

    // Single sustained note
    let notes = vec![Note::new(pitch, beats, 35, 0.0)];

    // Strings (48) for rich sustain
    NoteSequence::new(notes, 48, config.tempo)
}

/// Generate sporadic pentatonic tones
fn generate_sporadic_tones(config: &PresetConfig, beats: f64, rng: &mut impl Rng) -> NoteSequence {
    let root = config.key.root();
    let mut notes = Vec::new();

    // Pentatonic scale (works well for ambient)
    let pentatonic = if config.key.is_minor() {
        [0, 3, 5, 7, 10] // Minor pentatonic
    } else {
        [0, 2, 4, 7, 9] // Major pentatonic
    };

    // Sparse, random placement
    let num_tones = 3 + (config.intensity as usize / 30);
    let mut positions: Vec<f64> = (0..num_tones)
        .map(|_| rng.gen_range(0.5..beats - 1.0))
        .collect();
    positions.sort_by(|a, b| a.partial_cmp(b).unwrap());

    for pos in positions {
        let interval = pentatonic[rng.gen_range(0..pentatonic.len())];
        let octave = rng.gen_range(0..=2) * 12;
        let pitch = root + interval + octave;

        let velocity = 25 + rng.gen_range(0..15);
        let duration = rng.gen_range(1.5..3.0);

        notes.push(Note::new(pitch, duration, velocity, pos));
    }

    // Vibraphone (11) for bell-like ambient tones
    NoteSequence::new(notes, 11, config.tempo)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ambient_generates_sequences() {
        let config = PresetConfig::default();
        let sequences = AmbientPreset.generate(&config);
        assert_eq!(sequences.len(), 3); // 2 drone layers + sporadic tones
    }

    #[test]
    fn test_ambient_drones_are_sustained() {
        let config = PresetConfig {
            duration_secs: 5.0,
            tempo: 60,
            ..Default::default()
        };
        let sequences = AmbientPreset.generate(&config);

        // First two sequences are drones
        for drone in &sequences[0..2] {
            assert_eq!(drone.notes.len(), 1);
            assert!(drone.notes[0].duration >= 4.0);
        }
    }

    #[test]
    fn test_ambient_uses_pentatonic() {
        let config = PresetConfig {
            key: super::super::Key::C,
            ..Default::default()
        };
        let sequences = AmbientPreset.generate(&config);

        // Sporadic tones (last sequence)
        let tones = &sequences[2];
        for note in &tones.notes {
            let pitch_mod = note.pitch % 12;
            // C major pentatonic: C(0), D(2), E(4), G(7), A(9)
            assert!(
                [0, 2, 4, 7, 9].contains(&pitch_mod),
                "Note {} should be in pentatonic scale",
                note.pitch
            );
        }
    }
}
