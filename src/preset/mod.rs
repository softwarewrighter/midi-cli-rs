//! Mood preset generators for algorithmic music composition
//!
//! Each preset generates note sequences that evoke a specific mood,
//! suitable for video intro/outro stingers.

mod ambient;
mod calm;
mod eerie;
mod jazz;
mod suspense;
mod upbeat;

pub use ambient::AmbientPreset;
pub use calm::CalmPreset;
pub use eerie::EeriePreset;
pub use jazz::JazzPreset;
pub use suspense::SuspensePreset;
pub use upbeat::UpbeatPreset;

use crate::midi::sequence::NoteSequence;
use rand::Rng;
use rand::SeedableRng;
use rand::rngs::StdRng;

/// Centralized variation parameters generated from seed
/// This ensures different seeds produce noticeably different outputs
#[derive(Debug, Clone)]
pub struct PresetVariation {
    /// Tempo multiplier (0.85 to 1.15 = ±15%)
    pub tempo_factor: f64,
    /// Layer inclusion probabilities (0.0-1.0)
    pub layer_probs: [f64; 6],
    /// Instrument selection indices (0-255, mod by available instruments)
    pub instrument_indices: [u8; 6],
    /// Style variations (0-255)
    pub style_choices: [u8; 6],
    /// Density factor (0.5 to 1.5)
    pub density_factor: f64,
    /// Velocity base offset (-20 to +20)
    pub velocity_offset: i8,
    /// Note count multiplier (0.7 to 1.5)
    pub note_count_factor: f64,
}

impl PresetVariation {
    /// Generate variation parameters from seed
    pub fn from_seed(seed: u64) -> Self {
        let mut rng = StdRng::seed_from_u64(seed);

        Self {
            tempo_factor: 1.0 + (rng.gen_range(-15..=15) as f64 / 100.0),
            layer_probs: [
                rng.gen_range(0.3..1.0),
                rng.gen_range(0.3..1.0),
                rng.gen_range(0.2..0.9),
                rng.gen_range(0.2..0.8),
                rng.gen_range(0.1..0.7),
                rng.gen_range(0.1..0.6),
            ],
            instrument_indices: [
                rng.gen_range(0..=255),
                rng.gen_range(0..=255),
                rng.gen_range(0..=255),
                rng.gen_range(0..=255),
                rng.gen_range(0..=255),
                rng.gen_range(0..=255),
            ],
            style_choices: [
                rng.gen_range(0..=255),
                rng.gen_range(0..=255),
                rng.gen_range(0..=255),
                rng.gen_range(0..=255),
                rng.gen_range(0..=255),
                rng.gen_range(0..=255),
            ],
            density_factor: rng.gen_range(0.6..1.4),
            velocity_offset: rng.gen_range(-15..=15),
            note_count_factor: rng.gen_range(0.7..1.4),
        }
    }

    /// Get effective tempo given base tempo
    pub fn effective_tempo(&self, base_tempo: u16) -> u16 {
        ((base_tempo as f64 * self.tempo_factor) as u16).clamp(40, 200)
    }

    /// Check if a layer should be included (combines variation prob with intensity)
    pub fn include_layer(&self, layer_idx: usize, intensity: u8, base_threshold: u8) -> bool {
        let var_prob = self.layer_probs.get(layer_idx).copied().unwrap_or(0.5);
        let intensity_factor = intensity as f64 / 100.0;
        let threshold = base_threshold as f64 / 100.0;

        // Layer included if: random variation says yes AND intensity is above adjusted threshold
        var_prob > (1.0 - intensity_factor) && intensity as f64 / 100.0 >= (threshold * (1.0 - var_prob * 0.3))
    }

    /// Get instrument from a list using seeded index
    pub fn pick_instrument(&self, layer_idx: usize, instruments: &[u8]) -> u8 {
        let idx = self.instrument_indices.get(layer_idx).copied().unwrap_or(0) as usize;
        instruments[idx % instruments.len()]
    }

    /// Get style choice (0-255) for a layer, mod by num_styles for actual choice
    pub fn pick_style(&self, layer_idx: usize, num_styles: usize) -> usize {
        let choice = self.style_choices.get(layer_idx).copied().unwrap_or(0) as usize;
        choice % num_styles
    }

    /// Adjust velocity with variation offset
    pub fn adjust_velocity(&self, base_vel: u8) -> u8 {
        (base_vel as i16 + self.velocity_offset as i16).clamp(1, 127) as u8
    }
}

/// Musical key for preset generation
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Key {
    C,
    Cm,
    D,
    Dm,
    Eb,
    Ebm,
    E,
    Em,
    F,
    Fm,
    G,
    Gm,
    A,
    Am,
    Bb,
    Bbm,
    B,
    Bm,
}

impl Key {
    /// Parse key from string (e.g., "Am", "C", "F#m", "Bb", "Eb")
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "c" => Some(Key::C),
            "cm" => Some(Key::Cm),
            "d" => Some(Key::D),
            "dm" => Some(Key::Dm),
            "eb" | "d#" => Some(Key::Eb),
            "ebm" | "d#m" => Some(Key::Ebm),
            "e" => Some(Key::E),
            "em" => Some(Key::Em),
            "f" => Some(Key::F),
            "fm" => Some(Key::Fm),
            "g" => Some(Key::G),
            "gm" => Some(Key::Gm),
            "a" => Some(Key::A),
            "am" => Some(Key::Am),
            "bb" | "a#" => Some(Key::Bb),
            "bbm" | "a#m" => Some(Key::Bbm),
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
            Key::Eb | Key::Ebm => 63,
            Key::E | Key::Em => 64,
            Key::F | Key::Fm => 65,
            Key::G | Key::Gm => 67,
            Key::A | Key::Am => 69,
            Key::Bb | Key::Bbm => 70,
            Key::B | Key::Bm => 71,
        }
    }

    /// Check if this is a minor key
    pub fn is_minor(&self) -> bool {
        matches!(
            self,
            Key::Cm | Key::Dm | Key::Ebm | Key::Em | Key::Fm | Key::Gm | Key::Am | Key::Bbm | Key::Bm
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
    Jazz,
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
            "jazz" | "jazzy" | "swing" => Some(Mood::Jazz),
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
            Mood::Jazz => Key::F, // Common jazz key
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
        Mood::Jazz => JazzPreset.generate(config),
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
            Mood::Jazz,
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

    #[test]
    fn test_variation_from_seed_is_deterministic() {
        let var1 = PresetVariation::from_seed(42);
        let var2 = PresetVariation::from_seed(42);

        assert_eq!(var1.tempo_factor, var2.tempo_factor);
        assert_eq!(var1.instrument_indices, var2.instrument_indices);
        assert_eq!(var1.style_choices, var2.style_choices);
    }

    #[test]
    fn test_variation_differs_by_seed() {
        let var42 = PresetVariation::from_seed(42);
        let var43 = PresetVariation::from_seed(43);

        // Adjacent seeds should produce different variations
        let mut differences = 0;
        if (var42.tempo_factor - var43.tempo_factor).abs() > 0.001 {
            differences += 1;
        }
        if var42.instrument_indices != var43.instrument_indices {
            differences += 1;
        }
        if var42.style_choices != var43.style_choices {
            differences += 1;
        }
        if (var42.density_factor - var43.density_factor).abs() > 0.01 {
            differences += 1;
        }

        assert!(
            differences >= 2,
            "Seeds 42 and 43 should produce different variations"
        );
    }

    #[test]
    fn test_effective_tempo_clamped() {
        let var = PresetVariation::from_seed(999);
        let tempo = var.effective_tempo(100);
        assert!(tempo >= 40 && tempo <= 200);
    }

    #[test]
    fn test_pick_instrument_wraps() {
        let var = PresetVariation::from_seed(42);
        let instruments = [0u8, 1, 2];
        let picked = var.pick_instrument(0, &instruments);
        assert!(instruments.contains(&picked));
    }
}
