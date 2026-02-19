//! Upbeat mood preset
//!
//! Characteristics: Major key, rhythmic, energetic, clear pulse

use super::{create_rng, MoodGenerator, PresetConfig};
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
        let mut rng = create_rng(config.seed);
        let mut sequences = Vec::new();

        // Tempo variation: ±15% for upbeat (can go faster)
        let tempo_var = 1.0 + (rng.gen_range(-10..=15) as f64 / 100.0);
        let effective_tempo = ((config.tempo as f64 * tempo_var) as u16).clamp(80, 180);

        let beats = config.duration_secs * effective_tempo as f64 / 60.0;

        // Choose instruments
        let rhythm_inst = RHYTHM_INSTRUMENTS[rng.gen_range(0..RHYTHM_INSTRUMENTS.len())];
        let bass_inst = BASS_INSTRUMENTS[rng.gen_range(0..BASS_INSTRUMENTS.len())];
        let lead_inst = LEAD_INSTRUMENTS[rng.gen_range(0..LEAD_INSTRUMENTS.len())];

        // Choose pattern
        let pattern_idx = rng.gen_range(0..RHYTHM_PATTERNS.len());

        // Layer 1: Rhythmic chord pattern (always)
        sequences.push(generate_rhythm_pattern(config, beats, effective_tempo, rhythm_inst, pattern_idx, &mut rng));

        // Layer 2: Bass line (90% chance)
        if rng.gen_bool(0.9) {
            sequences.push(generate_bass_line(config, beats, effective_tempo, bass_inst, &mut rng));
        }

        // Layer 3: Melody hint (probability + intensity)
        let melody_prob = 0.3 + (config.intensity as f64 / 150.0);
        if rng.gen_bool(melody_prob) {
            sequences.push(generate_melody_hint(config, beats, effective_tempo, lead_inst, &mut rng));
        }

        // Layer 4: Percussion accent (random chance)
        if rng.gen_bool(0.4) {
            sequences.push(generate_percussion_accent(config, beats, effective_tempo, &mut rng));
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

    // Velocity variation style
    let accent_style = rng.gen_range(0..3);

    let mut t = 0.0;
    while t < beats {
        for (i, &offset) in pattern.iter().enumerate() {
            let pos = t + offset;
            if pos >= beats {
                break;
            }

            // Velocity varies by accent style
            let velocity = match accent_style {
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

/// Generate bass line with variation
fn generate_bass_line(
    config: &PresetConfig,
    beats: f64,
    tempo: u16,
    instrument: u8,
    rng: &mut impl Rng,
) -> NoteSequence {
    let root = config.key.root();
    let fifth = root + 7;
    let third = if config.key.is_minor() { root + 3 } else { root + 4 };
    let mut notes = Vec::new();

    // Bass pattern style
    let style = rng.gen_range(0..4);

    let mut t = 0.0;

    match style {
        0 => {
            // Root-fifth alternating
            let mut is_root = true;
            while t < beats {
                let pitch = if is_root { root - 12 } else { fifth - 12 };
                let velocity = 85 + rng.gen_range(0..15);
                notes.push(Note::new(pitch, 0.4, velocity, t));
                is_root = !is_root;
                t += 0.5;
            }
        }
        1 => {
            // Root-third-fifth pattern
            let pattern = [root - 12, third - 12, fifth - 12, third - 12];
            let mut idx = 0;
            while t < beats {
                let pitch = pattern[idx % 4];
                let velocity = 80 + rng.gen_range(0..20);
                notes.push(Note::new(pitch, 0.35, velocity, t));
                idx += 1;
                t += 0.5;
            }
        }
        2 => {
            // Syncopated bass
            while t < beats {
                let pitch = if rng.gen_bool(0.7) { root - 12 } else { fifth - 12 };
                let velocity = 90 + rng.gen_range(0..10);
                let step = if rng.gen_bool(0.3) { 0.75 } else { 0.5 };
                notes.push(Note::new(pitch, 0.3, velocity, t));
                t += step;
            }
        }
        _ => {
            // Octave jumps
            let mut low = true;
            while t < beats {
                let octave = if low { -12 } else { 0 };
                let pitch = (root as i8 + octave) as u8;
                let velocity = 85 + rng.gen_range(0..15);
                notes.push(Note::new(pitch, 0.4, velocity, t));
                if rng.gen_bool(0.4) {
                    low = !low;
                }
                t += 0.5;
            }
        }
    }

    NoteSequence::new(notes, instrument, tempo)
}

/// Generate melody hint with variation
fn generate_melody_hint(
    config: &PresetConfig,
    beats: f64,
    tempo: u16,
    instrument: u8,
    rng: &mut impl Rng,
) -> NoteSequence {
    let scale = config.key.scale_intervals();
    let root = config.key.root();
    let mut notes = Vec::new();

    // Number of notes varies more
    let num_notes = rng.gen_range(2..=8);

    // Start position varies
    let start_pos = beats * rng.gen_range(0.4..0.7);

    // Melodic style
    let style = rng.gen_range(0..3);

    for i in 0..num_notes {
        let interval = scale[rng.gen_range(0..scale.len())];
        let octave: i8 = match style {
            0 => if rng.gen_bool(0.3) { 12 } else { 0 },
            1 => rng.gen_range(0..2) * 12,
            _ => if i % 2 == 0 { 0 } else { 12 },
        };
        let pitch = (root as i8 + interval as i8 + octave) as u8;

        let step = match style {
            0 => 0.25,
            1 => 0.5,
            _ => rng.gen_range(0.2..0.4),
        };
        let pos = start_pos + (i as f64 * step);
        if pos >= beats {
            break;
        }

        let velocity = 65 + rng.gen_range(0..25);
        let duration = if i == num_notes - 1 {
            rng.gen_range(0.4..0.8)
        } else {
            rng.gen_range(0.2..0.4)
        };

        notes.push(Note::new(pitch, duration, velocity, pos));
    }

    NoteSequence::new(notes, instrument, tempo)
}

/// Generate percussion accent
fn generate_percussion_accent(
    _config: &PresetConfig,
    beats: f64,
    tempo: u16,
    rng: &mut impl Rng,
) -> NoteSequence {
    let mut notes = Vec::new();

    // Accent pitch (higher = brighter)
    let pitch = 70 + rng.gen_range(0..15);

    // Pattern style
    let style = rng.gen_range(0..3);

    let mut t = 0.0;
    match style {
        0 => {
            // Backbeat (2 and 4)
            t = 1.0;
            while t < beats {
                notes.push(Note::new(pitch, 0.1, 70 + rng.gen_range(0..20), t));
                t += 2.0;
            }
        }
        1 => {
            // Every beat
            while t < beats {
                notes.push(Note::new(pitch, 0.08, 60 + rng.gen_range(0..15), t));
                t += 1.0;
            }
        }
        _ => {
            // Sparse accents
            while t < beats {
                if rng.gen_bool(0.4) {
                    notes.push(Note::new(pitch, 0.1, 75 + rng.gen_range(0..15), t));
                }
                t += 1.0;
            }
        }
    }

    // Woodblock or similar
    let instrument = if rng.gen_bool(0.5) { 115 } else { 116 };
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
