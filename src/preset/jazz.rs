//! Jazz mood preset
//!
//! Characteristics: Swing feel, walking bass, piano comping with flourishes,
//! brushed drums (ride cymbal, soft hi-hat, gentle snare)

use super::{create_rng, MoodGenerator, PresetConfig, PresetVariation};
use crate::midi::{Note, NoteSequence};
use rand::Rng;

/// Jazz mood generator - nightclub trio style
pub struct JazzPreset;

/// Bass instrument: Acoustic Bass (GM 32) - plucked upright/double bass
/// This is the standard jazz walking bass sound
const BASS_INSTRUMENTS: &[u8] = &[32]; // Only acoustic bass for authentic jazz

/// Piano/keys instrument choices (acoustic piano preferred)
const KEYS_INSTRUMENTS: &[u8] = &[0, 0, 0, 1, 4]; // weighted toward acoustic grand

/// GM Drum note mappings (channel 9)
const DRUM_RIDE_CYMBAL: u8 = 51;
const DRUM_RIDE_BELL: u8 = 53;
const DRUM_CLOSED_HIHAT: u8 = 42;
const DRUM_PEDAL_HIHAT: u8 = 44;
const DRUM_SNARE: u8 = 38;
const DRUM_SIDE_STICK: u8 = 37;
const DRUM_BRUSH_SWIRL: u8 = 38; // Use snare with low velocity for brush effect

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
        let bass_style = match variation.pick_style(0, 4) {
            0 => BassStyle::Walking,
            1 => BassStyle::Walking,    // Prefer walking bass for jazz
            2 => BassStyle::TwoFeel,    // Half note feel for slower tunes
            _ => BassStyle::Syncopated,
        };

        let comp_style = match variation.pick_style(1, 4) {
            0 => CompStyle::Sparse,
            1 => CompStyle::Medium,
            2 => CompStyle::Medium,     // Prefer medium density for balanced sound
            _ => CompStyle::Dense,
        };

        // Choose instruments from variation
        let bass_inst = variation.pick_instrument(0, BASS_INSTRUMENTS);
        let keys_inst = variation.pick_instrument(1, KEYS_INSTRUMENTS);

        // Layer 1: Walking Bass on channel 1 (always included, prominent)
        let mut bass_seq = generate_walking_bass(config, &variation, beats, effective_tempo, bass_inst, bass_style, &mut rng);
        bass_seq.channel = 1; // Separate channel so bass instrument isn't overwritten
        sequences.push(bass_seq);

        // Layer 2: Piano comping on channel 0 (almost always included)
        if variation.layer_probs[1] > 0.05 {
            let mut piano_seq = generate_piano_comping(config, &variation, beats, effective_tempo, keys_inst, comp_style, &mut rng);
            piano_seq.channel = 0;
            sequences.push(piano_seq);
        }

        // Layer 3: Brushed drums on channel 9 (GM drum channel)
        if variation.layer_probs[2] > 0.1 {
            sequences.push(generate_brush_drums(config, &variation, beats, effective_tempo, &mut rng));
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

/// Generate walking bass line - the foundation of jazz trio
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

    // Scale degrees relative to root (all notes that sound good in the key)
    // For minor: natural minor with added chromatic approach notes
    // For major: major scale with bebop passing tones
    let scale_intervals: &[u8] = if config.key.is_minor() {
        &[0, 2, 3, 5, 7, 8, 10, 12] // Natural minor scale
    } else {
        &[0, 2, 4, 5, 7, 9, 11, 12] // Major scale
    };

    // Build all valid bass notes in range (two octaves centered on bass root)
    let bass_root = root.saturating_sub(24).max(28); // Two octaves down, but not below E1
    let mut scale_notes: Vec<u8> = Vec::new();
    for octave_offset in [-12i8, 0, 12] {
        for &interval in scale_intervals {
            let note = (bass_root as i8 + octave_offset + interval as i8) as u8;
            if note >= 28 && note <= bass_root + 14 {
                scale_notes.push(note);
            }
        }
    }
    scale_notes.sort();
    scale_notes.dedup();

    // Chord tones (root, 3rd, 5th, 7th) - these are "strong" notes for downbeats
    let chord_tones: Vec<u8> = if config.key.is_minor() {
        vec![bass_root, bass_root + 3, bass_root + 7, bass_root + 10] // m7
    } else {
        vec![bass_root, bass_root + 4, bass_root + 7, bass_root + 11] // maj7
    };

    let mut t = 0.0;
    let mut last_pitch = bass_root;

    // Determine step size and duration based on style
    let (step, base_duration) = match style {
        BassStyle::Walking => (1.0, 0.95),
        BassStyle::TwoFeel => (2.0, 1.9),
        BassStyle::Syncopated => (1.0, 0.8),
    };

    // Get contour for melodic direction - this makes bass lines differ by seed
    let phrase_len = variation.phrase_length as usize;
    let contour = variation.get_contour(phrase_len);
    let mut phrase_pos = 0;

    // Helper to find nearest scale note
    let find_scale_step = |from: u8, direction: i8| -> u8 {
        let idx = scale_notes.iter().position(|&n| n >= from).unwrap_or(0);
        if direction > 0 {
            // Moving up - get next note in scale
            scale_notes.get(idx + 1).copied().unwrap_or(from)
        } else {
            // Moving down - get previous note in scale
            if idx > 0 {
                scale_notes[idx - 1]
            } else {
                from
            }
        }
    };

    while t < beats {
        // For syncopated style, use seed-based rest probability
        if matches!(style, BassStyle::Syncopated) && variation.should_rest(rng) {
            t += 0.5;
            phrase_pos += 1;
            continue;
        }

        // Get contour direction for this position
        let contour_dir = contour[phrase_pos % contour.len()];

        // Walking bass: contour-guided motion with occasional leaps
        let pitch = if t == 0.0 {
            // Start on root
            bass_root
        } else if rng.gen_bool(0.55) {
            // Follow contour direction for stepwise motion
            let direction = match contour_dir {
                1 => 1i8,
                -1 => -1i8,
                _ => if rng.gen_bool(0.5) { 1 } else { -1 },
            };
            find_scale_step(last_pitch, direction)
        } else if rng.gen_bool(0.5) {
            // Leap to a chord tone (sounds strong and anchored)
            chord_tones[rng.gen_range(0..chord_tones.len())]
        } else {
            // Chromatic approach (half step to a chord tone - classic jazz move)
            let target = chord_tones[rng.gen_range(0..chord_tones.len())];
            if rng.gen_bool(0.5) {
                target.saturating_sub(1).max(28)
            } else {
                target.saturating_add(1).min(bass_root + 12)
            }
        };

        phrase_pos += 1;

        // Strong velocity with jazzy dynamic variation
        let vel_base = 95 + (config.intensity as i32 / 10) as u8;
        // Accent beat 1 and 3 more, beats 2 and 4 slightly softer for groove
        let beat_num = t as i32 % 4;
        let accent = if beat_num == 0 || beat_num == 2 { 5 } else { -3i8 as u8 };
        let velocity = variation
            .adjust_velocity(vel_base.saturating_add(accent))
            .saturating_add(rng.gen_range(0..8))
            .min(127);

        // Swing timing: slightly delay offbeat notes for swing feel
        let swing_offset = if (t * 2.0) as i32 % 2 == 1 {
            rng.gen_range(0.02..0.08) // Swing the offbeats
        } else {
            rng.gen_range(-0.02..0.02) // Slight humanization on downbeats
        };
        let actual_time = (t + swing_offset).max(0.0);

        // Duration with slight variation
        let duration: f64 = base_duration + rng.gen_range(-0.05..0.05);

        // Occasional grace note slide into the main note (chromatic approach)
        if rng.gen_bool(0.15) && actual_time > 0.1 {
            let grace_pitch = if rng.gen_bool(0.5) {
                pitch.saturating_sub(1)
            } else {
                pitch.saturating_add(1)
            };
            notes.push(Note::new(
                grace_pitch,
                0.08,
                (velocity as i32 - 20).max(40) as u8,
                actual_time - 0.08,
            ));
        }

        notes.push(Note::new(pitch, duration.max(0.1), velocity, actual_time));
        last_pitch = pitch;

        // Ghost notes on upbeats for syncopated style
        if matches!(style, BassStyle::Syncopated) && rng.gen_bool(0.25) {
            let ghost_pitch = chord_tones[rng.gen_range(0..chord_tones.len())];
            let ghost_time = t + 0.5 + rng.gen_range(0.0..0.05); // Slight timing variation
            if ghost_time < beats {
                notes.push(Note::new(ghost_pitch, 0.2, vel_base - 35, ghost_time));
            }
        }

        t += step;
    }

    NoteSequence::new(notes, instrument, tempo)
}

/// Generate jazz piano comping with chords and flourishes
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

    // Jazz voicings in comfortable piano range (around middle C)
    let voicings: Vec<Vec<i8>> = if config.key.is_minor() {
        vec![
            vec![3, 7, 10, 14],    // m9 (3rd, 5th, 7th, 9th)
            vec![3, 10, 14],       // m7 spread
            vec![10, 14, 17],      // m9 upper
            vec![-2, 3, 7, 10],    // m7 with 9th below
            vec![3, 7, 10],        // m7 basic
            vec![7, 10, 14, 17],   // m11 voicing
        ]
    } else {
        vec![
            vec![4, 7, 11, 14],    // maj9 (3rd, 5th, 7th, 9th)
            vec![4, 11, 14],       // maj7 spread
            vec![11, 14, 16],      // maj9 upper
            vec![-1, 4, 7, 11],    // maj7 with 7th below
            vec![4, 7, 11],        // maj7 basic
            vec![7, 11, 14, 18],   // maj9#11 upper
        ]
    };

    // Skip probability based on style (less skipping = more comping)
    let skip_prob = match style {
        CompStyle::Sparse => 0.45,
        CompStyle::Medium => 0.25,
        CompStyle::Dense => 0.10,
    };

    // Step size varies by style
    let base_step = match style {
        CompStyle::Sparse => 2.0,
        CompStyle::Medium => 1.5,
        CompStyle::Dense => 1.0,
    };

    // Get contour for voicing selection variation
    let phrase_len = variation.phrase_length as usize;
    let contour = variation.get_contour(phrase_len);
    let mut voicing_idx = (variation.scale_offset as usize) % voicings.len();
    let mut phrase_pos = 0;

    let mut t = 0.0;

    while t < beats - 0.5 {
        // Skip based on seed rest probability instead of fixed prob
        if variation.should_rest(rng) || rng.gen_bool(skip_prob * 0.5) {
            t += rng.gen_range(0.5..1.0);
            phrase_pos += 1;
            continue;
        }

        // Choose voicing based on contour-guided index
        let voicing = &voicings[voicing_idx % voicings.len()];

        // Swing feel: slightly late on offbeats
        let swing_offset = if rng.gen_bool(0.4) {
            rng.gen_range(0.1..0.4)
        } else {
            0.0
        };

        let chord_time = t + swing_offset;
        if chord_time >= beats {
            break;
        }

        // Varied chord durations (staccato to legato)
        let duration = if rng.gen_bool(0.3) {
            0.2 // Staccato stabs
        } else if rng.gen_bool(0.4) {
            0.5 // Medium
        } else {
            rng.gen_range(0.8..1.2) // Sustained
        };

        // Piano comping sits well behind the bass (30-40 velocity range)
        // Chords have 3-4 notes so cumulative volume is higher than single notes
        let vel_base = 30 + (config.intensity as i32 / 12) as u8;
        let vel_base = variation.adjust_velocity(vel_base);

        for (i, &interval) in voicing.iter().enumerate() {
            let pitch = ((root as i8 + interval) as u8).clamp(48, 84); // Keep in piano sweet spot
            // Top notes slightly louder
            let vel = vel_base.saturating_add(i as u8 * 2).saturating_add(rng.gen_range(0..10));
            notes.push(Note::new(pitch, duration, vel.min(110), chord_time));
        }

        // Add flourishes (grace notes, runs) occasionally
        if rng.gen_bool(0.15) && chord_time + 0.5 < beats {
            add_piano_flourish(&mut notes, root, chord_time, &config.key, rng);
        }

        // Move voicing selection based on contour
        let direction = contour[phrase_pos % contour.len()];
        match direction {
            1 => voicing_idx = (voicing_idx + 1) % voicings.len(),
            -1 => voicing_idx = if voicing_idx > 0 { voicing_idx - 1 } else { voicings.len() - 1 },
            _ => {} // Stay on current voicing
        }
        phrase_pos += 1;

        // Step forward with variation
        t += base_step + rng.gen_range(-0.3..0.3);
    }

    NoteSequence::new(notes, instrument, tempo)
}

/// Add a jazz piano flourish (short melodic run)
fn add_piano_flourish(
    notes: &mut Vec<Note>,
    root: u8,
    start_time: f64,
    key: &super::Key,
    rng: &mut impl Rng,
) {
    let intervals = key.scale_intervals();
    let flourish_start = start_time + rng.gen_range(0.5..0.8);

    // Pick random starting point in scale
    let start_degree = rng.gen_range(0..intervals.len());
    let direction: i8 = if rng.gen_bool(0.5) { 1 } else { -1 };
    let num_notes = rng.gen_range(2..5);

    let base_vel = 35; // Flourishes quieter than comping chords

    for i in 0..num_notes {
        let degree = (start_degree as i8 + direction * i as i8).rem_euclid(intervals.len() as i8) as usize;
        let pitch = (root + intervals[degree] + 12).clamp(60, 84); // Upper octave
        let time = flourish_start + i as f64 * 0.1;
        let vel = base_vel + rng.gen_range(0..15);
        notes.push(Note::new(pitch, 0.15, vel, time));
    }
}

/// Generate brushed drum pattern on GM channel 9
/// Soft jazz brushes: ride cymbal, gentle hi-hat, occasional snare swirls
fn generate_brush_drums(
    config: &PresetConfig,
    _variation: &PresetVariation,
    beats: f64,
    tempo: u16,
    rng: &mut impl Rng,
) -> NoteSequence {
    let _ = config; // May use intensity later
    let mut notes = Vec::new();

    // Swing ratio: 0.67 = classic swing feel
    let swing_ratio = rng.gen_range(0.62..0.72);

    let mut t = 0.0;

    while t < beats {
        // Ride cymbal: main timekeeping (every beat)
        // Ride cymbal - prominent in jazz trio
        let ride_vel = 65 + rng.gen_range(0..20);
        notes.push(Note::new(DRUM_RIDE_CYMBAL, 0.2, ride_vel, t));

        // Swung "and" on ride (the skip beat)
        let and_time = t + swing_ratio;
        if and_time < beats && rng.gen_bool(0.85) {
            let and_vel = 55 + rng.gen_range(0..15);
            // Alternate between ride cymbal and ride bell for variation
            let ride_sound = if rng.gen_bool(0.8) { DRUM_RIDE_CYMBAL } else { DRUM_RIDE_BELL };
            notes.push(Note::new(ride_sound, 0.15, and_vel, and_time));
        }

        // Hi-hat: pedal hits on beats 2 and 4
        if (t as i32) % 2 == 1 {
            let hh_vel = 50 + rng.gen_range(0..15);
            notes.push(Note::new(DRUM_PEDAL_HIHAT, 0.1, hh_vel, t));
        }

        // Occasional closed hi-hat on offbeats
        if rng.gen_bool(0.2) {
            let offbeat_time = t + 0.5;
            if offbeat_time < beats {
                notes.push(Note::new(DRUM_CLOSED_HIHAT, 0.08, 45 + rng.gen_range(0..10), offbeat_time));
            }
        }

        // Snare brush swirl: hits on 2 and 4 (classic jazz backbeat)
        // Using side stick or soft snare for brush effect
        if (t as i32) % 2 == 1 && rng.gen_bool(0.7) {
            let snare_vel = 50 + rng.gen_range(0..20); // Brush feel
            let snare_sound = if rng.gen_bool(0.6) { DRUM_SIDE_STICK } else { DRUM_SNARE };
            notes.push(Note::new(snare_sound, 0.15, snare_vel, t));
        }

        // Occasional brush swirl across beats (ghost notes)
        if rng.gen_bool(0.1) {
            let swirl_time = t + rng.gen_range(0.2..0.4);
            if swirl_time < beats {
                notes.push(Note::new(DRUM_BRUSH_SWIRL, 0.3, 40 + rng.gen_range(0..10), swirl_time));
            }
        }

        t += 1.0;
    }

    // Create drum sequence on channel 9 (GM drums)
    let mut seq = NoteSequence::new(notes, 0, tempo);
    seq.channel = 9; // GM drum channel
    seq
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
        // Bass is locked to acoustic bass (32) for authentic jazz sound
        // Instead, verify piano/keys instrument varies between seeds
        let piano_instruments: Vec<u8> = (1..=20)
            .filter_map(|seed| {
                let config = PresetConfig {
                    seed,
                    duration_secs: 3.0,
                    ..Default::default()
                };
                let seqs = JazzPreset.generate(&config);
                // Piano is second sequence if present (bass is first)
                seqs.get(1).map(|s| s.instrument)
            })
            .collect();

        // Bass should always be acoustic bass (32)
        let bass_config = PresetConfig {
            seed: 42,
            duration_secs: 3.0,
            ..Default::default()
        };
        let seqs = JazzPreset.generate(&bass_config);
        assert_eq!(seqs[0].instrument, 32, "Bass should be acoustic bass (GM 32)");

        // Piano instruments should vary
        let unique_instruments: std::collections::HashSet<_> = piano_instruments.iter().collect();
        assert!(
            unique_instruments.len() > 1,
            "Piano instrument should vary across seeds: got {:?}",
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
