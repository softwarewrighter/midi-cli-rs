//! Ambient mood preset
//!
//! Characteristics: Textural, non-rhythmic, drones, evolving, pentatonic

use super::{create_rng, MoodGenerator, PresetConfig, PresetVariation};
use crate::midi::{Note, NoteSequence};
use rand::Rng;

/// Ambient mood generator
pub struct AmbientPreset;

/// Drone instrument choices
const DRONE_INSTRUMENTS: &[u8] = &[
    48,  // String Ensemble 1
    49,  // String Ensemble 2
    89,  // Pad (warm)
    91,  // Pad (bowed)
    92,  // Pad (metallic)
    94,  // Pad (halo)
];

/// Bell/tone instrument choices
const BELL_INSTRUMENTS: &[u8] = &[
    11,  // Vibraphone
    14,  // Tubular Bells
    8,   // Celesta
    98,  // Crystal
    10,  // Glockenspiel
];

impl MoodGenerator for AmbientPreset {
    fn generate(&self, config: &PresetConfig) -> Vec<NoteSequence> {
        let variation = PresetVariation::from_seed(config.seed);
        let mut rng = create_rng(config.seed);
        let mut sequences = Vec::new();

        let effective_tempo = variation.effective_tempo(config.tempo);
        let beats = config.duration_secs * effective_tempo as f64 / 60.0;

        // Choose instruments
        let drone_inst = variation.pick_instrument(0, DRONE_INSTRUMENTS);
        let bell_inst = variation.pick_instrument(1, BELL_INSTRUMENTS);

        // Drone interval style varies
        let second_interval = match variation.pick_style(0, 4) {
            0 => 7,  // Perfect fifth
            1 => 5,  // Perfect fourth
            2 => 12, // Octave
            _ => 4,  // Major third
        };

        // Layer 1: Primary drone (always)
        sequences.push(generate_drone_layer(config, &variation, beats, effective_tempo, drone_inst, 0, &mut rng));

        // Layer 2: Second drone (high probability)
        if variation.layer_probs[1] > 0.25 {
            sequences.push(generate_drone_layer(config, &variation, beats, effective_tempo, drone_inst, second_interval, &mut rng));
        }

        // Layer 3: Sporadic tones (high probability)
        if variation.layer_probs[2] > 0.2 {
            sequences.push(generate_sporadic_tones(config, &variation, beats, effective_tempo, bell_inst, &mut rng));
        }

        // Layer 4: Sub-bass rumble
        if variation.layer_probs[3] > 0.5 {
            sequences.push(generate_sub_rumble(config, &variation, beats, effective_tempo, &mut rng));
        }

        sequences
    }

    fn name(&self) -> &'static str {
        "ambient"
    }

    fn description(&self) -> &'static str {
        "Atmospheric, textural mood with drones and sparse pentatonic tones"
    }
}

/// Generate drone layer with variation
fn generate_drone_layer(
    config: &PresetConfig,
    variation: &PresetVariation,
    beats: f64,
    tempo: u16,
    instrument: u8,
    interval: u8,
    rng: &mut impl Rng,
) -> NoteSequence {
    let root = config.key.root();
    let pitch = root - 12 + interval;
    let mut notes = Vec::new();

    // Drone style varies
    let style = variation.pick_style(1, 3);

    match style {
        0 => {
            // Single sustained
            let vel = variation.adjust_velocity(30 + rng.gen_range(0..15));
            notes.push(Note::new(pitch, beats, vel, 0.0));
        }
        1 => {
            // Swelling drone
            let vel1 = variation.adjust_velocity(20 + rng.gen_range(0..10));
            let vel2 = variation.adjust_velocity(40 + rng.gen_range(0..10));
            notes.push(Note::new(pitch, beats / 2.0, vel1, 0.0));
            notes.push(Note::new(pitch, beats / 2.0, vel2, beats / 2.0));
        }
        _ => {
            // Pulsing drone
            let mut t = 0.0;
            while t < beats {
                let vel = variation.adjust_velocity(25 + rng.gen_range(0..20));
                let dur = rng.gen_range(2.0_f64..5.0_f64).min(beats - t);
                notes.push(Note::new(pitch, dur, vel, t));
                t += dur;
            }
        }
    }

    NoteSequence::new(notes, instrument, tempo)
}

/// Generate sporadic pentatonic tones with melodic contour variation
fn generate_sporadic_tones(
    config: &PresetConfig,
    variation: &PresetVariation,
    beats: f64,
    tempo: u16,
    instrument: u8,
    rng: &mut impl Rng,
) -> NoteSequence {
    let root = config.key.root();
    let mut notes = Vec::new();

    // Pentatonic scale
    let pentatonic: &[u8] = if config.key.is_minor() {
        &[0, 3, 5, 7, 10]
    } else {
        &[0, 2, 4, 7, 9]
    };

    // Number of tones varies with variation
    let base_tones = 2 + (config.intensity as usize / 30);
    let num_tones = (base_tones as f64 * variation.note_count_factor) as usize;

    let mut positions: Vec<f64> = (0..num_tones)
        .map(|_| rng.gen_range(0.5_f64..beats - 1.0))
        .collect();
    positions.sort_by(|a, b| a.partial_cmp(b).unwrap());

    // Get contour for melodic movement
    let contour = variation.get_contour(num_tones);
    let mut scale_idx = (variation.scale_offset as usize) % pentatonic.len();

    // Base octave varies by seed
    let base_octave: u8 = match variation.style_choices[2] % 3 {
        0 => 0,
        1 => 12,
        _ => 24,
    };
    let mut current_octave = base_octave;

    // Octave range varies
    let max_octave = match variation.pick_style(2, 3) {
        0 => 1,
        1 => 2,
        _ => 3,
    } * 12;

    for (i, pos) in positions.iter().enumerate() {
        // Skip for ambient rests (sparse texture)
        if variation.should_rest(rng) {
            continue;
        }

        let interval = pentatonic[scale_idx % pentatonic.len()];
        let pitch = root + interval + current_octave;

        let velocity = variation.adjust_velocity(20 + rng.gen_range(0..20));
        let duration = rng.gen_range(1.5_f64..4.0_f64);

        notes.push(Note::new(pitch, duration, velocity, *pos));

        // Move through scale based on contour
        let direction = contour[i % contour.len()];
        let step = variation.get_interval(rng) as usize;
        match direction {
            1 => {
                scale_idx = (scale_idx + step) % pentatonic.len();
                // Sometimes move up an octave
                if rng.gen_bool(0.25) && current_octave < max_octave as u8 {
                    current_octave += 12;
                }
            }
            -1 => {
                scale_idx = if scale_idx >= step { scale_idx - step } else { pentatonic.len() - 1 };
                // Sometimes move down an octave
                if rng.gen_bool(0.25) && current_octave > 0 {
                    current_octave -= 12;
                }
            }
            _ => {} // Stay
        }
    }

    NoteSequence::new(notes, instrument, tempo)
}

/// Generate sub-bass rumble
fn generate_sub_rumble(
    config: &PresetConfig,
    variation: &PresetVariation,
    beats: f64,
    tempo: u16,
    rng: &mut impl Rng,
) -> NoteSequence {
    let root = config.key.root();
    let pitch = root.saturating_sub(36).max(24);
    let mut notes = Vec::new();

    let mut t = 0.0;
    while t < beats {
        let vel = variation.adjust_velocity(20 + rng.gen_range(0..15));
        let dur = rng.gen_range(1.5_f64..4.0_f64).min(beats - t);
        notes.push(Note::new(pitch, dur, vel, t));
        t += dur + rng.gen_range(0.0_f64..2.0_f64);
    }

    // Synth bass or contrabass
    let instrument = if rng.gen_bool(0.5) { 38 } else { 43 };
    NoteSequence::new(notes, instrument, tempo)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ambient_generates_sequences() {
        let config = PresetConfig::default();
        let sequences = AmbientPreset.generate(&config);
        assert!(!sequences.is_empty());
    }

    #[test]
    fn test_ambient_seeds_vary_across_range() {
        // Test that seeds produce variation across a range (not just adjacent)
        let configs: Vec<_> = (1..=10)
            .map(|seed| PresetConfig { seed, duration_secs: 5.0, ..Default::default() })
            .collect();

        let results: Vec<_> = configs.iter().map(|c| AmbientPreset.generate(c)).collect();

        // Check for variation in layer counts
        let layer_counts: std::collections::HashSet<_> = results.iter().map(|s| s.len()).collect();

        // Check for variation in instruments
        let instruments: std::collections::HashSet<_> = results.iter()
            .filter(|s| !s.is_empty())
            .map(|s| s[0].instrument)
            .collect();

        // At least one type of variation should occur across 10 seeds
        assert!(
            layer_counts.len() > 1 || instruments.len() > 1,
            "Seeds should produce variation in layers ({:?}) or instruments ({:?})",
            layer_counts, instruments
        );
    }

    #[test]
    fn test_ambient_instruments_vary() {
        let instruments: Vec<u8> = (1..=15)
            .map(|seed| {
                let config = PresetConfig { seed, duration_secs: 3.0, ..Default::default() };
                let seqs = AmbientPreset.generate(&config);
                seqs[0].instrument
            })
            .collect();

        let unique: std::collections::HashSet<_> = instruments.iter().collect();
        assert!(unique.len() > 1, "Instruments should vary across seeds");
    }
}
