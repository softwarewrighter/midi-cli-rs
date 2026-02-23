//! Chiptune preset - 8-bit video game style music
//!
//! Emulates classic 8-bit game console sound with dramatic seed-based variation
//! affecting tempo, key, melody style, bass patterns, percussion, and intensity.

use crate::midi::{Note, NoteSequence};
use crate::preset::{MoodGenerator, PresetConfig, PresetVariation, create_rng};
use rand::Rng;

/// GM program numbers for chip-like sounds
const SQUARE_LEAD: u8 = 80;      // Lead 1 (square)
const SAWTOOTH_LEAD: u8 = 81;    // Lead 2 (sawtooth)
const SYNTH_BASS_1: u8 = 38;     // Synth Bass 1
const SYNTH_BASS_2: u8 = 39;     // Synth Bass 2
const CALLIOPE: u8 = 82;         // Lead 3 (calliope) - another chip-like sound

/// Chiptune mood generator with dramatic seed variation
pub struct ChiptunePreset;

impl MoodGenerator for ChiptunePreset {
    fn generate(&self, config: &PresetConfig) -> Vec<NoteSequence> {
        let mut rng = create_rng(config.seed);
        let variation = PresetVariation::from_seed(config.seed);

        // TEMPO varies with seed (±15% of user-specified tempo)
        // tempo_factor is 0.85-1.15, so normalize to -0.15 to +0.15
        let tempo_adjustment = variation.tempo_factor - 1.0; // -0.15 to +0.15
        let effective_tempo = ((config.tempo as f64 * (1.0 + tempo_adjustment)) as u16).clamp(20, 300);

        // KEY/ROOT varies with seed - transpose by semitones
        let transpose: i8 = match variation.scale_offset % 7 {
            0 => 0,    // Original key
            1 => 5,    // Up a fourth
            2 => 7,    // Up a fifth
            3 => -5,   // Down a fourth
            4 => 2,    // Up a whole step
            5 => -2,   // Down a whole step
            _ => 3,    // Up a minor third
        };
        let root = ((config.key.root() as i16 + transpose as i16).clamp(36, 84)) as u8;

        // SCALE selection varies dramatically with seed
        let is_minor = config.key.is_minor();
        let scale: Vec<i8> = match variation.contour_pattern % 8 {
            0 => if is_minor { vec![0, 3, 5, 7, 10] } else { vec![0, 2, 4, 7, 9] },  // Pentatonic
            1 => if is_minor { vec![0, 2, 3, 5, 7, 8, 10] } else { vec![0, 2, 4, 5, 7, 9, 11] },  // Full scale
            2 => vec![0, 3, 6, 9],  // Diminished (dark)
            3 => vec![0, 4, 7],     // Just triad
            4 => vec![0, 2, 4, 6, 8, 10],  // Whole tone
            5 => vec![0, 1, 4, 5, 7, 8, 11],  // Harmonic minor
            6 => vec![0, 2, 3, 6, 7, 9, 10],  // Dorian
            _ => vec![0, 2, 4, 5, 7, 9, 10],  // Mixolydian
        };

        let beats = config.duration_secs * effective_tempo as f64 / 60.0;

        // INTENSITY varies with seed (base intensity ± 20)
        let intensity_offset: i16 = ((variation.velocity_offset as i16) * 2).clamp(-20, 20);
        let effective_intensity = ((config.intensity as i16 + intensity_offset).clamp(20, 100)) as u8;
        let base_velocity = 70 + (effective_intensity as i32 / 4) as u8;

        let mut sequences = Vec::new();

        // LEAD INSTRUMENT varies with seed
        let lead_instrument = match variation.instrument_indices[0] % 3 {
            0 => SQUARE_LEAD,
            1 => SAWTOOTH_LEAD,
            _ => CALLIOPE,
        };

        // Layer 1: Lead melody (always present)
        sequences.push(generate_lead_melody(
            root, &scale, beats, effective_tempo, base_velocity, lead_instrument, &variation, &mut rng
        ));

        // Layer 2: Bass line (varies with seed - sometimes absent)
        let include_bass = variation.layer_probs[1] > 0.3;
        if include_bass {
            sequences.push(generate_chip_bass(
                root, beats, effective_tempo, base_velocity.saturating_sub(10), &variation, &mut rng
            ));
        }

        // Layer 3: Counter-melody (seed-based probability)
        let counter_threshold = 30 + (variation.layer_probs[2] * 40.0) as u8;
        if effective_intensity > counter_threshold {
            sequences.push(generate_counter_melody(
                root, &scale, beats, effective_tempo, base_velocity.saturating_sub(20), &variation, &mut rng
            ));
        }

        // Layer 4: Percussion (seed-based probability - varies dramatically)
        let drum_threshold = 40 + (variation.layer_probs[3] * 50.0) as u8;
        if effective_intensity > drum_threshold && variation.density_factor > 0.7 {
            sequences.push(generate_chip_drums(
                beats, effective_tempo, base_velocity.saturating_sub(15), &variation, &mut rng
            ));
        }

        sequences
    }

    fn name(&self) -> &'static str {
        "chiptune"
    }

    fn description(&self) -> &'static str {
        "8-bit video game style with dramatic seed-based variation"
    }
}

/// Generate lead melody with many seed-varied styles
#[allow(clippy::too_many_arguments)]
fn generate_lead_melody(
    root: u8,
    scale: &[i8],
    beats: f64,
    tempo: u16,
    velocity: u8,
    instrument: u8,
    variation: &PresetVariation,
    rng: &mut impl Rng,
) -> NoteSequence {
    let mut notes = Vec::new();
    let beat_duration = 60.0 / tempo as f64;

    // RHYTHM STYLE varies dramatically
    let rhythm_style = variation.style_choices[0] % 7;
    let (note_duration, step) = match rhythm_style {
        0 => (beat_duration * 0.15, beat_duration * 0.25),    // Fast 16ths
        1 => (beat_duration * 0.4, beat_duration * 0.5),      // 8th notes
        2 => (beat_duration * 0.2, beat_duration * 0.333),    // Triplets
        3 => (beat_duration * 0.12, beat_duration * 0.125),   // 32nd notes
        4 => (beat_duration * 0.6, beat_duration * 0.75),     // Dotted 8ths
        5 => (beat_duration * 0.8, beat_duration),            // Quarter notes
        _ => (beat_duration * 0.25, beat_duration * 0.5),     // Swing-ish
    };

    let mut time = 0.0;
    let end_time = beats * beat_duration;

    // REGISTER varies dramatically
    let octave_shift: i8 = match variation.contour_pattern % 6 {
        0 => 24,   // Two octaves up (bright)
        1 => 12,   // One octave up
        2 => 0,    // Root register
        3 => 18,   // Octave + fifth
        4 => 6,    // Half octave
        _ => -12,  // Octave down (unusual, darker)
    };
    let base_pitch = (root as i16 + octave_shift as i16).clamp(36, 84) as u8;

    // MELODIC PATTERN TYPE varies
    let pattern_type = variation.phrase_transform % 8;
    let mut scale_pos: i32 = (variation.scale_offset % scale.len() as u8) as i32;
    let mut phrase_counter = 0;
    let phrase_length = variation.phrase_length.clamp(2, 8) as i32;

    // EMBELLISHMENT style varies
    let embellish_style = variation.style_choices[5] % 4;
    let rest_freq = variation.rest_probability;

    while time < end_time {
        // Variable rest frequency
        if rng.gen_bool(rest_freq) {
            time += step;
            continue;
        }

        // Generate note based on pattern type
        let interval = match pattern_type {
            0 => {  // Ascending arpeggio
                scale_pos = (scale_pos + 1) % scale.len() as i32;
                scale[scale_pos as usize]
            }
            1 => {  // Descending arpeggio
                scale_pos = (scale_pos + scale.len() as i32 - 1) % scale.len() as i32;
                scale[scale_pos as usize]
            }
            2 => {  // Random jumps
                scale_pos = rng.gen_range(0..scale.len() as i32);
                scale[scale_pos as usize]
            }
            3 => {  // Pedal tone alternating
                if phrase_counter % 2 == 0 { 0 } else { scale[scale.len() / 2] + 7 }
            }
            4 => {  // Zigzag
                let dir = if (phrase_counter / 2) % 2 == 0 { 1 } else { -1 };
                scale_pos = (scale_pos + dir).rem_euclid(scale.len() as i32);
                scale[scale_pos as usize]
            }
            5 => {  // Repeated notes with occasional leap
                if phrase_counter % phrase_length == 0 {
                    scale_pos = rng.gen_range(0..scale.len() as i32);
                }
                scale[scale_pos as usize]
            }
            6 => {  // Chromatic passing tones
                let base = scale[scale_pos as usize % scale.len()];
                if rng.gen_ratio(1, 4) {
                    base + if rng.gen_bool(0.5) { 1 } else { -1 }
                } else {
                    scale_pos = (scale_pos + 1) % scale.len() as i32;
                    scale[scale_pos as usize]
                }
            }
            _ => {  // Wide leaps
                let leap = rng.gen_range(2..5);
                scale_pos = (scale_pos + leap) % scale.len() as i32;
                scale[scale_pos as usize] + if rng.gen_bool(0.5) { 12 } else { 0 }
            }
        };

        // EMBELLISHMENTS vary with seed
        let extra_offset: i8 = match embellish_style {
            1 if phrase_counter % 4 == 0 => 12,  // Octave jumps
            2 if rng.gen_ratio(1, 6) => 7,        // Fifth grace notes
            3 if phrase_counter % 2 == 1 => -12,  // Alternating octaves
            _ => 0,
        };

        // Phrase-based octave shifts
        let phrase_shift: i8 = if phrase_counter > 0 && phrase_counter % (phrase_length * 2) == 0 {
            match rng.gen_range(0..4) {
                0 => 12,
                1 => -12,
                _ => 0,
            }
        } else {
            0
        };

        let pitch = (base_pitch as i16 + interval as i16 + extra_offset as i16 + phrase_shift as i16)
            .clamp(36, 96) as u8;

        // Accent and velocity variation
        let accent = if phrase_counter % phrase_length == 0 { 12 } else { 0 };
        let vel = (velocity + accent).saturating_sub(rng.gen_range(0..10));

        notes.push(Note::new(pitch, note_duration, vel, time));

        phrase_counter += 1;
        time += step;
    }

    NoteSequence::new(notes, instrument, tempo)
}

/// Generate bass line with dramatic seed variation
fn generate_chip_bass(
    root: u8,
    beats: f64,
    tempo: u16,
    velocity: u8,
    variation: &PresetVariation,
    rng: &mut impl Rng,
) -> NoteSequence {
    let mut notes = Vec::new();
    let beat_duration = 60.0 / tempo as f64;

    // BASS RHYTHM varies dramatically
    let bass_style = variation.style_choices[1] % 6;
    let (note_duration, step) = match bass_style {
        0 => (beat_duration * 0.4, beat_duration * 0.5),    // 8th notes
        1 => (beat_duration * 0.9, beat_duration),           // Quarter notes (sustained)
        2 => (beat_duration * 0.15, beat_duration * 0.25),  // 16ths (driving)
        3 => (beat_duration * 0.3, beat_duration * 0.375),  // Dotted
        4 => (beat_duration * 1.8, beat_duration * 2.0),    // Half notes (sparse)
        _ => (beat_duration * 0.5, beat_duration * 0.666),  // Triplet feel
    };

    let mut time = 0.0;
    let end_time = beats * beat_duration;

    // BASS REGISTER varies
    let bass_root = match variation.contour_pattern % 4 {
        0 => root.saturating_sub(12).max(36),
        1 => root.saturating_sub(24).max(36),
        2 => root.max(36),  // Higher bass
        _ => root.saturating_sub(18).max(36),
    };

    // BASS PATTERNS vary dramatically
    let bass_patterns: &[&[i8]] = &[
        &[0, 0, 7, 7],                    // Root-fifth
        &[0, 12, 0, 7],                   // Octave bounce
        &[0, 5, 7, 12, 7, 5],             // Walking up and down
        &[0],                              // Just root (minimal)
        &[0, 0, 0, 0, 7, 7, 12, 12],      // Extended pattern
        &[0, -5, 0, 7, 12, 7],            // Wide range
        &[0, 3, 5, 3],                    // Minor third movement
        &[0, 7, 5, 7],                    // Fifth-fourth
        &[0, 0, 0, 0],                    // Pedal (just root)
        &[0, 7, 0, 5, 0, 3, 0, 2],        // Descending
    ];

    let pattern_idx = ((variation.style_choices[1] as usize) * 3 +
                       (variation.phrase_length as usize)) % bass_patterns.len();
    let pattern = bass_patterns[pattern_idx];
    let mut pos = 0;

    // BASS INSTRUMENT varies
    let bass_instrument = if variation.instrument_indices[1] % 2 == 0 {
        SYNTH_BASS_1
    } else {
        SYNTH_BASS_2
    };

    while time < end_time {
        let interval = pattern[pos % pattern.len()];
        let pitch = (bass_root as i16 + interval as i16).clamp(36, 60) as u8;

        let groove = if pos % 2 == 0 { 0 } else { rng.gen_range(3..12) };
        let vel = velocity.saturating_sub(groove);

        notes.push(Note::new(pitch, note_duration, vel, time));

        pos += 1;
        time += step;
    }

    NoteSequence::new(notes, bass_instrument, tempo)
}

/// Generate counter-melody with seed variation
fn generate_counter_melody(
    root: u8,
    scale: &[i8],
    beats: f64,
    tempo: u16,
    velocity: u8,
    variation: &PresetVariation,
    rng: &mut impl Rng,
) -> NoteSequence {
    let mut notes = Vec::new();
    let beat_duration = 60.0 / tempo as f64;

    // COUNTER-MELODY STYLE varies
    let style = variation.style_choices[2] % 5;
    let (note_duration, step, play_probability) = match style {
        0 => (beat_duration * 1.5, beat_duration * 2.0, 0.9),  // Very sparse
        1 => (beat_duration * 0.3, beat_duration * 0.5, 0.7),  // Active
        2 => (beat_duration * 0.8, beat_duration, 0.5),        // Half density
        3 => (beat_duration * 0.2, beat_duration * 0.25, 0.4), // Sparse 16ths
        _ => (beat_duration * 0.6, beat_duration * 0.75, 0.6), // Syncopated
    };

    let mut time = 0.0;
    let end_time = beats * beat_duration;

    // REGISTER varies
    let base = match variation.interval_style % 5 {
        0 => root + 7,   // Fifth up
        1 => root + 12,  // Octave up
        2 => root + 19,  // Octave + fifth
        3 => root + 4,   // Third up
        _ => root + 5,   // Fourth up
    };

    let contour = variation.get_contour(8);
    let mut contour_pos = 0;
    let mut scale_idx = (variation.scale_offset as usize) % scale.len();

    while time < end_time {
        if rng.gen_bool(play_probability) {
            let direction = contour[contour_pos % contour.len()];
            let interval_size = variation.get_interval(rng) as usize;

            match direction {
                1 => scale_idx = (scale_idx + interval_size) % scale.len(),
                -1 => scale_idx = (scale_idx + scale.len() - (interval_size % scale.len())) % scale.len(),
                _ => {}
            }

            let scale_note = scale[scale_idx];
            let octave: i8 = if rng.gen_ratio(1, 3) { 12 } else { 0 };
            let pitch = (base as i16 + scale_note as i16 + octave as i16).clamp(48, 96) as u8;

            let vel = velocity.saturating_sub(rng.gen_range(0..10));
            notes.push(Note::new(pitch, note_duration, vel, time));

            contour_pos += 1;
        }

        time += step;
    }

    NoteSequence::new(notes, SAWTOOTH_LEAD, tempo)
}

/// Generate percussion with seed variation
fn generate_chip_drums(
    beats: f64,
    tempo: u16,
    velocity: u8,
    variation: &PresetVariation,
    rng: &mut impl Rng,
) -> NoteSequence {
    let mut notes = Vec::new();
    let beat_duration = 60.0 / tempo as f64;

    let note_duration = beat_duration * 0.1;

    // RHYTHM GRID varies
    let step = match variation.style_choices[3] % 4 {
        0 => beat_duration * 0.25,   // 16th notes
        1 => beat_duration * 0.5,    // 8th notes
        2 => beat_duration * 0.333,  // Triplets
        _ => beat_duration * 0.125,  // 32nd notes
    };

    let mut time = 0.0;
    let end_time = beats * beat_duration;
    let mut beat_pos = 0;

    // KICK PATTERN varies dramatically
    let kick_patterns: &[&[bool]] = &[
        &[true, false, false, false, true, false, false, false],  // Basic 4/4
        &[true, false, false, true, false, false, true, false],   // Syncopated
        &[true, false, true, false, true, false, true, false],    // Double time
        &[true, false, false, false, false, false, false, false], // Very sparse
        &[true, true, false, false, true, true, false, false],    // Stuttered
        &[true, false, true, true, false, false, true, false],    // Complex
        &[true, false, false, false],                              // Simple
        &[true, false, false, true, true, false, false, false],   // Offbeat
    ];

    let kick_pattern = kick_patterns[(variation.style_choices[3] as usize +
                                       variation.phrase_length as usize) % kick_patterns.len()];

    // HAT DENSITY varies
    let hat_density = 0.4 + (variation.density_factor - 0.5) * 0.8;

    while time < end_time {
        let pattern_pos = beat_pos % kick_pattern.len();

        if kick_pattern[pattern_pos] {
            notes.push(Note::new(36, note_duration * 2.0, velocity, time));
        }

        if rng.gen_bool(hat_density.clamp(0.1, 0.95)) {
            let is_downbeat = beat_pos % 4 == 0;
            let hat_vel = if is_downbeat {
                velocity
            } else {
                velocity.saturating_sub(10 + rng.gen_range(0..15))
            };
            notes.push(Note::new(42, note_duration, hat_vel, time));
        }

        beat_pos += 1;
        time += step;
    }

    NoteSequence::new(notes, 115, tempo)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::preset::Key;

    #[test]
    fn test_chiptune_generates_notes() {
        let preset = ChiptunePreset;
        let config = PresetConfig {
            duration_secs: 3.0,
            key: Key::C,
            intensity: 70,
            seed: 42,
            tempo: 140,
        };

        let sequences = preset.generate(&config);
        assert!(!sequences.is_empty(), "Should generate sequences");

        for seq in &sequences {
            assert!(!seq.notes.is_empty(), "Each sequence should have notes");
        }
    }

    #[test]
    fn test_chiptune_varies_dramatically_with_seed() {
        let preset = ChiptunePreset;

        for (seed1, seed2) in [(1, 2), (10, 20), (100, 200)] {
            let config1 = PresetConfig {
                duration_secs: 3.0,
                key: Key::C,
                intensity: 70,
                seed: seed1,
                tempo: 140,
            };
            let config2 = PresetConfig {
                duration_secs: 3.0,
                key: Key::C,
                intensity: 70,
                seed: seed2,
                tempo: 140,
            };

            let seq1 = preset.generate(&config1);
            let seq2 = preset.generate(&config2);

            // Different number of layers indicates different structure
            let layer_diff = seq1.len() != seq2.len();

            // Different note counts indicate different rhythm/density
            let note_count_diff = seq1.iter().map(|s| s.notes.len()).sum::<usize>() !=
                                  seq2.iter().map(|s| s.notes.len()).sum::<usize>();

            // At least one structural difference
            assert!(layer_diff || note_count_diff,
                "Seeds {} and {} should produce structurally different output", seed1, seed2);
        }
    }
}
