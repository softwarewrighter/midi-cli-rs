//! Eerie mood preset
//!
//! Characteristics: Sparse, wide intervals, diminished harmony, ethereal

use super::{create_rng, MoodGenerator, PresetConfig, PresetVariation};
use crate::midi::{Note, NoteSequence};
use rand::Rng;

/// Eerie mood generator
pub struct EeriePreset;

/// Pad instrument choices
const PAD_INSTRUMENTS: &[u8] = &[
    89,  // Pad (warm)
    91,  // Pad (bowed)
    94,  // Pad (halo)
    95,  // Pad (sweep)
    52,  // Choir Aahs
];

/// Bell instrument choices
const BELL_INSTRUMENTS: &[u8] = &[
    8,   // Celesta
    14,  // Tubular Bells
    98,  // Crystal
    10,  // Glockenspiel
    112, // Tinkle Bell
];

/// Texture instrument choices
const TEXTURE_INSTRUMENTS: &[u8] = &[
    99,  // Atmosphere
    97,  // Soundtrack
    100, // Brightness
    122, // Seashore
    77,  // Shakuhachi
];

impl MoodGenerator for EeriePreset {
    fn generate(&self, config: &PresetConfig) -> Vec<NoteSequence> {
        let variation = PresetVariation::from_seed(config.seed);
        let mut rng = create_rng(config.seed);
        let mut sequences = Vec::new();

        let effective_tempo = variation.effective_tempo(config.tempo);
        let beats = config.duration_secs * effective_tempo as f64 / 60.0;

        // Choose instruments
        let pad_inst = variation.pick_instrument(0, PAD_INSTRUMENTS);
        let bell_inst = variation.pick_instrument(1, BELL_INSTRUMENTS);
        let texture_inst = variation.pick_instrument(2, TEXTURE_INSTRUMENTS);

        // Layer 1: Pad (always, but chord type varies)
        sequences.push(generate_pad(config, &variation, beats, effective_tempo, pad_inst, &mut rng));

        // Layer 2: Bell tones (high probability)
        if variation.layer_probs[1] > 0.2 {
            sequences.push(generate_bell_tones(config, &variation, beats, effective_tempo, bell_inst, &mut rng));
        }

        // Layer 3: Breath/texture (varies with intensity + variation)
        if variation.layer_probs[2] > (0.6 - config.intensity as f64 / 150.0) {
            sequences.push(generate_breath_texture(config, &variation, beats, effective_tempo, texture_inst, &mut rng));
        }

        // Layer 4: Dissonant stabs (random)
        if variation.layer_probs[3] > 0.6 {
            sequences.push(generate_stabs(config, &variation, beats, effective_tempo, &mut rng));
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

/// Generate pad chord with variation
fn generate_pad(
    config: &PresetConfig,
    variation: &PresetVariation,
    beats: f64,
    tempo: u16,
    instrument: u8,
    rng: &mut impl Rng,
) -> NoteSequence {
    let root = config.key.root();
    let mut notes = Vec::new();

    // Chord type varies
    let chord_type = variation.pick_style(0, 4);

    let chord_notes: Vec<(i8, u8)> = match chord_type {
        0 => {
            // Diminished
            vec![(-12, 40), (3, 35), (6 + 12, 30)]
        }
        1 => {
            // Minor with tritone
            vec![(-12, 40), (3, 35), (6, 30), (15, 25)]
        }
        2 => {
            // Cluster
            vec![(-12, 40), (1, 30), (2, 30)]
        }
        _ => {
            // Augmented
            vec![(-12, 40), (4, 35), (8, 35)]
        }
    };

    for (interval, base_vel) in chord_notes {
        let pitch = (root as i8 + interval) as u8;
        let vel = variation.adjust_velocity(base_vel + rng.gen_range(0..10));
        notes.push(Note::new(pitch, beats, vel, 0.0));
    }

    NoteSequence::new(notes, instrument, tempo)
}

/// Generate bell tones with variation
fn generate_bell_tones(
    config: &PresetConfig,
    variation: &PresetVariation,
    beats: f64,
    tempo: u16,
    instrument: u8,
    rng: &mut impl Rng,
) -> NoteSequence {
    let root = config.key.root();
    let mut notes = Vec::new();

    // Scale varies
    let scale: &[u8] = match variation.pick_style(1, 3) {
        0 => &[0, 2, 3, 5, 6, 8, 9, 11],  // Diminished
        1 => &[0, 1, 4, 5, 8, 9],          // Augmented
        _ => &[0, 1, 3, 6, 7, 9],          // Locrian-ish
    };

    // Number of notes varies
    let num_notes = (2.0 + variation.note_count_factor * 3.0) as usize;

    for i in 0..num_notes {
        let interval = scale[rng.gen_range(0..scale.len())];
        let octave_offset = rng.gen_range(1..=3) * 12;
        let pitch = root + interval + octave_offset;

        let position = (i as f64 / num_notes as f64) * beats * 0.85;
        let velocity = variation.adjust_velocity(25 + rng.gen_range(0..20));
        let duration = rng.gen_range(0.8_f64..2.5_f64);

        notes.push(Note::new(pitch, duration, velocity, position));
    }

    NoteSequence::new(notes, instrument, tempo)
}

/// Generate breath/texture with variation
fn generate_breath_texture(
    config: &PresetConfig,
    variation: &PresetVariation,
    beats: f64,
    tempo: u16,
    instrument: u8,
    rng: &mut impl Rng,
) -> NoteSequence {
    let root = config.key.root();
    let mut notes = Vec::new();

    // Movement style varies
    let style = variation.pick_style(2, 3);

    let step_duration = match style {
        0 => 0.25,
        1 => 0.5,
        _ => 0.75,
    };

    let mut current_pitch = root as i8;
    let mut t = 0.0;

    while t < beats {
        let pitch = current_pitch.clamp(root as i8 - 8, root as i8 + 8) as u8;
        let velocity = variation.adjust_velocity(12 + rng.gen_range(0..12));

        notes.push(Note::new(pitch, step_duration, velocity, t));

        // Movement varies
        current_pitch += match style {
            0 => rng.gen_range(-1..=1),
            1 => rng.gen_range(-2..=2),
            _ => if rng.gen_bool(0.7) { 0 } else { rng.gen_range(-3..=3) },
        };
        t += step_duration;
    }

    NoteSequence::new(notes, instrument, tempo)
}

/// Generate dissonant stabs
fn generate_stabs(
    config: &PresetConfig,
    variation: &PresetVariation,
    beats: f64,
    tempo: u16,
    rng: &mut impl Rng,
) -> NoteSequence {
    let root = config.key.root();
    let mut notes = Vec::new();

    // Cluster type
    let cluster: Vec<u8> = match variation.pick_style(3, 3) {
        0 => vec![root, root + 1, root + 6],
        1 => vec![root, root + 5, root + 11],
        _ => vec![root, root + 3, root + 6, root + 9],
    };

    let num_stabs = rng.gen_range(1..=3);
    let mut positions: Vec<f64> = (0..num_stabs)
        .map(|_| rng.gen_range(0.3_f64..beats - 0.3))
        .collect();
    positions.sort_by(|a, b| a.partial_cmp(b).unwrap());

    for pos in positions {
        let vel = variation.adjust_velocity(50 + rng.gen_range(0..30));
        let dur = rng.gen_range(0.2_f64..0.5_f64);
        for &pitch in &cluster {
            notes.push(Note::new(pitch, dur, vel, pos));
        }
    }

    // Piano or harpsichord for sharp attack
    let instrument = if rng.gen_bool(0.5) { 0 } else { 6 };
    NoteSequence::new(notes, instrument, tempo)
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
    fn test_eerie_adjacent_seeds_differ() {
        let config1 = PresetConfig { seed: 42, duration_secs: 5.0, ..Default::default() };
        let config2 = PresetConfig { seed: 43, duration_secs: 5.0, ..Default::default() };

        let seq1 = EeriePreset.generate(&config1);
        let seq2 = EeriePreset.generate(&config2);

        let mut diffs = 0;
        if seq1.len() != seq2.len() { diffs += 5; }
        if !seq1.is_empty() && !seq2.is_empty() && seq1[0].instrument != seq2[0].instrument { diffs += 3; }

        assert!(diffs >= 1 || seq1.len() != seq2.len(), "Adjacent seeds should differ");
    }

    #[test]
    fn test_eerie_instruments_vary() {
        let instruments: Vec<u8> = (1..=15)
            .map(|seed| {
                let config = PresetConfig { seed, duration_secs: 3.0, ..Default::default() };
                let seqs = EeriePreset.generate(&config);
                seqs[0].instrument
            })
            .collect();

        let unique: std::collections::HashSet<_> = instruments.iter().collect();
        assert!(unique.len() > 1, "Instruments should vary across seeds");
    }
}
