//! Calm mood preset
//!
//! Characteristics: Major/modal, slow, sustained pads, gentle arpeggios

use super::{create_rng, MoodGenerator, PresetConfig, PresetVariation};
use crate::midi::{Note, NoteSequence};
use rand::Rng;

/// Calm mood generator
pub struct CalmPreset;

/// Pad instrument choices
const PAD_INSTRUMENTS: &[u8] = &[
    89,  // Pad (warm)
    88,  // Pad (new age)
    91,  // Pad (choir)
    94,  // Pad (halo)
    52,  // Choir Aahs
];

/// Arpeggio instrument choices
const ARPEGGIO_INSTRUMENTS: &[u8] = &[
    46,  // Harp
    11,  // Vibraphone
    14,  // Tubular Bells
    8,   // Celesta
    25,  // Acoustic Guitar (nylon)
];

/// Bass instrument choices
const BASS_INSTRUMENTS: &[u8] = &[
    42,  // Cello
    43,  // Contrabass
    32,  // Acoustic Bass
    89,  // Pad (warm) - for drone bass
];

impl MoodGenerator for CalmPreset {
    fn generate(&self, config: &PresetConfig) -> Vec<NoteSequence> {
        let variation = PresetVariation::from_seed(config.seed);
        let mut rng = create_rng(config.seed);
        let mut sequences = Vec::new();

        let effective_tempo = variation.effective_tempo(config.tempo);
        let beats = config.duration_secs * effective_tempo as f64 / 60.0;

        // Choose instruments
        let pad_inst = variation.pick_instrument(0, PAD_INSTRUMENTS);
        let arp_inst = variation.pick_instrument(1, ARPEGGIO_INSTRUMENTS);
        let bass_inst = variation.pick_instrument(2, BASS_INSTRUMENTS);

        // Layer 1: Sustained pad chord (always)
        sequences.push(generate_pad_chord(config, &variation, beats, effective_tempo, pad_inst, &mut rng));

        // Layer 2: Gentle arpeggio (high probability)
        if variation.layer_probs[1] > 0.2 {
            sequences.push(generate_arpeggio(config, &variation, beats, effective_tempo, arp_inst, &mut rng));
        }

        // Layer 3: Optional bass drone
        if variation.layer_probs[2] > 0.5 {
            sequences.push(generate_bass_drone(config, &variation, beats, effective_tempo, bass_inst, &mut rng));
        }

        // Layer 4: Optional high shimmer
        if variation.layer_probs[3] > 0.6 {
            sequences.push(generate_high_shimmer(config, &variation, beats, effective_tempo, &mut rng));
        }

        sequences
    }

    fn name(&self) -> &'static str {
        "calm"
    }

    fn description(&self) -> &'static str {
        "Peaceful, serene mood with sustained pads and gentle arpeggios"
    }
}

/// Generate sustained pad chord with variation
fn generate_pad_chord(
    config: &PresetConfig,
    variation: &PresetVariation,
    beats: f64,
    tempo: u16,
    instrument: u8,
    rng: &mut impl Rng,
) -> NoteSequence {
    let root = config.key.root();
    let mut notes = Vec::new();

    // Chord style varies
    let style = variation.pick_style(0, 4);

    let chord_notes: Vec<(u8, u8)> = match style {
        0 => {
            // Major 7th / Minor add9
            if config.key.is_minor() {
                vec![(root - 12, 40), (root + 3, 35), (root + 7, 35), (root + 14, 30)]
            } else {
                vec![(root - 12, 40), (root + 4, 35), (root + 7, 35), (root + 11, 30)]
            }
        }
        1 => {
            // Sus2 - open sound
            vec![(root - 12, 42), (root + 2, 35), (root + 7, 35)]
        }
        2 => {
            // Add9 without 3rd - very open
            vec![(root - 12, 40), (root + 7, 35), (root + 14, 32)]
        }
        _ => {
            // Octaves only - minimal
            vec![(root - 12, 45), (root, 35), (root + 12, 30)]
        }
    };

    for (pitch, base_vel) in chord_notes {
        let vel = variation.adjust_velocity(base_vel + rng.gen_range(0..10));
        notes.push(Note::new(pitch, beats, vel, 0.0));
    }

    NoteSequence::new(notes, instrument, tempo)
}

/// Generate gentle arpeggio with variation
fn generate_arpeggio(
    config: &PresetConfig,
    variation: &PresetVariation,
    beats: f64,
    tempo: u16,
    instrument: u8,
    rng: &mut impl Rng,
) -> NoteSequence {
    let chord = config.key.chord_tones();
    let mut notes = Vec::new();

    // Arpeggio style varies
    let style = variation.pick_style(1, 4);

    let (note_duration, note_spacing): (f64, f64) = match style {
        0 => (0.75, 1.0),   // Slow, sustained
        1 => (0.5, 0.5),    // Medium, flowing
        2 => (0.4, 0.75),   // Quick plucks
        _ => (1.0, 1.5),    // Very slow, meditative
    };

    // Octave offset varies
    let octave_offset = if variation.style_choices[1] % 3 == 0 { 0i8 } else { 12 };

    let mut t = rng.gen_range(0.1..0.5); // Varied start time
    let mut chord_index = 0;
    let mut ascending = variation.style_choices[2] % 2 == 0;

    while t < beats - 0.5 {
        let base_pitch = chord[chord_index] as i8 + octave_offset;
        let pitch = base_pitch as u8;
        let velocity = variation.adjust_velocity(40 + rng.gen_range(0..20));

        notes.push(Note::new(pitch, note_duration, velocity, t));

        // Movement pattern varies
        if ascending {
            chord_index += 1;
            if chord_index >= chord.len() {
                chord_index = chord.len() - 2;
                ascending = false;
            }
        } else if chord_index == 0 {
            ascending = true;
            chord_index = 1;
        } else {
            chord_index -= 1;
        }

        t += note_spacing * variation.density_factor;
    }

    NoteSequence::new(notes, instrument, tempo)
}

/// Generate bass drone
fn generate_bass_drone(
    config: &PresetConfig,
    variation: &PresetVariation,
    beats: f64,
    tempo: u16,
    instrument: u8,
    rng: &mut impl Rng,
) -> NoteSequence {
    let root = config.key.root();
    let mut notes = Vec::new();

    // Drone style
    let style = variation.pick_style(2, 3);

    match style {
        0 => {
            // Single sustained note
            let vel = variation.adjust_velocity(35 + rng.gen_range(0..10));
            notes.push(Note::new(root - 24, beats, vel, 0.0));
        }
        1 => {
            // Root + fifth
            let vel1 = variation.adjust_velocity(35 + rng.gen_range(0..10));
            let vel2 = variation.adjust_velocity(30 + rng.gen_range(0..10));
            notes.push(Note::new(root - 24, beats, vel1, 0.0));
            notes.push(Note::new(root - 17, beats, vel2, 0.0)); // Fifth below
        }
        _ => {
            // Pulsing bass
            let mut t = 0.0;
            while t < beats {
                let vel = variation.adjust_velocity(35 + rng.gen_range(0..15));
                let dur = rng.gen_range(1.5_f64..3.0_f64).min(beats - t);
                notes.push(Note::new(root - 24, dur, vel, t));
                t += dur + rng.gen_range(0.5_f64..1.5_f64);
            }
        }
    }

    NoteSequence::new(notes, instrument, tempo)
}

/// Generate high shimmer notes
fn generate_high_shimmer(
    config: &PresetConfig,
    variation: &PresetVariation,
    beats: f64,
    tempo: u16,
    rng: &mut impl Rng,
) -> NoteSequence {
    let root = config.key.root();
    let scale = config.key.scale_intervals();
    let mut notes = Vec::new();

    // Sparse high notes
    let num_notes = (2.0 + beats / 3.0 * variation.note_count_factor) as usize;

    let mut positions: Vec<f64> = (0..num_notes)
        .map(|_| rng.gen_range(0.5_f64..beats - 0.5))
        .collect();
    positions.sort_by(|a, b| a.partial_cmp(b).unwrap());

    for pos in positions {
        let interval = scale[rng.gen_range(0..scale.len())];
        let pitch = root + 24 + interval; // Two octaves up
        let vel = variation.adjust_velocity(25 + rng.gen_range(0..15));
        let dur = rng.gen_range(1.0_f64..2.0_f64);
        notes.push(Note::new(pitch, dur, vel, pos));
    }

    // Celesta or glockenspiel
    let instrument = if rng.gen_bool(0.6) { 8 } else { 9 };
    NoteSequence::new(notes, instrument, tempo)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calm_generates_sequences() {
        let config = PresetConfig {
            tempo: 70,
            ..Default::default()
        };
        let sequences = CalmPreset.generate(&config);
        assert!(!sequences.is_empty());
    }

    #[test]
    fn test_calm_adjacent_seeds_differ() {
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

        let seq1 = CalmPreset.generate(&config1);
        let seq2 = CalmPreset.generate(&config2);

        let mut diffs = 0;
        if seq1.len() != seq2.len() {
            diffs += 5;
        }
        if !seq1.is_empty() && !seq2.is_empty() && seq1[0].instrument != seq2[0].instrument {
            diffs += 3;
        }

        assert!(diffs >= 1 || seq1.len() != seq2.len(), "Adjacent seeds should differ");
    }

    #[test]
    fn test_calm_instruments_vary() {
        let instruments: Vec<u8> = (1..=15)
            .map(|seed| {
                let config = PresetConfig {
                    seed,
                    duration_secs: 3.0,
                    ..Default::default()
                };
                let seqs = CalmPreset.generate(&config);
                seqs[0].instrument
            })
            .collect();

        let unique: std::collections::HashSet<_> = instruments.iter().collect();
        assert!(unique.len() > 1, "Instruments should vary across seeds");
    }
}
