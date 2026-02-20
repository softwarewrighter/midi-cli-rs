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

/// Generate bell tones with melodic variation
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

    // Scale varies - all have eerie quality
    let scale: &[u8] = match variation.pick_style(1, 3) {
        0 => &[0, 2, 3, 5, 6, 8, 9, 11],  // Diminished
        1 => &[0, 1, 4, 5, 8, 9],          // Augmented
        _ => &[0, 1, 3, 6, 7, 9],          // Locrian-ish
    };

    // Number of notes varies
    let num_notes = (2.0 + variation.note_count_factor * 3.0) as usize;

    // Get contour for melodic movement
    let contour = variation.get_contour(num_notes);
    let mut scale_idx = (variation.scale_offset as usize) % scale.len();

    // Octave starting point varies by seed
    let base_octave: u8 = match variation.style_choices[2] % 3 {
        0 => 12,
        1 => 24,
        _ => 36,
    };
    let mut current_octave = base_octave;

    for i in 0..num_notes {
        // Skip some notes for rests (eerie sparse feeling)
        if variation.should_rest(rng) {
            continue;
        }

        let interval = scale[scale_idx % scale.len()];
        let pitch = root + interval + current_octave;

        let position = (i as f64 / num_notes as f64) * beats * 0.85;
        let velocity = variation.adjust_velocity(25 + rng.gen_range(0..20));
        let duration = rng.gen_range(0.8_f64..2.5_f64);

        notes.push(Note::new(pitch, duration, velocity, position));

        // Move through scale based on contour
        let direction = contour[i % contour.len()];
        let step = variation.get_interval(rng) as usize;
        match direction {
            1 => {
                scale_idx = (scale_idx + step) % scale.len();
                // Occasionally jump up an octave
                if rng.gen_bool(0.2) && current_octave < 36 {
                    current_octave += 12;
                }
            }
            -1 => {
                scale_idx = if scale_idx >= step { scale_idx - step } else { scale.len() - 1 };
                // Occasionally drop an octave
                if rng.gen_bool(0.2) && current_octave > 12 {
                    current_octave -= 12;
                }
            }
            _ => {} // Stay
        }
    }

    NoteSequence::new(notes, instrument, tempo)
}

/// Generate breath/texture with melodic contour variation
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

    // Use chromatic scale for eerie crawling texture
    let chromatic: &[i8] = &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11];

    // Get contour for movement direction
    let phrase_len = variation.phrase_length as usize;
    let contour = variation.get_contour(phrase_len);
    let mut scale_idx = (variation.scale_offset as usize) % chromatic.len();
    let mut phrase_pos = 0;

    let mut t = 0.0;

    while t < beats {
        // Skip for rests
        if variation.should_rest(rng) {
            t += step_duration;
            continue;
        }

        let interval = chromatic[scale_idx % chromatic.len()];
        let pitch = ((root as i8 + interval).clamp(root as i8 - 8, root as i8 + 12)) as u8;
        let velocity = variation.adjust_velocity(12 + rng.gen_range(0..12));

        notes.push(Note::new(pitch, step_duration, velocity, t));

        // Move based on contour
        let direction = contour[phrase_pos % contour.len()];
        let step = variation.get_interval(rng) as usize;
        match direction {
            1 => scale_idx = (scale_idx + step) % chromatic.len(),
            -1 => scale_idx = if scale_idx >= step { scale_idx - step } else { chromatic.len() - step },
            _ => {
                // Occasional micro-movement on "hold"
                if rng.gen_bool(0.3) {
                    scale_idx = (scale_idx + 1) % chromatic.len();
                }
            }
        }

        phrase_pos += 1;
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
