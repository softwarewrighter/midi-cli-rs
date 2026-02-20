//! Upbeat mood preset
//!
//! Characteristics: Major key, rhythmic, energetic, clear pulse

use super::{create_rng, MoodGenerator, PresetConfig, PresetVariation};
use crate::midi::{Note, NoteSequence};
use rand::Rng;

/// Upbeat mood generator
pub struct UpbeatPreset;

/// Rhythm instrument choices
const RHYTHM_INSTRUMENTS: &[u8] = &[
    0,  // Acoustic Grand
    1,  // Bright Acoustic
    4,  // Electric Piano 1
    5,  // Electric Piano 2
    7,  // Clavinet
    25, // Acoustic Guitar (steel)
];

/// Bass instrument choices
const BASS_INSTRUMENTS: &[u8] = &[
    33, // Electric Bass (finger)
    34, // Electric Bass (pick)
    36, // Slap Bass 1
    37, // Slap Bass 2
    38, // Synth Bass 1
];

/// Lead instrument choices
const LEAD_INSTRUMENTS: &[u8] = &[
    80, // Square Lead
    81, // Sawtooth Lead
    73, // Flute
    65, // Alto Sax
    56, // Trumpet
];

/// Rhythm patterns (offsets within a 4-beat bar)
const RHYTHM_PATTERNS: &[&[f64]] = &[
    &[0.0, 0.5, 1.0, 1.5, 2.5, 3.0, 3.5],           // Syncopated
    &[0.0, 1.0, 2.0, 3.0],                           // Quarter notes
    &[0.0, 0.5, 1.5, 2.0, 2.5, 3.5],                 // Funk pattern
    &[0.0, 0.75, 1.5, 2.25, 3.0, 3.75],              // Dotted eighths
    &[0.0, 0.5, 1.0, 2.0, 2.5, 3.0, 3.5],            // Pop pattern
];

impl MoodGenerator for UpbeatPreset {
    fn generate(&self, config: &PresetConfig) -> Vec<NoteSequence> {
        let variation = PresetVariation::from_seed(config.seed);
        let mut rng = create_rng(config.seed);
        let mut sequences = Vec::new();

        let effective_tempo = variation.effective_tempo(config.tempo);
        let beats = config.duration_secs * effective_tempo as f64 / 60.0;

        // Choose instruments using variation system
        let rhythm_inst = variation.pick_instrument(0, RHYTHM_INSTRUMENTS);
        let bass_inst = variation.pick_instrument(1, BASS_INSTRUMENTS);
        let lead_inst = variation.pick_instrument(2, LEAD_INSTRUMENTS);

        // Choose pattern based on seed
        let pattern_idx = variation.pick_style(0, RHYTHM_PATTERNS.len());

        // Layer 1: Rhythmic chord pattern (always)
        sequences.push(generate_rhythm_pattern(config, &variation, beats, effective_tempo, rhythm_inst, pattern_idx, &mut rng));

        // Layer 2: Bass line (high probability)
        if variation.layer_probs[1] > 0.1 {
            sequences.push(generate_bass_line(config, &variation, beats, effective_tempo, bass_inst, &mut rng));
        }

        // Layer 3: Melody hint (probability + intensity)
        let melody_threshold = 0.7 - (config.intensity as f64 / 150.0);
        if variation.layer_probs[2] > melody_threshold {
            sequences.push(generate_melody_hint(config, &variation, beats, effective_tempo, lead_inst, &mut rng));
        }

        // Layer 4: Percussion accent
        if variation.layer_probs[3] > 0.6 {
            sequences.push(generate_percussion_accent(config, &variation, beats, effective_tempo, &mut rng));
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

/// Generate rhythmic chord pattern with variation
fn generate_rhythm_pattern(
    config: &PresetConfig,
    variation: &PresetVariation,
    beats: f64,
    tempo: u16,
    instrument: u8,
    pattern_idx: usize,
    rng: &mut impl Rng,
) -> NoteSequence {
    let chord = config.key.chord_tones();
    let mut notes = Vec::new();

    let pattern = RHYTHM_PATTERNS[pattern_idx];
    let pattern_len = 4.0;

    // Velocity variation style from seed
    let accent_style = variation.pick_style(1, 3);

    let mut t = 0.0;
    while t < beats {
        for (i, &offset) in pattern.iter().enumerate() {
            let pos = t + offset;
            if pos >= beats {
                break;
            }

            // Velocity varies by accent style
            let base_velocity = match accent_style {
                0 => {
                    // First beat accent
                    if offset == 0.0 { 85 + rng.gen_range(0..15) } else { 60 + rng.gen_range(0..20) }
                }
                1 => {
                    // Every other accent
                    if i % 2 == 0 { 75 + rng.gen_range(0..15) } else { 55 + rng.gen_range(0..15) }
                }
                _ => {
                    // Random accents
                    65 + rng.gen_range(0..25)
                }
            };
            let velocity = variation.adjust_velocity(base_velocity);

            // Duration varies
            let duration = if rng.gen_bool(0.3) {
                rng.gen_range(0.15..0.35)
            } else {
                rng.gen_range(0.2..0.5)
            };

            // Sometimes drop a note from chord for variation
            for (j, &pitch) in chord.iter().enumerate() {
                if rng.gen_bool(0.95) || j == 0 {
                    // Always include root
                    notes.push(Note::new(pitch, duration, velocity, pos));
                }
            }
        }
        t += pattern_len;
    }

    NoteSequence::new(notes, instrument, tempo)
}

/// Generate bass line with melodic variation
fn generate_bass_line(
    config: &PresetConfig,
    variation: &PresetVariation,
    beats: f64,
    tempo: u16,
    instrument: u8,
    rng: &mut impl Rng,
) -> NoteSequence {
    let root = config.key.root();
    let fifth = root + 7;
    let third = if config.key.is_minor() { root + 3 } else { root + 4 };
    let mut notes = Vec::new();

    // Bass pattern style from seed
    let style = variation.pick_style(2, 4);

    // Get contour for melodic bass movement
    let contour = variation.get_contour(variation.phrase_length as usize);
    let bass_notes = [root - 12, third - 12, fifth - 12, root - 24];
    let mut bass_idx = (variation.scale_offset as usize) % bass_notes.len();
    let mut phrase_pos = 0;

    let mut t = 0.0;

    match style {
        0 => {
            // Contour-driven bass line
            while t < beats {
                let pitch = bass_notes[bass_idx % bass_notes.len()];
                let velocity = variation.adjust_velocity(85 + rng.gen_range(0..15));

                if !variation.should_rest(rng) {
                    notes.push(Note::new(pitch, 0.4, velocity, t));
                }

                // Move based on contour
                let direction = contour[phrase_pos % contour.len()];
                match direction {
                    1 => bass_idx = (bass_idx + 1) % bass_notes.len(),
                    -1 => bass_idx = if bass_idx > 0 { bass_idx - 1 } else { bass_notes.len() - 1 },
                    _ => {}
                }
                phrase_pos += 1;
                t += 0.5;
            }
        }
        1 => {
            // Root-third-fifth with contour variation
            while t < beats {
                let pitch = bass_notes[bass_idx % 3]; // First 3 notes only
                let velocity = variation.adjust_velocity(80 + rng.gen_range(0..20));
                notes.push(Note::new(pitch, 0.35, velocity, t));

                let direction = contour[phrase_pos % contour.len()];
                bass_idx = match direction {
                    1 => (bass_idx + 1) % 3,
                    -1 => if bass_idx > 0 { bass_idx - 1 } else { 2 },
                    _ => bass_idx,
                };
                phrase_pos += 1;
                t += 0.5;
            }
        }
        2 => {
            // Syncopated bass with contour
            while t < beats {
                let pitch = bass_notes[bass_idx % bass_notes.len()];
                let velocity = variation.adjust_velocity(90 + rng.gen_range(0..10));
                let step = if rng.gen_bool(0.3) { 0.75 } else { 0.5 };

                if !variation.should_rest(rng) {
                    notes.push(Note::new(pitch, 0.3, velocity, t));
                }

                let direction = contour[phrase_pos % contour.len()];
                bass_idx = match direction {
                    1 => (bass_idx + 1) % bass_notes.len(),
                    -1 => if bass_idx > 0 { bass_idx - 1 } else { bass_notes.len() - 1 },
                    _ => bass_idx,
                };
                phrase_pos += 1;
                t += step;
            }
        }
        _ => {
            // Octave jumps with contour influence
            let mut low = true;
            while t < beats {
                let octave = if low { -12 } else { 0 };
                let pitch = (root as i8 + octave) as u8;
                let velocity = variation.adjust_velocity(85 + rng.gen_range(0..15));
                notes.push(Note::new(pitch, 0.4, velocity, t));

                // Contour influences octave switching
                let direction = contour[phrase_pos % contour.len()];
                low = match direction {
                    1 => false,  // Up = high octave
                    -1 => true,  // Down = low octave
                    _ => low,    // Stay
                };
                phrase_pos += 1;
                t += 0.5;
            }
        }
    }

    NoteSequence::new(notes, instrument, tempo)
}

/// Generate melody hint with contour-based variation
fn generate_melody_hint(
    config: &PresetConfig,
    variation: &PresetVariation,
    beats: f64,
    tempo: u16,
    instrument: u8,
    rng: &mut impl Rng,
) -> NoteSequence {
    let scale = config.key.scale_intervals();
    let root = config.key.root();
    let mut notes = Vec::new();

    // Number of notes varies with seed
    let num_notes = (variation.phrase_length as usize).max(3).min(8);

    // Start position varies
    let start_pos = beats * (0.3 + variation.density_factor * 0.3);

    // Melodic style from seed
    let style = variation.pick_style(3, 3);

    // Get contour for melodic direction
    let contour = variation.get_contour(num_notes);
    let mut scale_idx = (variation.scale_offset as usize) % scale.len();

    // Base octave from seed
    let base_octave: i8 = match variation.style_choices[3] % 3 {
        0 => 0,
        1 => 12,
        _ => -12,
    };

    for i in 0..num_notes {
        // Skip for rests
        if variation.should_rest(rng) && i > 0 {
            continue;
        }

        let interval = scale[scale_idx % scale.len()];
        let octave: i8 = base_octave + match style {
            0 => if rng.gen_bool(0.3) { 12 } else { 0 },
            1 => rng.gen_range(0..2) * 12,
            _ => if i % 2 == 0 { 0 } else { 12 },
        };
        let pitch = ((root as i8 + interval as i8 + octave) as u8).clamp(48, 96);

        let step = match style {
            0 => 0.25,
            1 => 0.5,
            _ => rng.gen_range(0.2..0.4),
        };
        let pos = start_pos + (i as f64 * step);
        if pos >= beats {
            break;
        }

        let velocity = variation.adjust_velocity(65 + rng.gen_range(0..25));
        let duration = if i == num_notes - 1 {
            rng.gen_range(0.4..0.8)
        } else {
            rng.gen_range(0.2..0.4)
        };

        notes.push(Note::new(pitch, duration, velocity, pos));

        // Move through scale based on contour
        let direction = contour[i % contour.len()];
        let step_size = variation.get_interval(rng) as usize;
        match direction {
            1 => scale_idx = (scale_idx + step_size) % scale.len(),
            -1 => scale_idx = if scale_idx >= step_size { scale_idx - step_size } else { scale.len() - 1 },
            _ => {} // Hold
        }
    }

    NoteSequence::new(notes, instrument, tempo)
}

/// Generate percussion accent with variation
fn generate_percussion_accent(
    _config: &PresetConfig,
    variation: &PresetVariation,
    beats: f64,
    tempo: u16,
    rng: &mut impl Rng,
) -> NoteSequence {
    let mut notes = Vec::new();

    // Accent pitch varies by seed
    let pitch = 70 + (variation.style_choices[4] % 15);

    // Pattern style from seed
    let style = variation.pick_style(4, 3);

    let mut t = 0.0;
    match style {
        0 => {
            // Backbeat (2 and 4)
            t = 1.0;
            while t < beats {
                let vel = variation.adjust_velocity(70 + rng.gen_range(0..20));
                notes.push(Note::new(pitch, 0.1, vel, t));
                t += 2.0;
            }
        }
        1 => {
            // Every beat
            while t < beats {
                let vel = variation.adjust_velocity(60 + rng.gen_range(0..15));
                notes.push(Note::new(pitch, 0.08, vel, t));
                t += 1.0;
            }
        }
        _ => {
            // Sparse accents - use rest probability
            while t < beats {
                if !variation.should_rest(rng) {
                    let vel = variation.adjust_velocity(75 + rng.gen_range(0..15));
                    notes.push(Note::new(pitch, 0.1, vel, t));
                }
                t += 1.0;
            }
        }
    }

    // Woodblock or similar - based on seed
    let perc_instruments = &[115u8, 116, 117, 76]; // Woodblock, taiko, melodic tom, pan flute
    let instrument = variation.pick_instrument(4, perc_instruments);
    NoteSequence::new(notes, instrument, tempo)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_upbeat_generates_sequences() {
        let config = PresetConfig {
            key: super::super::Key::C,
            tempo: 140,
            ..Default::default()
        };
        let sequences = UpbeatPreset.generate(&config);
        assert!(!sequences.is_empty());
    }

    #[test]
    fn test_upbeat_adjacent_seeds_differ() {
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

        let seq1 = UpbeatPreset.generate(&config1);
        let seq2 = UpbeatPreset.generate(&config2);

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
    fn test_upbeat_instruments_vary() {
        let instruments: Vec<u8> = (1..=15)
            .map(|seed| {
                let config = PresetConfig {
                    seed,
                    duration_secs: 3.0,
                    ..Default::default()
                };
                let seqs = UpbeatPreset.generate(&config);
                seqs[0].instrument
            })
            .collect();

        let unique: std::collections::HashSet<_> = instruments.iter().collect();
        assert!(unique.len() > 1, "Instruments should vary across seeds");
    }
}
