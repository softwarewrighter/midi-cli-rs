//! Jazz mood preset
//!
//! Characteristics: Swing feel, walking bass, piano comping, ride cymbal

use super::{create_rng, MoodGenerator, PresetConfig};
use crate::midi::{Note, NoteSequence};
use rand::Rng;

/// Jazz mood generator - nightclub trio style
pub struct JazzPreset;

impl MoodGenerator for JazzPreset {
    fn generate(&self, config: &PresetConfig) -> Vec<NoteSequence> {
        let mut rng = create_rng(config.seed);
        let mut sequences = Vec::new();

        let beats = config.duration_secs * config.tempo as f64 / 60.0;

        // Layer 1: Walking bass
        sequences.push(generate_walking_bass(config, beats, &mut rng));

        // Layer 2: Piano comping
        sequences.push(generate_piano_comping(config, beats, &mut rng));

        // Layer 3: Ride cymbal pattern (if intensity > 30)
        if config.intensity > 30 {
            sequences.push(generate_ride_pattern(config, beats, &mut rng));
        }

        // Layer 4: Hi-hat on 2 and 4 (if intensity > 50)
        if config.intensity > 50 {
            sequences.push(generate_hihat_accents(config, beats));
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

/// Generate walking bass line with chromatic approach notes
fn generate_walking_bass(config: &PresetConfig, beats: f64, rng: &mut impl Rng) -> NoteSequence {
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

    while t < beats {
        // Choose next target (chord tone)
        let target = chord_tones[rng.gen_range(0..chord_tones.len())];

        // Sometimes use chromatic approach
        let pitch = if rng.gen_bool(0.3) && t > 0.0 {
            // Chromatic approach from below or above
            if rng.gen_bool(0.5) {
                target.saturating_sub(1)
            } else {
                target.saturating_add(1).min(127)
            }
        } else if rng.gen_bool(0.2) {
            // Passing tone (scale-based)
            let intervals = config.key.scale_intervals();
            let interval = intervals[rng.gen_range(0..intervals.len())];
            (bass_root + interval).min(bass_root + 12)
        } else {
            target
        };

        // Vary velocity for groove
        let vel_base = 75 + (config.intensity as i32 / 5) as u8;
        let velocity = vel_base + rng.gen_range(0..15);

        // Quarter note duration with slight swing feel
        let duration = 0.9; // Slightly detached for walking feel

        notes.push(Note::new(pitch, duration, velocity, t));
        t += 1.0; // One beat per note (walking)
    }

    // Acoustic bass (32) or Electric bass (33)
    NoteSequence::new(notes, 32, config.tempo)
}

/// Generate sparse jazz piano comping with shell voicings
fn generate_piano_comping(config: &PresetConfig, beats: f64, rng: &mut impl Rng) -> NoteSequence {
    let root = config.key.root();
    let mut notes = Vec::new();

    // Jazz voicings (rootless or shell voicings)
    let voicings: Vec<Vec<i8>> = if config.key.is_minor() {
        vec![
            vec![3, 7, 10],      // m7 (3rd, 5th, 7th)
            vec![3, 10, 14],     // m7 spread
            vec![10, 14, 17],    // m9 upper
            vec![-2, 3, 7],      // with 9th below
        ]
    } else {
        vec![
            vec![4, 7, 11],      // maj7 (3rd, 5th, 7th)
            vec![4, 11, 14],     // maj7 spread
            vec![11, 14, 16],    // maj9 upper
            vec![-1, 4, 7],      // with 7th below
        ]
    };

    let mut t = 0.0;

    while t < beats - 1.0 {
        // Comping rhythm: sparse, syncopated
        // Skip some beats for authentic jazz feel
        if rng.gen_bool(0.4) {
            t += 1.0;
            continue;
        }

        // Choose voicing
        let voicing = &voicings[rng.gen_range(0..voicings.len())];

        // Swing offset: push some chords to the "and" of the beat
        let swing_offset = if rng.gen_bool(0.35) {
            0.33 // Swung eighth
        } else if rng.gen_bool(0.25) {
            0.5 // Straight eighth
        } else {
            0.0 // On beat
        };

        let chord_time = t + swing_offset;
        if chord_time >= beats {
            break;
        }

        // Duration varies (staccato to sustained)
        let duration = if rng.gen_bool(0.3) {
            0.25 // Stab
        } else if rng.gen_bool(0.4) {
            0.5  // Short
        } else {
            1.0  // Held
        };

        // Velocity for dynamics
        let vel_base = 50 + (config.intensity as i32 / 4) as u8;

        for (i, &interval) in voicing.iter().enumerate() {
            let pitch = (root as i8 + interval) as u8;
            // Softer for upper notes
            let vel = vel_base.saturating_sub(i as u8 * 3) + rng.gen_range(0..10);
            notes.push(Note::new(pitch, duration, vel, chord_time));
        }

        // Move forward 1-2 beats
        t += if rng.gen_bool(0.6) { 2.0 } else { 1.0 };
    }

    // Electric piano (4) for jazz feel, or Acoustic (0)
    let instrument = if config.intensity > 60 { 4 } else { 0 };
    NoteSequence::new(notes, instrument, config.tempo)
}

/// Generate ride cymbal pattern with swing
fn generate_ride_pattern(config: &PresetConfig, beats: f64, rng: &mut impl Rng) -> NoteSequence {
    let mut notes = Vec::new();

    // Ride cymbal on drum channel (MIDI note 51 = ride cymbal 1)
    let ride = 51u8;

    let mut t = 0.0;
    let swing_ratio = 0.67; // Swing feel (triplet-based)

    while t < beats {
        // Main ride hit on each beat
        let vel = 70 + rng.gen_range(0..20);
        notes.push(Note::new(ride, 0.3, vel, t));

        // Swung "and" hit
        let and_time = t + swing_ratio;
        if and_time < beats && rng.gen_bool(0.85) {
            let and_vel = 55 + rng.gen_range(0..15);
            notes.push(Note::new(ride, 0.3, and_vel, and_time));
        }

        t += 1.0;
    }

    // Channel 10 (program 0) is drums in GM
    // We use a melodic instrument slot but drums are on channel 10
    // For simplicity, using tubular bells (14) which has similar attack
    // In a real implementation, we'd need drum channel support
    NoteSequence::new(notes, 14, config.tempo)
}

/// Generate hi-hat accents on beats 2 and 4
fn generate_hihat_accents(config: &PresetConfig, beats: f64) -> NoteSequence {
    let mut notes = Vec::new();

    // Closed hi-hat accent sound approximation
    let hihat_pitch = 80u8; // High pitch for hi-hat simulation

    let mut t = 1.0; // Start on beat 2

    while t < beats {
        // Beat 2
        notes.push(Note::new(hihat_pitch, 0.1, 60, t));

        // Beat 4
        if t + 2.0 < beats {
            notes.push(Note::new(hihat_pitch, 0.1, 65, t + 2.0));
        }

        t += 4.0; // Every 4 beats (one bar)
    }

    // Woodblock (115) for hi-hat-like click
    NoteSequence::new(notes, 115, config.tempo)
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
        // Should have bass, piano, ride, hi-hat at intensity 70
        assert_eq!(sequences.len(), 4);
    }

    #[test]
    fn test_jazz_walking_bass_quarter_notes() {
        let config = PresetConfig {
            duration_secs: 4.0,
            tempo: 60, // 1 beat per second
            ..Default::default()
        };
        let sequences = JazzPreset.generate(&config);

        // First sequence is walking bass
        let bass = &sequences[0];
        // At 60 BPM, 4 seconds = 4 beats = 4 notes
        assert_eq!(bass.notes.len(), 4, "Walking bass should have one note per beat");
    }

    #[test]
    fn test_jazz_intensity_affects_layers() {
        let low = PresetConfig {
            intensity: 20,
            ..Default::default()
        };
        let high = PresetConfig {
            intensity: 80,
            ..Default::default()
        };

        let low_seq = JazzPreset.generate(&low);
        let high_seq = JazzPreset.generate(&high);

        assert!(high_seq.len() > low_seq.len(), "Higher intensity should add drum layers");
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
            assert!(note.pitch < 60, "Bass notes should be below middle C");
        }
    }
}
