//! Show/Musical mood preset
//!
//! Characteristics: Broadway/Hollywood musical style with orchestral strings,
//! brass fanfares, piano accompaniment, and singable melodies.
//! Big dynamic builds, clear phrase structures, theatrical flair.

use super::{create_rng, MoodGenerator, PresetConfig, PresetVariation};
use crate::midi::{Note, NoteSequence};
use rand::Rng;

/// Show/Musical mood generator - Broadway/Hollywood style
pub struct ShowPreset;

/// String ensemble instruments (lush pads)
const STRING_INSTRUMENTS: &[u8] = &[
    48, // String Ensemble 1
    49, // String Ensemble 2
    44, // Tremolo Strings
    45, // Pizzicato Strings (for rhythmic accents)
    50, // Synth Strings 1
];

/// Brass instruments (fanfares and accents)
const BRASS_INSTRUMENTS: &[u8] = &[
    61, // Brass Section
    60, // French Horn
    57, // Trumpet
    58, // Trombone
    59, // Tuba (for low brass)
];

/// Piano/keyboard instruments (accompaniment)
const PIANO_INSTRUMENTS: &[u8] = &[
    0,  // Acoustic Grand Piano
    1,  // Bright Acoustic Piano
    2,  // Electric Grand Piano
    46, // Orchestral Harp
    8,  // Celesta (for sparkle)
];

/// Melody instruments (vocal-range, expressive)
const MELODY_INSTRUMENTS: &[u8] = &[
    73, // Flute
    68, // Oboe
    71, // Clarinet
    65, // Alto Sax (jazzy show tunes)
    40, // Violin
    0,  // Piano (for ballads)
];

/// Accompaniment patterns (Broadway style)
const ACCOMP_PATTERNS: &[&[f64]] = &[
    &[0.0, 0.5, 1.0, 1.5, 2.0, 2.5, 3.0, 3.5],     // Steady eighths
    &[0.0, 1.0, 1.5, 2.0, 3.0, 3.5],               // Broadway swing
    &[0.0, 0.5, 1.5, 2.0, 2.5, 3.5],               // Syncopated show
    &[0.0, 1.0, 2.0, 3.0],                          // Quarter note ballad
    &[0.0, 0.33, 0.67, 1.0, 1.33, 1.67, 2.0, 2.33, 2.67, 3.0, 3.33, 3.67], // Triplet feel
];

impl MoodGenerator for ShowPreset {
    fn generate(&self, config: &PresetConfig) -> Vec<NoteSequence> {
        let variation = PresetVariation::from_seed(config.seed);
        let mut rng = create_rng(config.seed);
        let mut sequences = Vec::new();

        let effective_tempo = variation.effective_tempo(config.tempo);
        let beats = config.duration_secs * effective_tempo as f64 / 60.0;

        // Choose instruments using variation system
        let string_inst = variation.pick_instrument(0, STRING_INSTRUMENTS);
        let brass_inst = variation.pick_instrument(1, BRASS_INSTRUMENTS);
        let piano_inst = variation.pick_instrument(2, PIANO_INSTRUMENTS);
        let melody_inst = variation.pick_instrument(3, MELODY_INSTRUMENTS);

        // Choose accompaniment pattern
        let pattern_idx = variation.pick_style(0, ACCOMP_PATTERNS.len());

        // Layer 1: String pad (foundation - always included)
        let mut strings = generate_string_pad(config, &variation, beats, effective_tempo, string_inst, &mut rng);
        strings.channel = 0;
        sequences.push(strings);

        // Layer 2: Piano accompaniment (high probability)
        if variation.layer_probs[1] > 0.15 {
            let mut piano = generate_piano_accomp(config, &variation, beats, effective_tempo, piano_inst, pattern_idx, &mut rng);
            piano.channel = 1;
            sequences.push(piano);
        }

        // Layer 3: Melody line (probability based on intensity)
        let melody_threshold = 0.5 - (config.intensity as f64 / 200.0);
        if variation.layer_probs[2] > melody_threshold {
            let mut melody = generate_melody_line(config, &variation, beats, effective_tempo, melody_inst, &mut rng);
            melody.channel = 2;
            sequences.push(melody);
        }

        // Layer 4: Brass fanfare/accents (probability)
        if variation.layer_probs[3] > 0.4 {
            let mut brass = generate_brass_accents(config, &variation, beats, effective_tempo, brass_inst, &mut rng);
            brass.channel = 3;
            sequences.push(brass);
        }

        // Layer 5: Percussion/timpani (lower probability)
        if variation.layer_probs[4] > 0.6 {
            let mut perc = generate_theatrical_percussion(config, &variation, beats, effective_tempo, &mut rng);
            perc.channel = 9; // GM drum channel
            sequences.push(perc);
        }

        sequences
    }

    fn name(&self) -> &'static str {
        "show"
    }

    fn description(&self) -> &'static str {
        "Broadway/musical theater style with strings, brass fanfares, and singable melodies"
    }
}

/// Generate lush string pad with swells
fn generate_string_pad(
    config: &PresetConfig,
    variation: &PresetVariation,
    beats: f64,
    tempo: u16,
    instrument: u8,
    rng: &mut impl Rng,
) -> NoteSequence {
    let chord = config.key.chord_tones();
    let mut notes = Vec::new();

    // Style determines swell behavior
    let style = variation.pick_style(1, 4);

    // Phrase length for swells
    let phrase_len = 4.0; // 4-beat phrases

    let mut t = 0.0;
    while t < beats {
        let phrase_pos = t / phrase_len;
        let phrase_frac = phrase_pos.fract();

        // Dynamic swell within phrase
        let swell_velocity: u8 = match style {
            0 => {
                // Crescendo within each phrase
                (55.0 + phrase_frac * 35.0) as u8
            }
            1 => {
                // Arch shape (up then down)
                let arch = if phrase_frac < 0.5 {
                    phrase_frac * 2.0
                } else {
                    2.0 - phrase_frac * 2.0
                };
                (50.0 + arch * 40.0) as u8
            }
            2 => {
                // Sustained forte
                75 + rng.gen_range(0..15)
            }
            _ => {
                // Gentle wave
                (60.0 + (phrase_frac * std::f64::consts::PI * 2.0).sin() * 15.0) as u8
            }
        };

        let velocity = variation.adjust_velocity(swell_velocity);

        // Play chord tones spread across registers
        for (i, &base_pitch) in chord.iter().enumerate() {
            // Spread voicing: bass, tenor, alto, soprano
            let octave_offset: i8 = match i {
                0 => -12, // Bass
                1 => 0,   // Tenor
                2 => 0,   // Alto
                _ => 12,  // Soprano (if 4th note)
            };
            let pitch = ((base_pitch as i8 + octave_offset).max(36) as u8).min(84);

            // Slight timing variation for richness
            let time_offset = rng.gen_range(0.0..0.03);
            notes.push(Note::new(pitch, 0.95, velocity, t + time_offset));
        }

        t += 1.0; // Whole note sustains, refresh each beat for MIDI
    }

    NoteSequence::new(notes, instrument, tempo)
}

/// Generate Broadway-style piano accompaniment
fn generate_piano_accomp(
    config: &PresetConfig,
    variation: &PresetVariation,
    beats: f64,
    tempo: u16,
    instrument: u8,
    pattern_idx: usize,
    rng: &mut impl Rng,
) -> NoteSequence {
    let chord = config.key.chord_tones();
    let root = config.key.root();
    let mut notes = Vec::new();

    let pattern = ACCOMP_PATTERNS[pattern_idx];
    let pattern_len = 4.0;

    // Voicing style
    let voicing = variation.pick_style(2, 4);

    let mut t = 0.0;
    while t < beats {
        for (i, &offset) in pattern.iter().enumerate() {
            let pos = t + offset;
            if pos >= beats {
                break;
            }

            // Velocity with accent on beat 1
            let base_vel = if offset == 0.0 {
                80 + rng.gen_range(0..15)
            } else if offset.fract() == 0.0 {
                70 + rng.gen_range(0..10)
            } else {
                60 + rng.gen_range(0..15)
            };
            let velocity = variation.adjust_velocity(base_vel);

            // Duration varies by position
            let duration = if offset.fract() == 0.0 {
                rng.gen_range(0.3..0.5)
            } else {
                rng.gen_range(0.15..0.3)
            };

            match voicing {
                0 => {
                    // Arpeggiated up
                    let note_idx = i % chord.len();
                    let pitch = chord[note_idx];
                    notes.push(Note::new(pitch, duration, velocity, pos));
                }
                1 => {
                    // Arpeggiated with octave jumps
                    let note_idx = i % chord.len();
                    let octave = if i % 4 < 2 { 0i8 } else { 12 };
                    let pitch = ((chord[note_idx] as i8 + octave) as u8).clamp(48, 84);
                    notes.push(Note::new(pitch, duration, velocity, pos));
                }
                2 => {
                    // Block chords on strong beats, arpeggios on weak
                    if offset.fract() == 0.0 {
                        // Block chord
                        for &pitch in chord.iter() {
                            notes.push(Note::new(pitch, duration, velocity, pos));
                        }
                    } else {
                        // Single note
                        let note_idx = i % chord.len();
                        notes.push(Note::new(chord[note_idx], duration, velocity, pos));
                    }
                }
                _ => {
                    // Walking bass + chord stabs
                    if offset.fract() == 0.0 {
                        // Bass note
                        notes.push(Note::new(root.saturating_sub(12), duration, velocity, pos));
                    } else if (offset * 2.0).fract() == 0.0 {
                        // Chord stab
                        for &pitch in chord.iter().skip(1) {
                            notes.push(Note::new(pitch, duration * 0.5, velocity.saturating_sub(10), pos));
                        }
                    }
                }
            }
        }
        t += pattern_len;
    }

    NoteSequence::new(notes, instrument, tempo)
}

/// Generate singable melody line (vocal range)
fn generate_melody_line(
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

    // Vocal range: roughly C4 to C6 (MIDI 60-84)
    let vocal_base = 60u8;
    let vocal_range = 24u8;

    // Phrase structure
    let phrase_length = variation.phrase_length.clamp(4, 8) as usize;
    let contour = variation.get_contour(phrase_length);

    // Melody style
    let style = variation.pick_style(3, 4);

    // Start after intro
    let start_time = if beats > 4.0 { 2.0 } else { 0.5 };

    let mut scale_idx = (variation.scale_offset as usize) % scale.len();
    let mut current_pitch: u8;

    let mut t = start_time;
    let mut phrase_pos = 0;
    let mut measure_pos: f64 = 0.0;

    while t < beats - 0.5 {
        // Occasional rests for breathing
        if variation.should_rest(rng) && phrase_pos > 0 {
            t += 0.5;
            phrase_pos += 1;
            continue;
        }

        // Calculate pitch from scale
        let interval = scale[scale_idx % scale.len()];
        let base = root as i8 + interval as i8;

        // Keep in vocal range
        let mut pitch = base;
        while pitch < vocal_base as i8 {
            pitch += 12;
        }
        while pitch > (vocal_base + vocal_range) as i8 {
            pitch -= 12;
        }
        current_pitch = pitch as u8;

        // Duration based on style and position
        let duration = match style {
            0 => {
                // Legato ballad
                if phrase_pos == phrase_length - 1 {
                    rng.gen_range(1.0..2.0) // Long held note at phrase end
                } else {
                    rng.gen_range(0.5..1.0)
                }
            }
            1 => {
                // Rhythmic show tune
                if measure_pos.fract() == 0.0 {
                    rng.gen_range(0.4..0.7)
                } else {
                    rng.gen_range(0.2..0.4)
                }
            }
            2 => {
                // Mixed - some long, some short
                if rng.gen_bool(0.3) {
                    rng.gen_range(0.75..1.5)
                } else {
                    rng.gen_range(0.25..0.5)
                }
            }
            _ => {
                // Swing feel
                rng.gen_range(0.3..0.6)
            }
        };

        // Velocity with expressive contour
        let phrase_intensity = (phrase_pos as f64 / phrase_length as f64 * 20.0) as u8;
        let base_vel = 70 + phrase_intensity + rng.gen_range(0..10);
        let velocity = variation.adjust_velocity(base_vel);

        notes.push(Note::new(current_pitch, duration, velocity, t));

        // Move through scale based on contour
        let direction = contour[phrase_pos % contour.len()];
        let step = variation.get_interval(rng) as usize;
        match direction {
            1 => scale_idx = (scale_idx + step) % scale.len(),
            -1 => scale_idx = scale_idx.saturating_sub(step),
            _ => {} // Hold
        }

        // Time step
        let time_step = match style {
            0 => duration.min(1.0),
            1 => 0.5,
            2 => duration.min(0.75),
            _ => rng.gen_range(0.3..0.6),
        };

        t += time_step;
        measure_pos += time_step;
        phrase_pos += 1;

        // Reset phrase
        if phrase_pos >= phrase_length {
            phrase_pos = 0;
            t += 0.5; // Brief pause between phrases
        }
    }

    NoteSequence::new(notes, instrument, tempo)
}

/// Generate brass fanfares and accents
fn generate_brass_accents(
    config: &PresetConfig,
    variation: &PresetVariation,
    beats: f64,
    tempo: u16,
    instrument: u8,
    rng: &mut impl Rng,
) -> NoteSequence {
    let chord = config.key.chord_tones();
    let root = config.key.root();
    let fifth = root + 7;
    let mut notes = Vec::new();

    // Brass style
    let style = variation.pick_style(4, 4);

    // Fanfare notes (root and fifth are classic)
    let fanfare_notes = [root, fifth, root + 12, fifth + 12];

    match style {
        0 => {
            // Opening fanfare
            if beats >= 2.0 {
                let fanfare_times = [0.0, 0.25, 0.5, 1.0];
                for (i, &t) in fanfare_times.iter().enumerate() {
                    if t < beats {
                        let pitch = fanfare_notes[i % fanfare_notes.len()];
                        let vel = variation.adjust_velocity(85 + rng.gen_range(0..15));
                        let dur = if i == fanfare_times.len() - 1 { 1.0 } else { 0.25 };
                        notes.push(Note::new(pitch, dur, vel, t));
                    }
                }
            }
            // Punctuation at phrase ends
            let mut t = 4.0;
            while t < beats {
                let pitch = fanfare_notes[rng.gen_range(0..fanfare_notes.len())];
                let vel = variation.adjust_velocity(80 + rng.gen_range(0..20));
                notes.push(Note::new(pitch, 0.5, vel, t - 0.5));
                t += 4.0;
            }
        }
        1 => {
            // Sustained brass swells
            let mut t = 2.0;
            while t < beats - 2.0 {
                // Brass chord swell
                for (i, &pitch) in chord.iter().enumerate() {
                    let octave = if i == 0 { 0i8 } else { 12 };
                    let p = ((pitch as i8 + octave) as u8).clamp(48, 84);
                    let vel = variation.adjust_velocity(60 + rng.gen_range(0..20));
                    notes.push(Note::new(p, 2.0, vel, t));
                }
                t += 4.0;
            }
        }
        2 => {
            // Staccato hits on strong beats
            let mut t = 0.0;
            while t < beats {
                if rng.gen_bool(0.6) {
                    let pitch = fanfare_notes[rng.gen_range(0..2)]; // Root or fifth
                    let vel = variation.adjust_velocity(90 + rng.gen_range(0..10));
                    notes.push(Note::new(pitch, 0.15, vel, t));
                }
                t += 2.0; // Every 2 beats
            }
        }
        _ => {
            // Counter melody (horn line)
            let mut t = 1.0;
            let mut note_idx = 0;
            while t < beats - 1.0 {
                if !variation.should_rest(rng) {
                    let pitch = fanfare_notes[note_idx % fanfare_notes.len()];
                    let vel = variation.adjust_velocity(70 + rng.gen_range(0..15));
                    notes.push(Note::new(pitch, 0.5, vel, t));
                    note_idx += 1;
                }
                t += 1.0;
            }
        }
    }

    NoteSequence::new(notes, instrument, tempo)
}

/// Generate theatrical percussion (timpani, cymbals)
fn generate_theatrical_percussion(
    _config: &PresetConfig,
    variation: &PresetVariation,
    beats: f64,
    tempo: u16,
    rng: &mut impl Rng,
) -> NoteSequence {
    let mut notes = Vec::new();

    // GM drum notes
    const CRASH_CYMBAL: u8 = 49;
    const RIDE_CYMBAL: u8 = 51;
    const TIMPANI_LOW: u8 = 41;  // Low floor tom as timpani substitute
    const TIMPANI_HIGH: u8 = 43; // High floor tom

    let style = variation.pick_style(5, 3);

    match style {
        0 => {
            // Timpani rolls and crashes
            // Opening crash
            if beats >= 1.0 {
                notes.push(Note::new(CRASH_CYMBAL, 0.5, variation.adjust_velocity(100), 0.0));
            }

            // Timpani on downbeats
            let mut t = 0.0;
            while t < beats {
                let vel = variation.adjust_velocity(80 + rng.gen_range(0..20));
                notes.push(Note::new(TIMPANI_LOW, 0.3, vel, t));
                t += 4.0;
            }

            // Crash at climax points
            let climax_time = beats * 0.75;
            if climax_time > 2.0 {
                notes.push(Note::new(CRASH_CYMBAL, 0.5, variation.adjust_velocity(110), climax_time));
            }
        }
        1 => {
            // Ride cymbal pattern
            let mut t = 0.0;
            while t < beats {
                let vel = if t.fract() == 0.0 {
                    variation.adjust_velocity(70 + rng.gen_range(0..15))
                } else {
                    variation.adjust_velocity(55 + rng.gen_range(0..10))
                };
                notes.push(Note::new(RIDE_CYMBAL, 0.1, vel, t));
                t += 0.5;
            }
        }
        _ => {
            // Sparse dramatic hits
            let hit_times = [0.0, beats * 0.5, beats - 1.0];
            for &t in hit_times.iter() {
                if t >= 0.0 && t < beats {
                    notes.push(Note::new(TIMPANI_HIGH, 0.3, variation.adjust_velocity(90), t));
                    if rng.gen_bool(0.5) {
                        notes.push(Note::new(CRASH_CYMBAL, 0.5, variation.adjust_velocity(85), t));
                    }
                }
            }
        }
    }

    NoteSequence::new(notes, 0, tempo) // Channel 9 ignores instrument
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_show_generates_sequences() {
        let config = PresetConfig {
            key: super::super::Key::Bb,
            tempo: 120,
            duration_secs: 5.0,
            ..Default::default()
        };
        let sequences = ShowPreset.generate(&config);
        assert!(!sequences.is_empty(), "Should generate at least one sequence");
        assert!(sequences.len() >= 2, "Should have multiple layers");
    }

    #[test]
    fn test_show_different_seeds_vary() {
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

        let seq1 = ShowPreset.generate(&config1);
        let seq2 = ShowPreset.generate(&config2);

        // Check for differences
        let mut diffs = 0;
        if seq1.len() != seq2.len() {
            diffs += 5;
        }
        if !seq1.is_empty() && !seq2.is_empty() {
            if seq1[0].instrument != seq2[0].instrument {
                diffs += 3;
            }
            if seq1[0].notes.len() != seq2[0].notes.len() {
                diffs += 1;
            }
        }

        assert!(diffs >= 1, "Adjacent seeds should produce different output");
    }

    #[test]
    fn test_show_vocal_range_melody() {
        let config = PresetConfig {
            seed: 100,
            duration_secs: 8.0,
            intensity: 80,
            ..Default::default()
        };
        let sequences = ShowPreset.generate(&config);

        // Find melody layer (channel 2)
        let melody = sequences.iter().find(|s| s.channel == 2);
        if let Some(m) = melody {
            // Check all notes are in roughly vocal range (MIDI 48-96)
            for note in &m.notes {
                assert!(note.pitch >= 48 && note.pitch <= 96,
                    "Melody note {} outside vocal range", note.pitch);
            }
        }
    }

    #[test]
    fn test_show_instruments_vary() {
        let instruments: Vec<u8> = (1..=10)
            .map(|seed| {
                let config = PresetConfig {
                    seed,
                    duration_secs: 3.0,
                    ..Default::default()
                };
                let seqs = ShowPreset.generate(&config);
                seqs[0].instrument
            })
            .collect();

        let unique: std::collections::HashSet<_> = instruments.iter().collect();
        assert!(unique.len() > 1, "Instruments should vary across seeds");
    }
}
