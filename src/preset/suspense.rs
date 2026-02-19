//! Suspense mood preset
//!
//! Characteristics: Minor key, low drones, tremolo strings, dissonance

use super::{create_rng, MoodGenerator, PresetConfig};
use crate::midi::{Note, NoteSequence};
use rand::Rng;

/// Suspense mood generator
pub struct SuspensePreset;

/// Drone instrument choices
const DRONE_INSTRUMENTS: &[u8] = &[
    42, // Cello
    43, // Contrabass
    70, // Bassoon
    71, // Clarinet (low register)
    89, // Pad (warm)
];

/// Tremolo instrument choices
const TREMOLO_INSTRUMENTS: &[u8] = &[
    44, // Tremolo strings
    48, // String ensemble 1
    49, // String ensemble 2
    92, // Pad (bowed)
];

/// Hit instrument choices
const HIT_INSTRUMENTS: &[u8] = &[
    0,   // Piano
    6,   // Harpsichord
    11,  // Vibraphone
    14,  // Tubular bells
    46,  // Orchestral harp
];

impl MoodGenerator for SuspensePreset {
    fn generate(&self, config: &PresetConfig) -> Vec<NoteSequence> {
        let mut rng = create_rng(config.seed);
        let mut sequences = Vec::new();

        // Tempo variation: ±10%
        let tempo_var = 1.0 + (rng.gen_range(-10..=10) as f64 / 100.0);
        let effective_tempo = ((config.tempo as f64 * tempo_var) as u16).max(40).min(120);

        let beats = config.duration_secs * effective_tempo as f64 / 60.0;

        // Choose instruments for this seed
        let drone_inst = DRONE_INSTRUMENTS[rng.gen_range(0..DRONE_INSTRUMENTS.len())];
        let tremolo_inst = TREMOLO_INSTRUMENTS[rng.gen_range(0..TREMOLO_INSTRUMENTS.len())];
        let hit_inst = HIT_INSTRUMENTS[rng.gen_range(0..HIT_INSTRUMENTS.len())];

        // Layer 1: Low drone (always, but style varies)
        sequences.push(generate_drone(config, beats, effective_tempo, drone_inst, &mut rng));

        // Layer 2: High tremolo strings (probability + intensity based)
        let tremolo_prob = 0.4 + (config.intensity as f64 / 200.0);
        if rng.gen_bool(tremolo_prob) {
            sequences.push(generate_tremolo(config, beats, effective_tempo, tremolo_inst, &mut rng));
        }

        // Layer 3: Sparse hits (probability + intensity based)
        let hit_prob = 0.2 + (config.intensity as f64 / 150.0);
        if rng.gen_bool(hit_prob) {
            sequences.push(generate_sparse_hits(config, beats, effective_tempo, hit_inst, &mut rng));
        }

        // Layer 4: Sub-bass rumble (random chance)
        if rng.gen_bool(0.3) {
            sequences.push(generate_sub_bass(config, beats, effective_tempo, &mut rng));
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

/// Generate low sustained drone with variation
fn generate_drone(
    config: &PresetConfig,
    beats: f64,
    tempo: u16,
    instrument: u8,
    rng: &mut impl Rng,
) -> NoteSequence {
    let root = config.key.root();
    let mut notes = Vec::new();

    // Drone style varies
    let style = rng.gen_range(0..4);

    match style {
        0 => {
            // Simple sustained drone
            let fifth = root + 7;
            notes.push(Note::new(root - 24, beats, 45 + rng.gen_range(0..15), 0.0));
            notes.push(Note::new(fifth - 24, beats, 35 + rng.gen_range(0..15), 0.0));
        }
        1 => {
            // Pulsing drone
            let mut t = 0.0;
            while t < beats {
                let vel = 40 + rng.gen_range(0..20);
                let dur: f64 = rng.gen_range(1.5..3.0);
                notes.push(Note::new(root - 24, dur.min(beats - t), vel, t));
                t += dur + rng.gen_range(0.0..0.5);
            }
        }
        2 => {
            // Octave drone with movement
            let mut t = 0.0;
            let mut current_octave = -24i8;
            while t < beats {
                let pitch = (root as i8 + current_octave) as u8;
                let vel = 40 + rng.gen_range(0..15);
                let dur: f64 = rng.gen_range(2.0_f64..4.0_f64).min(beats - t);
                notes.push(Note::new(pitch, dur, vel, t));
                t += dur;
                // Sometimes shift octave
                if rng.gen_bool(0.3) {
                    current_octave = if current_octave == -24 { -12 } else { -24 };
                }
            }
        }
        _ => {
            // Tritone tension drone
            let tritone = root + 6;
            notes.push(Note::new(root - 24, beats, 50 + rng.gen_range(0..10), 0.0));
            notes.push(Note::new(tritone - 24, beats, 30 + rng.gen_range(0..10), 0.0));
        }
    }

    NoteSequence::new(notes, instrument, tempo)
}

/// Generate tremolo pattern with variation
fn generate_tremolo(
    config: &PresetConfig,
    beats: f64,
    tempo: u16,
    instrument: u8,
    rng: &mut impl Rng,
) -> NoteSequence {
    let root = config.key.root();
    let mut notes = Vec::new();

    // Tremolo interval varies
    let interval: u8 = match rng.gen_range(0..4) {
        0 => 1,  // Minor second (very tense)
        1 => 2,  // Major second
        2 => 6,  // Tritone
        _ => 11, // Major 7th
    };

    // Octave varies
    let octave_shift: i8 = match rng.gen_range(0..3) {
        0 => 0,
        1 => 12,
        _ => 24,
    };

    let note1 = (root as i8 + octave_shift) as u8;
    let note2 = note1 + interval;

    // Tremolo speed varies
    let tremolo_speed = match rng.gen_range(0..3) {
        0 => 0.0625, // 64th notes (very fast)
        1 => 0.125,  // 32nd notes
        _ => 0.25,   // 16th notes
    };

    let velocity_base = 20 + (config.intensity as i32 / 5) as u8;

    let mut t = 0.0;
    while t < beats {
        let vel = velocity_base + rng.gen_range(0..15);
        notes.push(Note::new(note1, tremolo_speed, vel, t));
        t += tremolo_speed;

        if t < beats {
            let vel = velocity_base + rng.gen_range(0..15);
            notes.push(Note::new(note2, tremolo_speed, vel, t));
            t += tremolo_speed;
        }
    }

    NoteSequence::new(notes, instrument, tempo)
}

/// Generate sparse dissonant hits with variation
fn generate_sparse_hits(
    config: &PresetConfig,
    beats: f64,
    tempo: u16,
    instrument: u8,
    rng: &mut impl Rng,
) -> NoteSequence {
    let root = config.key.root();
    let mut notes = Vec::new();

    // Cluster type varies
    let cluster: Vec<u8> = match rng.gen_range(0..4) {
        0 => vec![root, root + 1, root + 6],           // Root + m2 + tritone
        1 => vec![root, root + 3, root + 6, root + 9], // Diminished
        2 => vec![root + 1, root + 5, root + 8],       // Random dissonance
        _ => vec![root, root + 11],                     // Root + major 7th
    };

    // Number of hits varies more
    let num_hits = rng.gen_range(1..=5);
    let mut positions: Vec<f64> = (0..num_hits)
        .map(|_| rng.gen_range(0.25..beats - 0.25))
        .collect();
    positions.sort_by(|a, b| a.partial_cmp(b).unwrap());

    for pos in positions {
        let velocity = 50 + rng.gen_range(0..40);
        let duration = rng.gen_range(0.3..1.0);
        for &pitch in &cluster {
            notes.push(Note::new(pitch, duration, velocity, pos));
        }
    }

    NoteSequence::new(notes, instrument, tempo)
}

/// Generate sub-bass rumble
fn generate_sub_bass(
    config: &PresetConfig,
    beats: f64,
    tempo: u16,
    rng: &mut impl Rng,
) -> NoteSequence {
    let root = config.key.root();
    let mut notes = Vec::new();

    // Very low, quiet rumble
    let pitch = root.saturating_sub(36).max(24); // 3 octaves down, min MIDI 24

    let mut t = 0.0;
    while t < beats {
        let vel = 25 + rng.gen_range(0..15);
        let dur: f64 = rng.gen_range(0.5_f64..2.0_f64).min(beats - t);
        notes.push(Note::new(pitch, dur, vel, t));
        t += dur + rng.gen_range(0.0..1.0);
    }

    // Synth bass (38) or contrabass (43)
    let instrument = if rng.gen_bool(0.5) { 38 } else { 43 };
    NoteSequence::new(notes, instrument, tempo)
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
            assert!(note.pitch < 72, "Drone notes should be low");
        }
    }

    #[test]
    fn test_suspense_adjacent_seeds_differ() {
        let config1 = PresetConfig {
            seed: 42,
            duration_secs: 5.0,
            ..Default::default()
        };
        let config2 = PresetConfig {
            seed: 43,
            duration_secs: 5.0,
            ..Default::default()
        };

        let seq1 = SuspensePreset.generate(&config1);
        let seq2 = SuspensePreset.generate(&config2);

        // Count differences
        let mut diffs = 0;
        if seq1.len() != seq2.len() {
            diffs += 5;
        }
        if !seq1.is_empty() && !seq2.is_empty() && seq1[0].instrument != seq2[0].instrument {
            diffs += 3;
        }
        let notes1: usize = seq1.iter().map(|s| s.notes.len()).sum();
        let notes2: usize = seq2.iter().map(|s| s.notes.len()).sum();
        if notes1 != notes2 {
            diffs += 1;
        }

        assert!(diffs >= 1, "Adjacent seeds should differ");
    }

    #[test]
    fn test_suspense_instruments_vary() {
        let instruments: Vec<u8> = (1..=15)
            .map(|seed| {
                let config = PresetConfig {
                    seed,
                    duration_secs: 3.0,
                    ..Default::default()
                };
                let seqs = SuspensePreset.generate(&config);
                seqs[0].instrument
            })
            .collect();

        let unique: std::collections::HashSet<_> = instruments.iter().collect();
        assert!(unique.len() > 1, "Instruments should vary across seeds");
    }
}
