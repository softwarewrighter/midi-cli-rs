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

/// Generate gentle arpeggio/melody with rich variation
fn generate_arpeggio(
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

    // Arpeggio style varies timing
    let style = variation.pick_style(1, 4);
    let (base_duration, base_spacing): (f64, f64) = match style {
        0 => (0.75, 1.0),   // Slow, sustained
        1 => (0.5, 0.5),    // Medium, flowing
        2 => (0.4, 0.75),   // Quick plucks
        _ => (1.0, 1.5),    // Very slow, meditative
    };

    // Octave varies based on seed
    let base_octave: i8 = match variation.style_choices[1] % 4 {
        0 => 0,
        1 => 12,
        2 => -12,
        _ => if rng.gen_bool(0.5) { 0 } else { 12 },
    };

    // Get melodic contour pattern from seed
    let phrase_len = variation.phrase_length as usize;
    let contour = variation.get_contour(phrase_len);

    // Start at a seed-determined scale position
    let mut scale_index = (variation.scale_offset as usize) % scale.len();
    let mut t = rng.gen_range(0.1..0.5);
    let mut phrase_position = 0;
    let mut phrase_count = 0;

    // Track the phrase for potential repetition/transformation
    let mut current_phrase: Vec<(u8, f64, u8, f64)> = Vec::new(); // (pitch, dur, vel, time)

    while t < beats - 0.5 {
        // Check for rest
        if variation.should_rest(rng) && phrase_position > 0 {
            t += base_spacing * variation.density_factor;
            continue;
        }

        // Calculate pitch from scale
        let interval = scale[scale_index];
        let octave_adjust = base_octave + (scale_index as i8 / scale.len() as i8) * 12;
        let pitch = ((root as i8 + interval as i8 + octave_adjust) as u8).clamp(36, 96);

        // Vary duration slightly
        let duration = base_duration * rng.gen_range(0.8..1.2);
        let velocity = variation.adjust_velocity(40 + rng.gen_range(0..20));

        notes.push(Note::new(pitch, duration, velocity, t));
        current_phrase.push((pitch, duration, velocity, t));

        // Apply contour pattern to determine next note
        let direction = contour[phrase_position % contour.len()];
        let step = variation.get_interval(rng) as i8;

        match direction {
            1 => {
                // Move up
                scale_index = (scale_index + step as usize) % (scale.len() * 2);
            }
            -1 => {
                // Move down
                if scale_index >= step as usize {
                    scale_index -= step as usize;
                } else {
                    scale_index = 0;
                }
            }
            _ => {
                // Stay on same note or small variation
                if rng.gen_bool(0.3) {
                    scale_index = (scale_index + 1) % scale.len();
                }
            }
        }

        // Keep scale_index in valid range
        scale_index = scale_index % (scale.len() * 2);
        if scale_index >= scale.len() {
            scale_index = scale_index % scale.len();
        }

        phrase_position += 1;
        t += base_spacing * variation.density_factor;

        // End of phrase - possibly transform and repeat
        if phrase_position >= phrase_len && !current_phrase.is_empty() {
            phrase_count += 1;

            // Apply phrase transformation based on seed
            if phrase_count <= 2 && variation.phrase_transform < 4 && t < beats - 2.0 {
                match variation.phrase_transform {
                    0 => {
                        // Repeat phrase exactly
                        for (p, d, v, _) in &current_phrase {
                            if t >= beats - 0.5 { break; }
                            notes.push(Note::new(*p, *d, *v, t));
                            t += base_spacing * variation.density_factor;
                        }
                    }
                    1 => {
                        // Invert phrase (mirror pitches)
                        let mid_pitch = current_phrase.iter().map(|(p, _, _, _)| *p as i16).sum::<i16>()
                            / current_phrase.len() as i16;
                        for (p, d, v, _) in &current_phrase {
                            if t >= beats - 0.5 { break; }
                            let inverted = (2 * mid_pitch - *p as i16).clamp(36, 96) as u8;
                            notes.push(Note::new(inverted, *d, *v, t));
                            t += base_spacing * variation.density_factor;
                        }
                    }
                    2 => {
                        // Play faster (double speed)
                        for (p, d, v, _) in &current_phrase {
                            if t >= beats - 0.5 { break; }
                            notes.push(Note::new(*p, d * 0.5, *v, t));
                            t += base_spacing * variation.density_factor * 0.5;
                        }
                    }
                    3 => {
                        // Play slower (half speed)
                        for (p, d, v, _) in current_phrase.iter().take(phrase_len / 2) {
                            if t >= beats - 0.5 { break; }
                            notes.push(Note::new(*p, d * 2.0, *v, t));
                            t += base_spacing * variation.density_factor * 2.0;
                        }
                    }
                    _ => {}
                }
            }

            // Reset for next phrase
            current_phrase.clear();
            phrase_position = 0;

            // Add a rest between phrases sometimes
            if rng.gen_bool(0.4) {
                t += base_spacing;
            }
        }
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

/// Generate high shimmer notes with melodic variation
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

    // Sparse high notes - count varies by seed
    let num_notes = (2.0 + beats / 3.0 * variation.note_count_factor) as usize;

    let mut positions: Vec<f64> = (0..num_notes)
        .map(|_| rng.gen_range(0.5_f64..beats - 0.5))
        .collect();
    positions.sort_by(|a, b| a.partial_cmp(b).unwrap());

    // Use contour for melodic movement in shimmer
    let contour = variation.get_contour(num_notes);
    let mut scale_idx = (variation.scale_offset as usize) % scale.len();

    // Octave varies by seed
    let octave_up: u8 = match variation.style_choices[3] % 3 {
        0 => 24, // Two octaves
        1 => 12, // One octave
        _ => 36, // Three octaves (very high)
    };

    for (i, pos) in positions.iter().enumerate() {
        // Skip some notes for rests
        if variation.should_rest(rng) {
            continue;
        }

        let interval = scale[scale_idx % scale.len()];
        let pitch = (root + octave_up + interval).min(108); // Cap at high C
        let vel = variation.adjust_velocity(25 + rng.gen_range(0..15));
        let dur = rng.gen_range(1.0_f64..2.0_f64);
        notes.push(Note::new(pitch, dur, vel, *pos));

        // Move scale position based on contour
        let direction = contour[i % contour.len()];
        match direction {
            1 => scale_idx = (scale_idx + 1) % scale.len(),
            -1 => scale_idx = if scale_idx > 0 { scale_idx - 1 } else { scale.len() - 1 },
            _ => {}
        }
    }

    // Instrument varies by seed
    let instrument = match variation.style_choices[4] % 4 {
        0 => 8,   // Celesta
        1 => 9,   // Glockenspiel
        2 => 11,  // Vibraphone
        _ => 14,  // Tubular Bells
    };
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
