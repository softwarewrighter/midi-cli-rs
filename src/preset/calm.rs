//! Calm mood preset
//!
//! Characteristics: Major/modal, slow, sustained pads, gentle arpeggios

use super::{MoodGenerator, PresetConfig, create_rng};
use crate::midi::{Note, NoteSequence};
use rand::Rng;

/// Calm mood generator
pub struct CalmPreset;

impl MoodGenerator for CalmPreset {
    fn generate(&self, config: &PresetConfig) -> Vec<NoteSequence> {
        let mut rng = create_rng(config.seed);
        let mut sequences = Vec::new();

        let beats = config.duration_secs * config.tempo as f64 / 60.0;

        // Layer 1: Sustained pad chord
        sequences.push(generate_pad_chord(config, beats));

        // Layer 2: Gentle arpeggio
        sequences.push(generate_arpeggio(config, beats, &mut rng));

        sequences
    }

    fn name(&self) -> &'static str {
        "calm"
    }

    fn description(&self) -> &'static str {
        "Peaceful, serene mood with sustained pads and gentle arpeggios"
    }
}

/// Generate sustained pad chord (major 7th for lush sound)
fn generate_pad_chord(config: &PresetConfig, beats: f64) -> NoteSequence {
    let root = config.key.root();

    // Major 7th chord for calm, lush sound
    // Even for minor keys, we add the major 7th for warmth
    let notes = if config.key.is_minor() {
        // Minor add9 chord: root, minor 3rd, fifth, 9th
        vec![
            Note::new(root - 12, beats, 40, 0.0), // Root, low
            Note::new(root + 3, beats, 35, 0.0),  // Minor 3rd
            Note::new(root + 7, beats, 35, 0.0),  // Fifth
            Note::new(root + 14, beats, 30, 0.0), // 9th
        ]
    } else {
        // Major 7th chord: root, major 3rd, fifth, major 7th
        vec![
            Note::new(root - 12, beats, 40, 0.0), // Root, low
            Note::new(root + 4, beats, 35, 0.0),  // Major 3rd
            Note::new(root + 7, beats, 35, 0.0),  // Fifth
            Note::new(root + 11, beats, 30, 0.0), // Major 7th
        ]
    };

    // Pad warm (89)
    NoteSequence::new(notes, 89, config.tempo)
}

/// Generate gentle arpeggio
fn generate_arpeggio(config: &PresetConfig, beats: f64, rng: &mut impl Rng) -> NoteSequence {
    let chord = config.key.chord_tones();
    let mut notes = Vec::new();

    // Slow arpeggio with some variation
    let note_duration = 0.75;
    let note_spacing = 1.0;

    let mut t = 0.25; // Start slightly delayed
    let mut chord_index = 0;
    let mut ascending = true;

    while t < beats - 0.5 {
        let pitch = chord[chord_index] + 12; // One octave up from middle
        let velocity = 45 + rng.gen_range(0..15);

        notes.push(Note::new(pitch, note_duration, velocity, t));

        // Move through chord tones
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

        t += note_spacing;
    }

    // Harp (46) for gentle plucked sound
    NoteSequence::new(notes, 46, config.tempo)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calm_generates_sequences() {
        let config = PresetConfig {
            tempo: 70, // Slow tempo
            ..Default::default()
        };
        let sequences = CalmPreset.generate(&config);
        assert_eq!(sequences.len(), 2); // Pad + arpeggio
    }

    #[test]
    fn test_calm_pad_is_sustained() {
        let config = PresetConfig {
            duration_secs: 5.0,
            tempo: 60,
            ..Default::default()
        };
        let sequences = CalmPreset.generate(&config);

        // Pad notes should span full duration
        let pad = &sequences[0];
        for note in &pad.notes {
            assert!(note.duration >= 4.0, "Pad notes should be sustained");
        }
    }
}
