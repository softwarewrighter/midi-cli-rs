# Mood Configuration Design

This document describes the serializable Rust data structures for configuring moods,
and how they map to a generalized UI editor.

## Design Principles

1. **Declarative over Imperative**: Moods are described as data, not code
2. **Composable**: Layers, patterns, and traits can be mixed and matched
3. **Seed-Invariant Character**: Seeds vary *within* a mood's parameter ranges
4. **Round-trip Safe**: TOML <-> Rust struct <-> UI edits are lossless

---

## Core Data Structures

### MoodDefinition (Top-Level)

```rust
/// Complete mood definition - the root serializable type
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MoodDefinition {
    /// Metadata
    pub meta: MoodMeta,
    /// Default parameters
    pub defaults: MoodDefaults,
    /// Harmonic configuration
    pub harmony: HarmonyConfig,
    /// Dynamic/velocity configuration
    pub dynamics: DynamicsConfig,
    /// Layer definitions (instruments, patterns, roles)
    pub layers: Vec<LayerDefinition>,
    /// Embellishment rules
    #[serde(default)]
    pub embellishments: EmbellishmentConfig,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MoodMeta {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub aliases: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,          // ["cinematic", "dark", "orchestral"]
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub version: String,
}
```

### Defaults Configuration

```rust
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MoodDefaults {
    /// Default musical key
    pub key: KeySpec,
    /// Tempo configuration
    pub tempo: TempoConfig,
    /// Intensity range (affects layer inclusion, velocity)
    pub intensity: RangeU8,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum KeySpec {
    /// Single key: "Dm", "G", "F#m"
    Single(String),
    /// Pool of keys to choose from (seed selects)
    Pool(Vec<String>),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TempoConfig {
    /// Default tempo in BPM
    pub default: u16,
    /// Allowed range for seed variation
    pub range: (u16, u16),
    /// Swing ratio (0.5 = straight, 0.67 = swing, 0.75 = hard swing)
    #[serde(default = "default_swing")]
    pub swing: f64,
}

fn default_swing() -> f64 { 0.5 }

#[derive(Serialize, Deserialize, Clone, Debug, Copy)]
pub struct RangeU8 {
    pub min: u8,
    pub max: u8,
}
```

### Harmony Configuration

```rust
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HarmonyConfig {
    /// Scale type used for note selection
    pub scale: ScaleType,
    /// How chords are voiced
    #[serde(default)]
    pub voicing: VoicingStyle,
    /// Chord progression (Roman numerals or absolute)
    #[serde(default)]
    pub progression: Vec<ChordSymbol>,
    /// Tension/dissonance level (0.0 = consonant, 1.0 = very dissonant)
    #[serde(default)]
    pub tension: f64,
    /// Allowed intervals for tension (semitones)
    #[serde(default)]
    pub tension_intervals: Vec<u8>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ScaleType {
    Major,
    NaturalMinor,
    HarmonicMinor,
    MelodicMinor,
    Dorian,
    Phrygian,
    Lydian,
    Mixolydian,
    Locrian,
    Pentatonic,
    Blues,
    Chromatic,
    /// Custom scale intervals from root
    Custom(Vec<u8>),
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "snake_case")]
pub enum VoicingStyle {
    #[default]
    Close,      // Notes close together
    Open,       // Spread voicings
    Drop2,      // Jazz drop-2 voicings
    Quartal,    // Stacked 4ths
    Shell,      // Root + 3rd + 7th only
    Power,      // Root + 5th only
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum ChordSymbol {
    Roman(String),    // "I", "IV", "vi", "V7"
    Absolute(String), // "Cmaj7", "Dm9"
}
```

### Dynamics Configuration

```rust
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DynamicsConfig {
    /// Base velocity (0-127)
    pub velocity_base: u8,
    /// Velocity variation range
    pub velocity_range: u8,
    /// How dynamics evolve over the piece
    #[serde(default)]
    pub build_curve: BuildCurve,
    /// Accent placement rules
    #[serde(default)]
    pub accents: AccentConfig,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "snake_case")]
pub enum BuildCurve {
    #[default]
    Flat,           // Constant dynamics
    Crescendo,      // Gradually louder
    Decrescendo,    // Gradually softer
    Wave,           // Gentle swells
    Terraced,       // Sudden level changes (baroque)
    Climax,         // Build to peak at end
    Fade,           // Start strong, fade out
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct AccentConfig {
    /// Accent on beat 1
    #[serde(default)]
    pub downbeat: u8,       // Velocity boost (0-40)
    /// Accent on off-beats (jazz/funk)
    #[serde(default)]
    pub offbeat: u8,
    /// Random accent probability
    #[serde(default)]
    pub random_prob: f64,
    #[serde(default)]
    pub random_boost: u8,
}
```

### Layer Definition

```rust
/// A single instrumental layer in the mood
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LayerDefinition {
    /// Layer name (for display/debugging)
    pub name: String,
    /// Musical role this layer plays
    pub role: LayerRole,
    /// Instruments to choose from (seed selects)
    pub instruments: InstrumentPool,
    /// Note register (octave range)
    pub register: RegisterConfig,
    /// Rhythmic/melodic pattern
    pub pattern: PatternConfig,
    /// Probability this layer is included (0.0-1.0)
    #[serde(default = "default_probability")]
    pub probability: f64,
    /// Velocity adjustment relative to base (-64 to +64)
    #[serde(default)]
    pub velocity_offset: i8,
    /// MIDI channel (0-15, or None for auto-assign)
    #[serde(default)]
    pub channel: Option<u8>,
}

fn default_probability() -> f64 { 1.0 }

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub enum LayerRole {
    Foundation,     // Bass, drones, pads - always-on harmonic base
    Melody,         // Lead melodic line
    CounterMelody,  // Secondary melodic interest
    Rhythm,         // Rhythmic drive (drums, percussion, strumming)
    Bass,           // Low-end harmonic support
    Accent,         // Sparse hits, stabs, punctuation
    Texture,        // Ambient fills, sustained pads
    Arpeggio,       // Arpeggiated patterns
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum InstrumentPool {
    /// Single instrument by name
    Single(String),
    /// List of instruments (seed selects one)
    List(Vec<String>),
    /// GM program number
    Program(u8),
    /// Category reference (resolved at runtime)
    Category(InstrumentCategory),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub enum InstrumentCategory {
    Strings,
    Brass,
    Woodwinds,
    Keyboards,
    Guitars,
    Basses,
    Synths,
    Pads,
    Bells,
    Percussion,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RegisterConfig {
    /// Base note (e.g., "C3", "G2")
    pub base: String,
    /// Range in semitones from base
    pub range: u8,
}
```

### Pattern Configuration

```rust
/// How notes are generated for a layer
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PatternConfig {
    /// Long held notes
    Sustained {
        #[serde(default)]
        duration_beats: f64,
    },

    /// Broken chord patterns
    Arpeggiated {
        /// Intervals from root (semitones)
        intervals: Vec<i8>,
        /// Beat positions within measure
        rhythm: RhythmPattern,
        /// Direction: up, down, updown, random
        #[serde(default)]
        direction: ArpeggioDirection,
    },

    /// Repeated rhythmic pattern
    Rhythmic {
        /// Beat positions for hits
        rhythm: RhythmPattern,
        /// Use chord tones or single note
        #[serde(default)]
        chord_tones: bool,
    },

    /// Melodic contour-based generation
    Melodic {
        /// Contour shape
        contour: ContourType,
        /// Notes per phrase
        phrase_length: RangeU8,
        /// Use scale degrees or chromatic
        #[serde(default = "default_true")]
        diatonic: bool,
    },

    /// Sparse random hits
    Sparse {
        /// Note density (0.0-1.0)
        density: f64,
        /// When to trigger
        #[serde(default)]
        trigger: SparseTrigger,
    },

    /// Walking bass style
    Walking {
        /// Style variation
        style: WalkingStyle,
    },

    /// Tremolo/trill effect
    Tremolo {
        /// Speed: slow, medium, fast
        speed: TremoloSpeed,
        /// Interval (semitones, 0 = same note)
        #[serde(default)]
        interval: u8,
    },

    /// Drum pattern (for rhythm layers)
    Drums {
        /// Named pattern or custom
        pattern: DrumPattern,
    },
}

fn default_true() -> bool { true }

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum RhythmPattern {
    /// Named preset pattern
    Preset(RhythmPreset),
    /// Custom beat positions
    Custom(Vec<f64>),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub enum RhythmPreset {
    WholeNotes,
    HalfNotes,
    QuarterNotes,
    EighthNotes,
    SixteenthNotes,
    Dotted,
    Syncopated,
    Waltz,
    Shuffle,
    Clave,
    Tresillo,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "snake_case")]
pub enum ArpeggioDirection {
    #[default]
    Up,
    Down,
    UpDown,
    DownUp,
    Random,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ContourType {
    /// Choose from built-in patterns
    Preset(ContourPreset),
    /// Custom direction sequence: 1=up, 0=hold, -1=down
    Custom(Vec<i8>),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ContourPreset {
    Ascending,
    Descending,
    Arch,           // up then down
    Valley,         // down then up
    Wave,           // gentle oscillation
    Zigzag,         // alternating
    Flat,           // same note repeated
    Random,         // seed-determined
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "snake_case")]
pub enum SparseTrigger {
    #[default]
    Random,
    Downbeat,
    Offbeat,
    PhraseStart,
    PhraseEnd,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub enum WalkingStyle {
    Standard,       // Quarter notes following harmony
    TwoFeel,        // Half notes
    Syncopated,     // With ghost notes
    Bebop,          // Chromatic approach notes
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub enum TremoloSpeed {
    Slow,           // Half note tremolo
    Medium,         // Eighth note
    Fast,           // Sixteenth note
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum DrumPattern {
    Preset(DrumPreset),
    Custom(Vec<DrumHit>),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub enum DrumPreset {
    Basic,
    Brushes,
    Swing,
    Latin,
    Rock,
    Shuffle,
    FourOnFloor,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DrumHit {
    pub beat: f64,
    pub drum: DrumVoice,
    #[serde(default = "default_velocity")]
    pub velocity: u8,
}

fn default_velocity() -> u8 { 80 }

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub enum DrumVoice {
    Kick,
    Snare,
    HiHatClosed,
    HiHatOpen,
    Ride,
    RideBell,
    Crash,
    Tom1,
    Tom2,
    FloorTom,
    SideStick,
    Clap,
}
```

### Embellishments Configuration

```rust
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct EmbellishmentConfig {
    /// Grace notes before main notes
    #[serde(default)]
    pub grace_notes: GraceNoteConfig,
    /// Fills at phrase boundaries
    #[serde(default)]
    pub fills: FillConfig,
    /// Trills and ornaments
    #[serde(default)]
    pub ornaments: OrnamentConfig,
    /// Dynamic swells
    #[serde(default)]
    pub swells: SwellConfig,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct GraceNoteConfig {
    /// Probability of adding grace note
    #[serde(default)]
    pub probability: f64,
    /// Interval from main note (semitones)
    #[serde(default)]
    pub intervals: Vec<i8>,     // e.g., [-2, -1, 1, 2]
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct FillConfig {
    /// Probability of fill at phrase end
    #[serde(default)]
    pub probability: f64,
    /// Duration in beats
    #[serde(default = "default_fill_duration")]
    pub duration: f64,
    /// Fill style
    #[serde(default)]
    pub style: FillStyle,
}

fn default_fill_duration() -> f64 { 0.5 }

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "snake_case")]
pub enum FillStyle {
    #[default]
    ScaleRun,
    Arpeggio,
    Trill,
    DrumFill,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct OrnamentConfig {
    /// Probability of trill
    #[serde(default)]
    pub trill_prob: f64,
    /// Probability of mordent
    #[serde(default)]
    pub mordent_prob: f64,
    /// Probability of turn
    #[serde(default)]
    pub turn_prob: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct SwellConfig {
    /// Probability of adding swell
    #[serde(default)]
    pub probability: f64,
    /// Swell duration in beats
    #[serde(default = "default_swell_duration")]
    pub duration: f64,
    /// Velocity increase at peak
    #[serde(default = "default_swell_amount")]
    pub amount: u8,
}

fn default_swell_duration() -> f64 { 2.0 }
fn default_swell_amount() -> u8 { 20 }
```

---

## UI Editor Design

### Main Editor Layout

```
┌─────────────────────────────────────────────────────────────────┐
│  Mood Editor: orchestral-dark                        [Save] [x] │
├────────────┬────────────────────────────────────────────────────┤
│            │                                                    │
│  Sections  │  Section Content                                   │
│  ────────  │  ─────────────────────────────────────────────     │
│            │                                                    │
│  ▼ Meta    │  ┌─ Meta ─────────────────────────────────────┐   │
│  ▼ Defaults│  │ Name: [orchestral-dark                   ] │   │
│  ▼ Harmony │  │ Description: [Ominous dramatic brass...  ] │   │
│  ▼ Dynamics│  │ Aliases: [orch-dark, ominous, dramatic   ] │   │
│  ▼ Layers  │  │ Tags: [orchestral, dark, cinematic       ] │   │
│    ├ strings│ └────────────────────────────────────────────┘   │
│    ├ brass │                                                    │
│    ├ timpani│                                                   │
│    └ tremolo│                                                   │
│  ▼ Embellish                                                    │
│            │                                                    │
├────────────┴────────────────────────────────────────────────────┤
│  Preview: [▶ Play]  Seed: [42____] Duration: [5s___]            │
└─────────────────────────────────────────────────────────────────┘
```

### Section-Specific Editors

#### Defaults Section
```
┌─ Defaults ──────────────────────────────────────────────────────┐
│                                                                 │
│  Key        ○ Single: [Dm     ▼]                               │
│             ● Pool:   [Dm, Am, Em, Cm                     ] [+] │
│                                                                 │
│  Tempo      Default: [60 ] BPM                                  │
│             Range:   [50 ]─────●───────[70 ]                    │
│             Swing:   [0.5]─●───────────[0.75]  (0.5 = straight) │
│                                                                 │
│  Intensity  Range:   [30 ]─────────●───[100]                    │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

#### Harmony Section
```
┌─ Harmony ───────────────────────────────────────────────────────┐
│                                                                 │
│  Scale       [Harmonic Minor  ▼]                                │
│              Preview: C D Eb F G Ab B                           │
│                                                                 │
│  Voicing     [Close ▼]  ○ Close  ○ Open  ○ Drop2  ○ Shell       │
│                                                                 │
│  Progression [i    ] [iv   ] [V    ] [i    ] [+] [-]            │
│              (Click to edit, drag to reorder)                   │
│                                                                 │
│  Tension     [0.0]───────●─────────[1.0]  (dissonance level)    │
│                                                                 │
│  Tension     □ b2 (♭2)   □ #4 (tritone)   ☑ m7                  │
│  Intervals   □ M7        □ b9             □ #9                  │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

#### Layer Editor
```
┌─ Layer: brass_swells ───────────────────────────────────────────┐
│                                                                 │
│  Name       [brass_swells            ]                          │
│  Role       [Accent        ▼]                                   │
│  Probability [0.0]──────●───────[1.0]  (0.8)                    │
│  Velocity   [─64]─────────●─────[+64]  (+20)                    │
│  Channel    [Auto ▼]  (0-15 or Auto)                            │
│                                                                 │
│  ┌─ Instruments ────────────────────────────────────────────┐   │
│  │ ● Category: [Brass        ▼]                             │   │
│  │ ○ List:     [french_horn, trumpet, trombone, tuba    ]   │   │
│  │ ○ Single:   [                                        ]   │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                 │
│  ┌─ Register ───────────────────────────────────────────────┐   │
│  │ Base Note: [C3  ▼]   Range: [18] semitones               │   │
│  │                                                          │   │
│  │ ┌───────────────────────────────────────────────────┐    │   │
│  │ │ C2    C3    C4    C5    C6    C7                  │    │   │
│  │ │       ╠═════════════════╣                          │    │   │
│  │ │       ↑ base           ↑ base+range               │    │   │
│  │ └───────────────────────────────────────────────────┘    │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                 │
│  ┌─ Pattern ────────────────────────────────────────────────┐   │
│  │ Type: [Sparse         ▼]                                 │   │
│  │                                                          │   │
│  │ Density:  [0.0]────●─────────[1.0]  (0.3)                │   │
│  │ Trigger:  ○ Random  ● Downbeat  ○ Offbeat  ○ Phrase      │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                 │
│  [Delete Layer]                                                 │
└─────────────────────────────────────────────────────────────────┘
```

#### Pattern Type Options

Based on the selected pattern type, show relevant parameters:

**Sustained:**
```
Duration: [4.0] beats (whole note)
```

**Arpeggiated:**
```
Intervals: [0, 4, 7, 12] (click to edit as piano roll)
Rhythm:    ○ Quarter  ● Eighth  ○ Sixteenth  ○ Custom: [0, 0.5, 1, 1.5]
Direction: ○ Up  ○ Down  ● Up-Down  ○ Random
```

**Melodic:**
```
Contour:      [Arch          ▼]  Preview: ╱╲
              ○ Ascending  ○ Descending  ● Arch  ○ Valley  ○ Custom
Custom:       [1, 1, 1, -1, -1, -1, -1]  (1=up, 0=hold, -1=down)

Phrase Length: [4  ]─────●─────[8  ]  (range)
Diatonic:      ☑ Stay in scale
```

**Drums:**
```
Pattern: [Brushes        ▼]
         ● Preset  ○ Custom

Custom Pattern Editor:
         1   2   3   4   |
Kick     ●   ○   ○   ○   |
Snare    ○   ○   ●   ○   |
HiHat    ●   ●   ●   ●   |
Ride     ○   ○   ○   ○   |
```

### Layer List Management

```
┌─ Layers ────────────────────────────────────────────────────────┐
│                                                                 │
│  ≡  ● strings_foundation   Foundation   strings      1.0       │
│  ≡  ● brass_swells         Accent       brass        0.8       │
│  ≡  ● timpani_hits         Accent       percussion   0.7       │
│  ≡  ○ tremolo_texture      Texture      strings      0.5       │
│                                                                 │
│  (● = included at current intensity, ○ = excluded)              │
│  (drag ≡ to reorder, click to edit)                             │
│                                                                 │
│  [+ Add Layer]  Templates: [Foundation ▼] [Melody ▼] [Rhythm ▼] │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## Trait Library (Reusable Algorithms)

The pattern configurations map to trait implementations:

```rust
/// Trait for generating melodic content
pub trait MelodicGenerator: Send + Sync {
    fn generate_notes(
        &self,
        config: &PatternConfig,
        register: &RegisterConfig,
        harmony: &HarmonyConfig,
        variation: &PresetVariation,
        beats: f64,
        rng: &mut impl Rng,
    ) -> Vec<Note>;
}

/// Trait for generating rhythmic patterns
pub trait RhythmGenerator: Send + Sync {
    fn generate_rhythm(
        &self,
        pattern: &RhythmPattern,
        beats: f64,
        swing: f64,
        rng: &mut impl Rng,
    ) -> Vec<f64>;  // Returns beat positions
}

/// Trait for applying dynamics
pub trait DynamicsProcessor: Send + Sync {
    fn apply_dynamics(
        &self,
        notes: &mut [Note],
        config: &DynamicsConfig,
        total_beats: f64,
    );
}

/// Trait for adding embellishments
pub trait Embellisher: Send + Sync {
    fn embellish(
        &self,
        notes: &mut Vec<Note>,
        config: &EmbellishmentConfig,
        variation: &PresetVariation,
        rng: &mut impl Rng,
    );
}
```

### Built-in Implementations

```rust
// These are always available
pub struct StandardMelodicGenerator;
pub struct JazzWalkingBassGenerator;
pub struct ArpeggioGenerator;
pub struct DrumPatternGenerator;
pub struct StandardRhythmGenerator;
pub struct SwingRhythmGenerator;
pub struct BuildCurveDynamics;
pub struct GraceNoteEmbellisher;
pub struct FillEmbellisher;

// Registry maps pattern types to implementations
pub fn get_melodic_generator(pattern: &PatternConfig) -> Box<dyn MelodicGenerator> {
    match pattern {
        PatternConfig::Melodic { .. } => Box::new(StandardMelodicGenerator),
        PatternConfig::Walking { .. } => Box::new(JazzWalkingBassGenerator),
        PatternConfig::Arpeggiated { .. } => Box::new(ArpeggioGenerator),
        // ... etc
    }
}
```

---

## Example TOML

```toml
[meta]
name = "orchestral-dark"
description = "Ominous dramatic brass and timpani"
aliases = ["orch-dark", "ominous"]
tags = ["orchestral", "dark", "cinematic"]

[defaults]
key = { pool = ["Dm", "Am", "Em"] }
tempo = { default = 60, range = [50, 70], swing = 0.5 }
intensity = { min = 40, max = 100 }

[harmony]
scale = "harmonic_minor"
voicing = "close"
progression = ["i", "iv", "V", "i"]
tension = 0.6
tension_intervals = [1, 6]  # minor 2nd, tritone

[dynamics]
velocity_base = 50
velocity_range = 40
build_curve = "crescendo"

[dynamics.accents]
downbeat = 15
random_prob = 0.1
random_boost = 20

[[layers]]
name = "low_strings_drone"
role = "foundation"
instruments = ["cello", "contrabass"]
register = { base = "C2", range = 12 }
probability = 1.0
velocity_offset = 0

[layers.pattern]
type = "sustained"
duration_beats = 4.0

[[layers]]
name = "brass_swells"
role = "accent"
instruments = { category = "brass" }
register = { base = "C3", range = 18 }
probability = 0.8
velocity_offset = 20

[layers.pattern]
type = "sparse"
density = 0.3
trigger = "downbeat"

[[layers]]
name = "timpani_hits"
role = "accent"
instruments = ["timpani"]
register = { base = "C2", range = 12 }
probability = 0.7
velocity_offset = 30

[layers.pattern]
type = "rhythmic"
rhythm = { custom = [0.0, 0.5, 1.5, 3.0] }
chord_tones = false

[[layers]]
name = "tremolo_tension"
role = "texture"
instruments = ["tremolo_strings"]
register = { base = "C4", range = 12 }
probability = 0.5
velocity_offset = -5

[layers.pattern]
type = "tremolo"
speed = "fast"
interval = 0

[embellishments.swells]
probability = 0.3
duration = 2.0
amount = 25
```

---

## Implementation Roadmap

1. **Phase 1**: Core Types
   - Define all structs in `src/mood/config.rs`
   - Implement serde serialization
   - Add TOML parsing tests

2. **Phase 2**: Trait Library
   - Extract common algorithms from existing presets
   - Implement trait-based generators
   - Create registry for pattern -> generator mapping

3. **Phase 3**: Runtime Engine
   - `MoodDefinition::generate()` that uses trait implementations
   - Integration with existing `MoodGenerator` trait
   - Validate TOML at load time

4. **Phase 4**: Web UI Editor
   - Section-based collapsible editor
   - Layer list with drag-and-drop
   - Live preview with seed/duration controls
   - Export to TOML

5. **Phase 5**: Migration
   - Convert existing compiled moods to TOML
   - Deprecate hardcoded implementations
   - Community mood pack repository
