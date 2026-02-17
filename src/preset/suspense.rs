//! Suspense mood preset
//!
//! Characteristics: Minor key, low drones, tremolo strings, dissonance

use super::{MoodGenerator, PresetConfig, create_rng};
use crate::midi::{Note, NoteSequence};
use rand::Rng;

/// Suspense mood generator
pub struct SuspensePreset;

impl MoodGenerator for SuspensePreset {
    fn generate(&self, config: &PresetConfig) -> Vec<NoteSequence> {
        let mut rng = create_rng(config.seed);
        let mut sequences = Vec::new();

        let beats = config.duration_secs * config.tempo as f64 / 60.0;

        // Layer 1: Low drone (cello/contrabass)
        sequences.push(generate_drone(config, beats));

        // Layer 2: High tremolo strings (if intensity > 30)
        if config.intensity > 30 {
            sequences.push(generate_tremolo(config, beats, &mut rng));
        }

        // Layer 3: Sparse piano hits (if intensity > 60)
        if config.intensity > 60 {
            sequences.push(generate_sparse_hits(config, beats, &mut rng));
        }

        sequences
    }

    fn name(&self) -> &'static str {
        "suspense"
    }

    fn description(&self) -> &'static str {
        "Tense, anxious mood with low drones and tremolo strings"
    }
}

/// Generate low sustained drone
fn generate_drone(config: &PresetConfig, beats: f64) -> NoteSequence {
    let root = config.key.root();
    let fifth = root + 7;

    // Drone notes: root and fifth, 2 octaves down
    let notes = vec![
        Note::new(root - 24, beats, 50, 0.0),  // Root, very low
        Note::new(fifth - 24, beats, 40, 0.0), // Fifth, very low
    ];

    // Use cello (42) or contrabass (43)
    NoteSequence::new(notes, 42, config.tempo)
}

/// Generate high tremolo pattern
fn generate_tremolo(config: &PresetConfig, beats: f64, rng: &mut impl Rng) -> NoteSequence {
    let root = config.key.root();
    let mut notes = Vec::new();

    // Tremolo between root and minor second (creates tension)
    let note1 = root + 12; // One octave up
    let note2 = root + 13; // Minor second above

    let tremolo_speed = 0.125; // 32nd notes
    let velocity_base = 25 + (config.intensity as i32 / 4) as u8;

    let mut t = 0.0;
    while t < beats {
        let vel = velocity_base + rng.gen_range(0..10);
        notes.push(Note::new(note1, tremolo_speed, vel, t));
        t += tremolo_speed;

        if t < beats {
            let vel = velocity_base + rng.gen_range(0..10);
            notes.push(Note::new(note2, tremolo_speed, vel, t));
            t += tremolo_speed;
        }
    }

    // Tremolo strings (44)
    NoteSequence::new(notes, 44, config.tempo)
}

/// Generate sparse dissonant piano hits
fn generate_sparse_hits(config: &PresetConfig, beats: f64, rng: &mut impl Rng) -> NoteSequence {
    let root = config.key.root();
    let mut notes = Vec::new();

    // Dissonant cluster: root + minor 2nd + tritone
    let cluster = [root, root + 1, root + 6];

    // Place 1-3 hits randomly
    let num_hits = rng.gen_range(1..=3);
    let mut positions: Vec<f64> = (0..num_hits)
        .map(|_| rng.gen_range(0.5..beats - 0.5))
        .collect();
    positions.sort_by(|a, b| a.partial_cmp(b).unwrap());

    for pos in positions {
        let velocity = 60 + rng.gen_range(0..30);
        for &pitch in &cluster {
            notes.push(Note::new(pitch, 0.5, velocity, pos));
        }
    }

    // Piano (0)
    NoteSequence::new(notes, 0, config.tempo)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_suspense_generates_sequences() {
        let config = PresetConfig::default();
        let sequences = SuspensePreset.generate(&config);
        assert!(!sequences.is_empty());
    }

    #[test]
    fn test_suspense_drone_is_low() {
        let config = PresetConfig::default();
        let sequences = SuspensePreset.generate(&config);

        // First sequence should be the drone
        let drone = &sequences[0];
        for note in &drone.notes {
            // Should be 2 octaves below middle C area
            assert!(note.pitch < 60, "Drone notes should be low");
        }
    }

    #[test]
    fn test_suspense_intensity_affects_layers() {
        let low_intensity = PresetConfig {
            intensity: 20,
            ..Default::default()
        };
        let high_intensity = PresetConfig {
            intensity: 80,
            ..Default::default()
        };

        let low_seq = SuspensePreset.generate(&low_intensity);
        let high_seq = SuspensePreset.generate(&high_intensity);

        // Higher intensity should have more layers
        assert!(high_seq.len() > low_seq.len());
    }
}
