//! Jazz mood preset
//!
//! Characteristics: Swing feel, walking bass, piano comping, ride cymbal

use super::{create_rng, MoodGenerator, PresetConfig, PresetVariation};
use crate::midi::{Note, NoteSequence};
use rand::Rng;

/// Jazz mood generator - nightclub trio style
pub struct JazzPreset;

/// Bass instrument choices
const BASS_INSTRUMENTS: &[u8] = &[32, 33, 34, 35]; // acoustic, finger, pick, fretless

/// Piano/keys instrument choices
const KEYS_INSTRUMENTS: &[u8] = &[0, 1, 2, 4, 5, 7]; // various pianos/keys

/// Bass pattern styles
#[derive(Clone, Copy)]
enum BassStyle {
    Walking,    // Quarter notes
    TwoFeel,    // Half notes
    Syncopated, // Mix of rhythms
}

/// Comping density styles
#[derive(Clone, Copy)]
enum CompStyle {
    Sparse,  // Few chords, more space
    Medium,  // Balanced
    Dense,   // More active comping
}

impl MoodGenerator for JazzPreset {
    fn generate(&self, config: &PresetConfig) -> Vec<NoteSequence> {
        // Use centralized variation for consistent seed-based differences
        let variation = PresetVariation::from_seed(config.seed);
        let mut rng = create_rng(config.seed);
        let mut sequences = Vec::new();

        let effective_tempo = variation.effective_tempo(config.tempo);
        let beats = config.duration_secs * effective_tempo as f64 / 60.0;

        // Choose styles from variation
        let bass_style = match variation.pick_style(0, 3) {
            0 => BassStyle::Walking,
            1 => BassStyle::TwoFeel,
            _ => BassStyle::Syncopated,
        };

        let comp_style = match variation.pick_style(1, 3) {
            0 => CompStyle::Sparse,
            1 => CompStyle::Medium,
            _ => CompStyle::Dense,
        };

        // Choose instruments from variation
        let bass_inst = variation.pick_instrument(0, BASS_INSTRUMENTS);
        let keys_inst = variation.pick_instrument(1, KEYS_INSTRUMENTS);

        // Layer 1: Bass (always included, style varies)
        sequences.push(generate_walking_bass(config, &variation, beats, effective_tempo, bass_inst, bass_style, &mut rng));

        // Layer 2: Piano comping (based on variation probability)
        if variation.layer_probs[1] > 0.15 {
            sequences.push(generate_piano_comping(config, &variation, beats, effective_tempo, keys_inst, comp_style, &mut rng));
        }

        // Layer 3: Ride cymbal (variation + intensity based)
        if variation.layer_probs[2] > (0.7 - config.intensity as f64 / 150.0) {
            sequences.push(generate_ride_pattern(config, &variation, beats, effective_tempo, &mut rng));
        }

        // Layer 4: Hi-hat accents (variation + intensity based)
        if variation.layer_probs[3] > (0.6 - config.intensity as f64 / 200.0) {
            sequences.push(generate_hihat_accents(&variation, beats, effective_tempo, &mut rng));
        }

        sequences
    }

    fn name(&self) -> &'static str {
        "jazz"
    }

    fn description(&self) -> &'static str {
        "Nightclub trio style with walking bass, piano comping, and brushed drums"
    }
}

/// Generate bass line with style variation
fn generate_walking_bass(
    config: &PresetConfig,
    variation: &PresetVariation,
    beats: f64,
    tempo: u16,
    instrument: u8,
    style: BassStyle,
    rng: &mut impl Rng,
) -> NoteSequence {
    let root = config.key.root();
    let mut notes = Vec::new();

    // Chord tones for the key (in bass register)
    let bass_root = root - 24; // Two octaves down
    let chord_tones: Vec<u8> = if config.key.is_minor() {
        vec![bass_root, bass_root + 3, bass_root + 7, bass_root + 10] // m7
    } else {
        vec![bass_root, bass_root + 4, bass_root + 7, bass_root + 11] // maj7
    };

    let mut t = 0.0;

    // Determine step size and duration based on style
    let (step, base_duration) = match style {
        BassStyle::Walking => (1.0, 0.9),
        BassStyle::TwoFeel => (2.0, 1.8),
        BassStyle::Syncopated => (1.0, 0.7),
    };

    while t < beats {
        // For syncopated style, sometimes skip or add extra notes
        if matches!(style, BassStyle::Syncopated) && rng.gen_bool(0.2) {
            t += 0.5;
            continue;
        }

        // Choose next target (chord tone)
        let target = chord_tones[rng.gen_range(0..chord_tones.len())];

        // Chromatic approach probability varies by style
        let chromatic_prob = match style {
            BassStyle::Walking => 0.35,
            BassStyle::TwoFeel => 0.15,
            BassStyle::Syncopated => 0.45,
        };

        let pitch = if rng.gen_bool(chromatic_prob) && t > 0.0 {
            if rng.gen_bool(0.5) {
                target.saturating_sub(1)
            } else {
                target.saturating_add(1).min(127)
            }
        } else if rng.gen_bool(0.25) {
            let intervals = config.key.scale_intervals();
            let interval = intervals[rng.gen_range(0..intervals.len())];
            (bass_root + interval).min(bass_root + 12)
        } else {
            target
        };

        // Velocity variation increases with syncopated style
        let vel_base = variation.adjust_velocity(70 + (config.intensity as i32 / 5) as u8);
        let vel_range = match style {
            BassStyle::Walking => rng.gen_range(0..15),
            BassStyle::TwoFeel => rng.gen_range(0..10),
            BassStyle::Syncopated => rng.gen_range(0..25),
        };
        let velocity = vel_base.saturating_add(vel_range);

        // Duration varies slightly
        let duration: f64 = base_duration + rng.gen_range(-0.1..0.1);

        notes.push(Note::new(pitch, duration.max(0.1), velocity, t));

        // Syncopated style sometimes adds ghost notes
        if matches!(style, BassStyle::Syncopated) && rng.gen_bool(0.3) {
            let ghost_pitch = chord_tones[rng.gen_range(0..chord_tones.len())];
            let ghost_time = t + 0.5;
            if ghost_time < beats {
                notes.push(Note::new(ghost_pitch, 0.3, vel_base - 20, ghost_time));
            }
        }

        t += step;
    }

    NoteSequence::new(notes, instrument, tempo)
}

/// Generate jazz piano comping with style variation
fn generate_piano_comping(
    config: &PresetConfig,
    variation: &PresetVariation,
    beats: f64,
    tempo: u16,
    instrument: u8,
    style: CompStyle,
    rng: &mut impl Rng,
) -> NoteSequence {
    let root = config.key.root();
    let mut notes = Vec::new();

    // Jazz voicings - more options for variety
    let voicings: Vec<Vec<i8>> = if config.key.is_minor() {
        vec![
            vec![3, 7, 10],       // m7 (3rd, 5th, 7th)
            vec![3, 10, 14],      // m7 spread
            vec![10, 14, 17],     // m9 upper
            vec![-2, 3, 7],       // with 9th below
            vec![3, 7],           // shell (3rd, 5th)
            vec![10, 15, 19],     // m11 upper structure
        ]
    } else {
        vec![
            vec![4, 7, 11],       // maj7 (3rd, 5th, 7th)
            vec![4, 11, 14],      // maj7 spread
            vec![11, 14, 16],     // maj9 upper
            vec![-1, 4, 7],       // with 7th below
            vec![4, 11],          // shell (3rd, 7th)
            vec![11, 14, 18],     // maj9#11 upper
        ]
    };

    // Skip probability based on style
    let skip_prob = match style {
        CompStyle::Sparse => 0.55,
        CompStyle::Medium => 0.35,
        CompStyle::Dense => 0.15,
    };

    // Step size varies by style
    let base_step = match style {
        CompStyle::Sparse => 2.5,
        CompStyle::Medium => 1.5,
        CompStyle::Dense => 1.0,
    };

    let mut t = 0.0;

    while t < beats - 0.5 {
        // Skip some beats
        if rng.gen_bool(skip_prob) {
            t += rng.gen_range(0.5..1.5);
            continue;
        }

        // Choose voicing
        let voicing = &voicings[rng.gen_range(0..voicings.len())];

        // Swing offset varies by style
        let swing_offset = match style {
            CompStyle::Sparse => {
                if rng.gen_bool(0.4) { 0.33 } else { 0.0 }
            }
            CompStyle::Medium => {
                if rng.gen_bool(0.35) { 0.33 }
                else if rng.gen_bool(0.25) { 0.5 }
                else { 0.0 }
            }
            CompStyle::Dense => {
                if rng.gen_bool(0.5) { rng.gen_range(0.0..0.5) }
                else { 0.0 }
            }
        };

        let chord_time = t + swing_offset;
        if chord_time >= beats {
            break;
        }

        // Duration varies by style
        let duration = match style {
            CompStyle::Sparse => rng.gen_range(0.8..1.5),
            CompStyle::Medium => {
                if rng.gen_bool(0.3) { 0.25 }
                else if rng.gen_bool(0.4) { 0.5 }
                else { 1.0 }
            }
            CompStyle::Dense => rng.gen_range(0.2..0.6),
        };

        let vel_base = variation.adjust_velocity(45 + (config.intensity as i32 / 4) as u8);

        for (i, &interval) in voicing.iter().enumerate() {
            let pitch = (root as i8 + interval) as u8;
            let vel = vel_base.saturating_sub(i as u8 * 3).saturating_add(rng.gen_range(0..15));
            notes.push(Note::new(pitch, duration, vel, chord_time));
        }

        // Step forward
        t += base_step + rng.gen_range(-0.5..0.5);
    }

    NoteSequence::new(notes, instrument, tempo)
}

/// Generate ride cymbal pattern with swing
fn generate_ride_pattern(
    config: &PresetConfig,
    variation: &PresetVariation,
    beats: f64,
    tempo: u16,
    rng: &mut impl Rng,
) -> NoteSequence {
    let _ = config; // May use intensity later
    let mut notes = Vec::new();

    // Ride variations
    let ride_pitches = [51u8, 53, 59]; // Different ride sounds
    let ride = ride_pitches[rng.gen_range(0..ride_pitches.len())];

    let mut t = 0.0;

    // Swing ratio varies: 0.5 = straight, 0.67 = swing, 0.75 = hard swing
    let swing_ratio = rng.gen_range(0.55..0.72);

    // Pattern density: sometimes skip beats
    let density = rng.gen_range(0.7..1.0);

    while t < beats {
        if rng.gen_bool(density) {
            // Main ride hit
            let vel = 65 + rng.gen_range(0..25);
            notes.push(Note::new(ride, 0.3, vel, t));
        }

        // Swung "and" hit
        let and_time = t + swing_ratio;
        if and_time < beats && rng.gen_bool(0.75 * density) {
            let and_vel = 50 + rng.gen_range(0..20);
            notes.push(Note::new(ride, 0.25, and_vel, and_time));
        }

        t += 1.0;
    }

    // Use tubular bells (14) or celesta (8) for ride-like sound
    let instrument = if rng.gen_bool(0.7) { 14 } else { 8 };
    NoteSequence::new(notes, instrument, tempo)
}

/// Generate hi-hat accents with variation
fn generate_hihat_accents(
    variation: &PresetVariation,
    beats: f64,
    tempo: u16,
    rng: &mut impl Rng,
) -> NoteSequence {
    let _ = variation; // Using for consistency
    let mut notes = Vec::new();

    // Hi-hat pitch varies
    let hihat_pitch = 75 + rng.gen_range(0..10);

    // Pattern style: 2-and-4 vs every beat vs sparse
    let pattern = rng.gen_range(0..3);

    let mut t = match pattern {
        0 => 1.0, // 2 and 4
        1 => 0.0, // Every beat
        _ => 0.5, // Off-beats
    };

    let step = match pattern {
        0 => 2.0,
        1 => 1.0,
        _ => 2.0,
    };

    while t < beats {
        let vel = 55 + rng.gen_range(0..20);
        notes.push(Note::new(hihat_pitch, 0.08, vel, t));
        t += step;
    }

    // Woodblock (115) or agogo (113) for click sound
    let instrument = if rng.gen_bool(0.6) { 115 } else { 113 };
    NoteSequence::new(notes, instrument, tempo)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::preset::Key;

    #[test]
    fn test_jazz_generates_sequences() {
        let config = PresetConfig {
            intensity: 70,
            ..Default::default()
        };
        let sequences = JazzPreset.generate(&config);
        // Should have at least bass (always included)
        assert!(!sequences.is_empty());
        // Most seeds will produce 2-4 layers
        assert!(sequences.len() >= 1 && sequences.len() <= 4);
    }

    #[test]
    fn test_jazz_bass_has_notes() {
        let config = PresetConfig {
            duration_secs: 4.0,
            tempo: 60,
            ..Default::default()
        };
        let sequences = JazzPreset.generate(&config);

        // First sequence is always bass
        let bass = &sequences[0];
        // Should have some notes (varies by style)
        assert!(!bass.notes.is_empty(), "Bass should have notes");
    }

    #[test]
    fn test_jazz_different_seeds_vary() {
        let config1 = PresetConfig {
            seed: 1,
            duration_secs: 5.0,
            ..Default::default()
        };
        let config2 = PresetConfig {
            seed: 999,
            duration_secs: 5.0,
            ..Default::default()
        };

        let seq1 = JazzPreset.generate(&config1);
        let seq2 = JazzPreset.generate(&config2);

        // Different seeds should produce different results
        // Check layer count or note count differs
        let notes1: usize = seq1.iter().map(|s| s.notes.len()).sum();
        let notes2: usize = seq2.iter().map(|s| s.notes.len()).sum();

        // Very unlikely to be exactly the same
        assert!(
            seq1.len() != seq2.len() || notes1 != notes2 || seq1[0].instrument != seq2[0].instrument,
            "Different seeds should produce variation"
        );
    }

    #[test]
    fn test_jazz_adjacent_seeds_produce_noticeable_differences() {
        // Even seeds 42 and 43 should produce audibly different results
        let config42 = PresetConfig {
            seed: 42,
            duration_secs: 8.0,
            intensity: 60,
            ..Default::default()
        };
        let config43 = PresetConfig {
            seed: 43,
            duration_secs: 8.0,
            intensity: 60,
            ..Default::default()
        };

        let seq42 = JazzPreset.generate(&config42);
        let seq43 = JazzPreset.generate(&config43);

        // Count differences
        let mut differences = 0;

        // Check layer count
        if seq42.len() != seq43.len() {
            differences += 10; // Major difference
        }

        // Check instruments used
        for (s1, s2) in seq42.iter().zip(seq43.iter()) {
            if s1.instrument != s2.instrument {
                differences += 5;
            }
        }

        // Check note pitches in first layer (bass)
        let pitches42: Vec<u8> = seq42[0].notes.iter().map(|n| n.pitch).collect();
        let pitches43: Vec<u8> = seq43[0].notes.iter().map(|n| n.pitch).collect();
        for (p1, p2) in pitches42.iter().zip(pitches43.iter()) {
            if p1 != p2 {
                differences += 1;
            }
        }

        // Check velocities
        let vels42: Vec<u8> = seq42[0].notes.iter().map(|n| n.velocity).collect();
        let vels43: Vec<u8> = seq43[0].notes.iter().map(|n| n.velocity).collect();
        for (v1, v2) in vels42.iter().zip(vels43.iter()) {
            if v1 != v2 {
                differences += 1;
            }
        }

        // Should have multiple noticeable differences
        assert!(
            differences >= 3,
            "Adjacent seeds should produce at least 3 differences, got {}",
            differences
        );
    }

    #[test]
    fn test_jazz_tempo_varies_by_seed() {
        // Check that effective tempo varies between seeds
        let tempos: Vec<u16> = (1..=10)
            .map(|seed| {
                let config = PresetConfig {
                    seed,
                    tempo: 100,
                    duration_secs: 4.0,
                    ..Default::default()
                };
                let seqs = JazzPreset.generate(&config);
                seqs[0].tempo
            })
            .collect();

        // Not all tempos should be 100 (some variation expected)
        let unique_tempos: std::collections::HashSet<_> = tempos.iter().collect();
        assert!(
            unique_tempos.len() > 1,
            "Tempo should vary across different seeds"
        );
    }

    #[test]
    fn test_jazz_instruments_vary_by_seed() {
        // Check that bass instrument varies between seeds
        let bass_instruments: Vec<u8> = (1..=20)
            .map(|seed| {
                let config = PresetConfig {
                    seed,
                    duration_secs: 3.0,
                    ..Default::default()
                };
                let seqs = JazzPreset.generate(&config);
                seqs[0].instrument
            })
            .collect();

        let unique_instruments: std::collections::HashSet<_> = bass_instruments.iter().collect();
        assert!(
            unique_instruments.len() > 1,
            "Bass instrument should vary across seeds: got {:?}",
            unique_instruments
        );
    }

    #[test]
    fn test_jazz_uses_bass_register() {
        let config = PresetConfig {
            key: Key::F,
            ..Default::default()
        };
        let sequences = JazzPreset.generate(&config);

        let bass = &sequences[0];
        for note in &bass.notes {
            assert!(note.pitch < 72, "Bass notes should be in lower register");
        }
    }
}
