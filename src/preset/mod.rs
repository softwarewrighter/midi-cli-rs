//! Mood preset generators for algorithmic music composition
//!
//! Each preset generates note sequences that evoke a specific mood,
//! suitable for video intro/outro stingers.

mod ambient;
mod calm;
mod eerie;
mod suspense;
mod upbeat;

pub use ambient::AmbientPreset;
pub use calm::CalmPreset;
pub use eerie::EeriePreset;
pub use suspense::SuspensePreset;
pub use upbeat::UpbeatPreset;

use crate::midi::sequence::NoteSequence;
use rand::SeedableRng;
use rand::rngs::StdRng;

/// Musical key for preset generation
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Key {
    C,
    Cm,
    D,
    Dm,
    E,
    Em,
    F,
    Fm,
    G,
    Gm,
    A,
    Am,
    B,
    Bm,
}

impl Key {
    /// Parse key from string (e.g., "Am", "C", "F#m")
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "c" => Some(Key::C),
            "cm" => Some(Key::Cm),
            "d" => Some(Key::D),
            "dm" => Some(Key::Dm),
            "e" => Some(Key::E),
            "em" => Some(Key::Em),
            "f" => Some(Key::F),
            "fm" => Some(Key::Fm),
            "g" => Some(Key::G),
            "gm" => Some(Key::Gm),
            "a" => Some(Key::A),
            "am" => Some(Key::Am),
            "b" => Some(Key::B),
            "bm" => Some(Key::Bm),
            _ => None,
        }
    }

    /// Get the root note MIDI number (octave 4)
    pub fn root(&self) -> u8 {
        match self {
            Key::C | Key::Cm => 60,
            Key::D | Key::Dm => 62,
            Key::E | Key::Em => 64,
            Key::F | Key::Fm => 65,
            Key::G | Key::Gm => 67,
            Key::A | Key::Am => 69,
            Key::B | Key::Bm => 71,
        }
    }

    /// Check if this is a minor key
    pub fn is_minor(&self) -> bool {
        matches!(
            self,
            Key::Cm | Key::Dm | Key::Em | Key::Fm | Key::Gm | Key::Am | Key::Bm
        )
    }

    /// Get scale intervals (semitones from root)
    pub fn scale_intervals(&self) -> &'static [u8] {
        if self.is_minor() {
            // Natural minor scale
            &[0, 2, 3, 5, 7, 8, 10]
        } else {
            // Major scale
            &[0, 2, 4, 5, 7, 9, 11]
        }
    }

    /// Get chord tones (root, third, fifth)
    pub fn chord_tones(&self) -> [u8; 3] {
        let root = self.root();
        if self.is_minor() {
            [root, root + 3, root + 7] // Minor chord
        } else {
            [root, root + 4, root + 7] // Major chord
        }
    }
}

/// Available mood presets
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mood {
    Suspense,
    Eerie,
    Upbeat,
    Calm,
    Ambient,
}

impl Mood {
    /// Parse mood from string
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "suspense" | "tense" | "tension" => Some(Mood::Suspense),
            "eerie" | "creepy" | "spooky" => Some(Mood::Eerie),
            "upbeat" | "happy" | "energetic" => Some(Mood::Upbeat),
            "calm" | "peaceful" | "serene" => Some(Mood::Calm),
            "ambient" | "atmospheric" | "drone" => Some(Mood::Ambient),
            _ => None,
        }
    }

    /// Get default key for this mood
    pub fn default_key(&self) -> Key {
        match self {
            Mood::Suspense => Key::Am,
            Mood::Eerie => Key::Dm,
            Mood::Upbeat => Key::C,
            Mood::Calm => Key::G,
            Mood::Ambient => Key::Em,
        }
    }
}

/// Configuration for preset generation
#[derive(Debug, Clone)]
pub struct PresetConfig {
    /// Duration in seconds
    pub duration_secs: f64,
    /// Musical key
    pub key: Key,
    /// Intensity level (0-100)
    pub intensity: u8,
    /// Random seed for reproducibility
    pub seed: u64,
    /// Tempo in BPM
    pub tempo: u16,
}

impl Default for PresetConfig {
    fn default() -> Self {
        Self {
            duration_secs: 5.0,
            key: Key::Am,
            intensity: 50,
            seed: 42,
            tempo: 90,
        }
    }
}

/// Trait for mood preset generators
pub trait MoodGenerator {
    /// Generate note sequences for this mood
    fn generate(&self, config: &PresetConfig) -> Vec<NoteSequence>;

    /// Get the mood name
    fn name(&self) -> &'static str;

    /// Get a description of this mood
    fn description(&self) -> &'static str;
}

/// Generate sequences for a given mood
pub fn generate_mood(mood: Mood, config: &PresetConfig) -> Vec<NoteSequence> {
    match mood {
        Mood::Suspense => SuspensePreset.generate(config),
        Mood::Eerie => EeriePreset.generate(config),
        Mood::Upbeat => UpbeatPreset.generate(config),
        Mood::Calm => CalmPreset.generate(config),
        Mood::Ambient => AmbientPreset.generate(config),
    }
}

/// Create a seeded RNG for reproducible generation
pub fn create_rng(seed: u64) -> StdRng {
    StdRng::seed_from_u64(seed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_parse() {
        assert_eq!(Key::parse("Am"), Some(Key::Am));
        assert_eq!(Key::parse("C"), Some(Key::C));
        assert_eq!(Key::parse("dm"), Some(Key::Dm));
        assert_eq!(Key::parse("invalid"), None);
    }

    #[test]
    fn test_key_root() {
        assert_eq!(Key::C.root(), 60);
        assert_eq!(Key::A.root(), 69);
        assert_eq!(Key::Am.root(), 69);
    }

    #[test]
    fn test_key_is_minor() {
        assert!(Key::Am.is_minor());
        assert!(Key::Dm.is_minor());
        assert!(!Key::C.is_minor());
        assert!(!Key::G.is_minor());
    }

    #[test]
    fn test_mood_parse() {
        assert_eq!(Mood::parse("suspense"), Some(Mood::Suspense));
        assert_eq!(Mood::parse("UPBEAT"), Some(Mood::Upbeat));
        assert_eq!(Mood::parse("creepy"), Some(Mood::Eerie));
        assert_eq!(Mood::parse("invalid"), None);
    }

    #[test]
    fn test_mood_default_key() {
        assert_eq!(Mood::Suspense.default_key(), Key::Am);
        assert_eq!(Mood::Upbeat.default_key(), Key::C);
    }

    #[test]
    fn test_generate_mood_produces_sequences() {
        let config = PresetConfig::default();
        for mood in [
            Mood::Suspense,
            Mood::Eerie,
            Mood::Upbeat,
            Mood::Calm,
            Mood::Ambient,
        ] {
            let sequences = generate_mood(mood, &config);
            assert!(
                !sequences.is_empty(),
                "Mood {:?} should produce sequences",
                mood
            );
        }
    }

    #[test]
    fn test_deterministic_generation() {
        let config = PresetConfig {
            seed: 12345,
            ..Default::default()
        };

        let seq1 = generate_mood(Mood::Suspense, &config);
        let seq2 = generate_mood(Mood::Suspense, &config);

        assert_eq!(seq1.len(), seq2.len());
        for (s1, s2) in seq1.iter().zip(seq2.iter()) {
            assert_eq!(s1.notes.len(), s2.notes.len());
            for (n1, n2) in s1.notes.iter().zip(s2.notes.iter()) {
                assert_eq!(n1.pitch, n2.pitch);
                assert_eq!(n1.duration, n2.duration);
            }
        }
    }
}
