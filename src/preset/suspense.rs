//! Suspense mood preset
//!
//! Characteristics: Minor key, low drones, tremolo strings, dissonance

use super::{create_rng, MoodGenerator, PresetConfig, PresetVariation};
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
        let variation = PresetVariation::from_seed(config.seed);
        let mut rng = create_rng(config.seed);
        let mut sequences = Vec::new();

        let effective_tempo = variation.effective_tempo(config.tempo);
        let beats = config.duration_secs * effective_tempo as f64 / 60.0;

        // Choose instruments using variation system
        let drone_inst = variation.pick_instrument(0, DRONE_INSTRUMENTS);
        let tremolo_inst = variation.pick_instrument(1, TREMOLO_INSTRUMENTS);
        let hit_inst = variation.pick_instrument(2, HIT_INSTRUMENTS);

        // Layer 1: Low drone (always, but style varies)
        sequences.push(generate_drone(config, &variation, beats, effective_tempo, drone_inst, &mut rng));

        // Layer 2: High tremolo strings (probability + intensity based)
        let tremolo_threshold = 0.6 - config.intensity as f64 / 200.0;
        if variation.layer_probs[1] > tremolo_threshold {
            sequences.push(generate_tremolo(config, &variation, beats, effective_tempo, tremolo_inst, &mut rng));
        }

        // Layer 3: Sparse hits (probability + intensity based)
        let hit_threshold = 0.7 - config.intensity as f64 / 150.0;
        if variation.layer_probs[2] > hit_threshold {
            sequences.push(generate_sparse_hits(config, &variation, beats, effective_tempo, hit_inst, &mut rng));
        }

        // Layer 4: Sub-bass rumble (random chance)
        if variation.layer_probs[3] > 0.6 {
            sequences.push(generate_sub_bass(config, &variation, beats, effective_tempo, &mut rng));
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
    variation: &PresetVariation,
    beats: f64,
    tempo: u16,
    instrument: u8,
    rng: &mut impl Rng,
) -> NoteSequence {
    let root = config.key.root();
    let mut notes = Vec::new();

    // Drone style varies based on seed
    let style = variation.pick_style(0, 4);

    match style {
        0 => {
            // Simple sustained drone
            let fifth = root + 7;
            notes.push(Note::new(root - 24, beats, variation.adjust_velocity(45 + rng.gen_range(0..15)), 0.0));
            notes.push(Note::new(fifth - 24, beats, variation.adjust_velocity(35 + rng.gen_range(0..15)), 0.0));
        }
        1 => {
            // Pulsing drone
            let mut t = 0.0;
            while t < beats {
                let vel = variation.adjust_velocity(40 + rng.gen_range(0..20));
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
                let vel = variation.adjust_velocity(40 + rng.gen_range(0..15));
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
            notes.push(Note::new(root - 24, beats, variation.adjust_velocity(50 + rng.gen_range(0..10)), 0.0));
            notes.push(Note::new(tritone - 24, beats, variation.adjust_velocity(30 + rng.gen_range(0..10)), 0.0));
        }
    }

    NoteSequence::new(notes, instrument, tempo)
}

/// Accent patterns for tremolo dynamics
const ACCENT_PATTERNS: &[&[i8]] = &[
    &[10, -10],              // loud-soft
    &[-10, 10],              // soft-loud
    &[-10, -10, 10],         // soft-soft-loud
    &[10, 10, -10],          // loud-loud-soft
    &[10, 0, -10, 0],        // loud-mid-soft-mid
    &[-15, 0, 10, 15],       // crescendo
    &[15, 10, 0, -15],       // decrescendo
    &[10, -15, 10, -15],     // strong alternating
];

/// Generate tremolo pattern with melodic and dynamic variation
fn generate_tremolo(
    config: &PresetConfig,
    variation: &PresetVariation,
    beats: f64,
    tempo: u16,
    instrument: u8,
    rng: &mut impl Rng,
) -> NoteSequence {
    let root = config.key.root();
    let mut notes = Vec::new();

    // Use chromatic/tension scale for suspense
    let tension_intervals: &[u8] = &[0, 1, 3, 6, 7, 8, 11]; // Root, m2, m3, tritone, P5, m6, M7

    // Octave varies based on seed
    let octave_shift: i8 = match variation.pick_style(2, 3) {
        0 => 0,
        1 => 12,
        _ => 24,
    };

    // Tremolo speed varies based on seed
    let tremolo_speed = match variation.pick_style(3, 3) {
        0 => 0.0625, // 64th notes (very fast)
        1 => 0.125,  // 32nd notes
        _ => 0.25,   // 16th notes
    };

    // Number of tones in tremolo group (1-4)
    let tone_count = match variation.style_choices[4] % 4 {
        0 => 1,  // Single repeated note (simple tension)
        1 => 2,  // Classic two-note tremolo
        2 => 3,  // Three-note pattern
        _ => 4,  // Four-note pattern
    };

    // Select accent pattern based on seed
    let accent_pattern = ACCENT_PATTERNS[(variation.style_choices[5] as usize) % ACCENT_PATTERNS.len()];

    let velocity_base = 20 + (config.intensity as i32 / 5) as u8;

    // Get contour pattern for melodic movement
    let phrase_len = variation.phrase_length as usize;
    let contour = variation.get_contour(phrase_len);
    let mut scale_idx = (variation.scale_offset as usize) % tension_intervals.len();

    let mut t = 0.0;
    let mut phrase_pos = 0;
    let mut note_in_group = 0;
    let mut accent_pos = 0;

    while t < beats {
        // Build the current tone group
        let mut tone_pitches: Vec<u8> = Vec::new();
        for i in 0..tone_count {
            let idx = (scale_idx + i) % tension_intervals.len();
            let interval = tension_intervals[idx];
            let pitch = ((root as i8 + octave_shift) as u8).saturating_add(interval);
            tone_pitches.push(pitch);
        }

        // Play current note in group
        let pitch = tone_pitches[note_in_group % tone_pitches.len()];

        // Apply accent pattern to velocity
        let accent = accent_pattern[accent_pos % accent_pattern.len()];
        let vel_adjusted = (velocity_base as i16 + accent as i16).clamp(15, 100) as u8;
        let vel = variation.adjust_velocity(vel_adjusted + rng.gen_range(0..10));

        // Sometimes skip for rest (tension builder)
        if !variation.should_rest(rng) {
            notes.push(Note::new(pitch, tremolo_speed, vel, t));
        }

        t += tremolo_speed;
        note_in_group += 1;
        accent_pos += 1;

        // When we've cycled through the tone group, potentially change
        if note_in_group >= tone_count {
            note_in_group = 0;
            phrase_pos += 1;

            // Move through scale based on contour
            if phrase_pos >= phrase_len {
                phrase_pos = 0;
                let direction = contour[rng.gen_range(0..contour.len())];
                let step = variation.get_interval(rng) as usize;
                match direction {
                    1 => scale_idx = (scale_idx + step) % tension_intervals.len(),
                    -1 => scale_idx = if scale_idx >= step { scale_idx - step } else { tension_intervals.len() - 1 },
                    _ => {} // Stay on current
                }
            }
        }
    }

    NoteSequence::new(notes, instrument, tempo)
}

/// Generate sparse dissonant hits with variation
fn generate_sparse_hits(
    config: &PresetConfig,
    variation: &PresetVariation,
    beats: f64,
    tempo: u16,
    instrument: u8,
    rng: &mut impl Rng,
) -> NoteSequence {
    let root = config.key.root();
    let mut notes = Vec::new();

    // Cluster type varies based on seed
    let cluster: Vec<u8> = match variation.pick_style(4, 4) {
        0 => vec![root, root + 1, root + 6],           // Root + m2 + tritone
        1 => vec![root, root + 3, root + 6, root + 9], // Diminished
        2 => vec![root + 1, root + 5, root + 8],       // Random dissonance
        _ => vec![root, root + 11],                     // Root + major 7th
    };

    // Number of hits varies based on note_count_factor
    let num_hits = (1.0 + variation.note_count_factor * 4.0) as usize;
    let mut positions: Vec<f64> = (0..num_hits)
        .map(|_| rng.gen_range(0.25..beats - 0.25))
        .collect();
    positions.sort_by(|a, b| a.partial_cmp(b).unwrap());

    for pos in positions {
        let velocity = variation.adjust_velocity(50 + rng.gen_range(0..40));
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
    variation: &PresetVariation,
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
        let vel = variation.adjust_velocity(25 + rng.gen_range(0..15));
        let dur: f64 = rng.gen_range(0.5_f64..2.0_f64).min(beats - t);
        notes.push(Note::new(pitch, dur, vel, t));
        t += dur + rng.gen_range(0.0..1.0);
    }

    // Synth bass (38) or contrabass (43) based on seed
    let bass_instruments: &[u8] = &[38, 43];
    let instrument = variation.pick_instrument(3, bass_instruments);
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
