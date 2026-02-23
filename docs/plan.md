# Implementation Plan: midi-cli-rs

## Phase Overview

| Phase | Focus | Deliverables | Status |
|-------|-------|--------------|--------|
| 1 | Core MIDI Generation | Note parsing, MIDI output | Complete |
| 2 | CLI Interface | clap-based CLI, JSON input | Complete |
| 3 | WAV Rendering | FluidSynth integration | Complete |
| 4 | Mood Presets | Suspense, eerie, upbeat, calm, ambient, jazz | Complete |
| 5 | Polish | Error handling, docs, tests, web UI | Complete |
| 6 | Future Enhancements | Mood packs, plugin architecture, new moods | Planned |

---

## Phase 1: Core MIDI Generation

**Goal**: Generate valid MIDI files from programmatic note data

### Tasks

- [ ] **1.1** Add dependencies to Cargo.toml
  - `midly` for MIDI parsing/writing
  - `thiserror` for error types

- [ ] **1.2** Create Note data structure
  - File: `src/midi/note.rs`
  - Struct: pitch, duration, velocity, offset
  - Parse from string format: `"C4:1:80"`
  - Convert note names to MIDI numbers

- [ ] **1.3** Create NoteSequence structure
  - File: `src/midi/sequence.rs`
  - Collection of notes with instrument and tempo
  - Validation logic

- [ ] **1.4** Implement MIDI file writer
  - File: `src/midi/writer.rs`
  - Use `midly` to create SMF Format 1
  - Write tempo track + instrument tracks
  - Output .mid files

- [ ] **1.5** Unit tests for MIDI generation
  - Test note parsing
  - Test MIDI file validity (re-parse generated files)
  - Test edge cases (empty sequence, single note, max velocity)

### Acceptance Criteria

- [x] `Note::parse("C4:1:80")` returns correct MIDI pitch 60
- [x] Generated .mid files open in standard MIDI players
- [x] Round-trip: generate -> parse -> compare is identical

---

## Phase 2: CLI Interface

**Goal**: Expose MIDI generation via command-line interface

### Tasks

- [ ] **2.1** Add CLI dependencies
  - `clap` with derive feature
  - `serde` and `serde_json` for JSON input

- [ ] **2.2** Implement CLI structure
  - File: `src/main.rs` and `src/input/cli.rs`
  - Commands: generate, instruments, help
  - Global flags: verbose, quiet

- [ ] **2.3** Implement note input via CLI args
  - `--notes "C4:1:80,E4:0.5:100"`
  - Parse comma-separated notes

- [ ] **2.4** Implement JSON stdin input
  - `--json` flag to read from stdin
  - Parse JSON note sequences
  - Support single-track and multi-track formats

- [ ] **2.5** Implement instrument listing
  - `instruments` command
  - List all named instruments with GM program numbers

- [ ] **2.6** Integration tests for CLI
  - Test CLI invocation with various arguments
  - Test JSON input piping
  - Test error messages

### Acceptance Criteria

- [ ] `midi-cli-rs generate --notes "C4:1:80" -o test.mid` creates valid MIDI
- [ ] `echo '{"notes":[...]}' | midi-cli-rs generate --json -o test.mid` works
- [ ] Clear error messages for invalid input

---

## Phase 3: WAV Rendering

**Goal**: Render MIDI files to WAV audio using FluidSynth

### Tasks

- [ ] **3.1** Implement FluidSynth detection
  - File: `src/render/fluidsynth.rs`
  - Check if `fluidsynth` binary is available
  - Provide installation instructions if missing

- [ ] **3.2** Implement SoundFont discovery
  - File: `src/render/soundfont.rs`
  - Search standard paths for SoundFonts
  - Support `--soundfont` override

- [ ] **3.3** Implement WAV rendering
  - Call FluidSynth as external process
  - Handle output format detection from file extension
  - Support `.wav` output

- [ ] **3.4** Add `render` command
  - Convert existing MIDI files to WAV
  - `midi-cli-rs render input.mid -o output.wav`

- [ ] **3.5** Integration tests for rendering
  - Test WAV generation (if FluidSynth available)
  - Test error handling when FluidSynth missing

### Acceptance Criteria

- [ ] `midi-cli-rs generate --notes "..." -o test.wav` produces audio
- [ ] WAV files play correctly in standard audio players
- [ ] Clear error message if FluidSynth not installed

---

## Phase 4: Mood Presets

**Goal**: Algorithmic mood-based music generation

### Tasks

- [ ] **4.1** Create MoodGenerator trait
  - File: `src/preset/mod.rs`
  - Define interface for mood generators
  - Seeded randomness for reproducibility

- [ ] **4.2** Implement Suspense preset
  - File: `src/preset/suspense.rs`
  - Low drone + high tremolo + sparse hits
  - Minor key, dissonant intervals

- [ ] **4.3** Implement Eerie preset
  - File: `src/preset/eerie.rs`
  - Sparse, wide intervals, diminished harmony
  - Pad-based with bell accents

- [ ] **4.4** Implement Upbeat preset
  - File: `src/preset/upbeat.rs`
  - Major key, rhythmic, clear pulse
  - Piano + bass pattern

- [ ] **4.5** Implement Calm preset
  - File: `src/preset/calm.rs`
  - Sustained pads, gentle arpeggios
  - Major 7th harmony

- [ ] **4.6** Implement Ambient preset
  - File: `src/preset/ambient.rs`
  - Drones, textures, evolving
  - Pentatonic sporadic tones

- [ ] **4.7** Add `preset` command
  - `midi-cli-rs preset --mood suspense -o suspense.wav`
  - Support duration, key, intensity, seed

- [ ] **4.8** Tests for presets
  - Determinism tests (same seed = same output)
  - Output validation

### Acceptance Criteria

- [ ] `midi-cli-rs preset --mood suspense --seed 42 -o test.wav` works
- [ ] Same seed produces identical output
- [ ] Each mood has distinct character

---

## Phase 5: Polish

**Goal**: Production-ready quality

### Tasks

- [ ] **5.1** Comprehensive error handling
  - All error paths covered
  - AI-friendly error messages

- [ ] **5.2** Documentation
  - README with examples
  - `--help` text is comprehensive
  - Doc comments on public API

- [ ] **5.3** CI/CD setup
  - GitHub Actions workflow
  - Test on Linux, macOS
  - Clippy and fmt checks

- [ ] **5.4** Performance optimization
  - Profile and optimize hot paths
  - Benchmark typical use cases

- [ ] **5.5** Example scripts
  - Shell scripts showing AI agent workflows
  - Multi-track layering examples

### Acceptance Criteria

- [ ] All clippy warnings resolved
- [ ] Code coverage > 80%
- [ ] README is complete and accurate

---

## Dependencies

### Rust Crates

| Crate | Version | Purpose | License |
|-------|---------|---------|---------|
| midly | ^0.5 | MIDI parsing/writing | MIT/Apache-2.0 |
| clap | ^4 | CLI parsing | MIT/Apache-2.0 |
| serde | ^1 | Serialization | MIT/Apache-2.0 |
| serde_json | ^1 | JSON parsing | MIT/Apache-2.0 |
| thiserror | ^1 | Error types | MIT/Apache-2.0 |
| rand | ^0.8 | Randomization | MIT/Apache-2.0 |
| dirs | ^5 | Directory paths | MIT/Apache-2.0 |

### External Dependencies

| Tool | Purpose | License | Installation |
|------|---------|---------|--------------|
| FluidSynth | WAV rendering | LGPL-2.1 | `brew install fluid-synth` |
| FluidR3_GM.sf2 | SoundFont | MIT | Bundled with FluidSynth |

---

## Risk Mitigation

| Risk | Mitigation |
|------|------------|
| FluidSynth not available | Clear error messages, optional feature |
| SoundFont licensing | Only recommend MIT/CC0 SoundFonts |
| MIDI complexity | Start simple, expand as needed |
| Cross-platform issues | Test on Linux and macOS early |

---

## Success Metrics

- [x] AI agent can generate 5-second intro music with single CLI call
- [x] Generation + rendering completes in < 2 seconds
- [x] Zero licensing concerns for commercial use
- [x] 6 distinct mood presets available (suspense, eerie, upbeat, calm, ambient, jazz)

---

## Phase 6: Mood Plugin Architecture

**Goal**: Extensible mood system where each mood is a distinct, selectable preset. Seeds provide variation *within* a mood, not selection between moods.

### Design Principles

1. **Each mood is distinct**: `orchestral-dark` and `orchestral-relaxing` are separate moods, not seed variations
2. **Seeds vary within a mood**: Same mood + different seed = different instruments, rhythms, velocities
3. **Mood packs bundle related moods**: A pack like "orchestral" contains multiple distinct moods
4. **Inheritance reduces duplication**: Moods can extend base definitions

---

### 6.1 Mood Hierarchy

```
MoodPack (file: orchestral.toml)
├── [base]                    # Shared settings for all moods in pack
│   ├── instrument_pools
│   ├── scales
│   └── common layers
│
├── orchestral-relaxing       # Distinct mood
├── orchestral-dark           # Distinct mood
├── orchestral-energetic      # Distinct mood
└── orchestral-baroque        # Distinct mood
```

**CLI Usage**:
```bash
# Each mood is directly selectable
midi-cli-rs preset -m orchestral-dark -d 10 -o dark.wav
midi-cli-rs preset -m orchestral-relaxing -d 10 -o relax.wav

# Seeds vary WITHIN that mood
midi-cli-rs preset -m orchestral-dark --seed 1 -o dark1.wav
midi-cli-rs preset -m orchestral-dark --seed 2 -o dark2.wav  # Different variation, same dark character

# List all moods including sub-moods
midi-cli-rs moods
# Output:
#   orchestral-relaxing  G    Peaceful flowing strings and woodwinds
#   orchestral-dark      Dm   Ominous brass and timpani
#   orchestral-energetic C    Driving full orchestra
#   orchestral-baroque   D    Period harpsichord and counterpoint
```

---

### 6.2 MoodPack File Format (TOML)

```toml
# ~/.midi-cli-rs/moods/orchestral.toml

[pack]
name = "orchestral"
version = "1.0"
author = "midi-cli-rs"
description = "Classical orchestral mood family"

# ============================================================
# BASE DEFINITIONS (inherited by all moods in this pack)
# ============================================================

[base]
# Shared instrument pools that moods can reference by name
[base.instruments]
strings_ensemble = ["strings", "tremolo_strings", "string_ensemble_1", "string_ensemble_2"]
woodwinds = ["flute", "oboe", "clarinet", "bassoon"]
brass = ["french_horn", "trumpet", "trombone", "tuba"]
percussion = ["timpani", "orchestral_hit", "tubular_bells"]
keyboards = ["harpsichord", "celesta", "piano"]

# Shared scale definitions
[base.scales]
major = [0, 2, 4, 5, 7, 9, 11]
natural_minor = [0, 2, 3, 5, 7, 8, 10]
harmonic_minor = [0, 2, 3, 5, 7, 8, 11]
dorian = [0, 2, 3, 5, 7, 9, 10]

# Shared rhythm patterns (in beats)
[base.rhythms]
sustained = [0.0]
quarter_notes = [0.0, 1.0, 2.0, 3.0]
half_notes = [0.0, 2.0]
waltz = [0.0, 1.0, 2.0]
fanfare = [0.0, 0.5, 1.5, 3.0]

# ============================================================
# MOOD: orchestral-relaxing
# ============================================================

[[moods]]
name = "orchestral-relaxing"
aliases = ["orch-relax", "peaceful-orchestra"]
description = "Peaceful flowing strings and woodwinds"
default_key = "G"
default_tempo = 70
tempo_range = [60, 80]

[moods.dynamics]
velocity_base = 60
velocity_range = 20
build_curve = "wave"      # gentle swells

[moods.harmony]
scale = "major"
mode = "ionian"
chord_style = "open"      # wide voicings
progression = ["I", "vi", "IV", "V"]

[[moods.layers]]
name = "strings_foundation"
instruments = "@strings_ensemble"   # Reference base pool
role = "foundation"
register = { base = "C3", range = 24 }
pattern = "sustained"
probability = 1.0
velocity_offset = 0

[[moods.layers]]
name = "woodwind_melody"
instruments = "@woodwinds"
role = "melody"
register = { base = "C4", range = 12 }
pattern = { type = "melodic", contour = "arch", phrase_length = [4, 8] }
probability = 0.8
velocity_offset = 10

[[moods.layers]]
name = "harp_arpeggios"
instruments = ["harp"]
role = "texture"
register = { base = "C4", range = 24 }
pattern = { type = "arpeggiated", intervals = [0, 4, 7, 12], rhythm = "@quarter_notes" }
probability = 0.6
velocity_offset = -10

# ============================================================
# MOOD: orchestral-dark
# ============================================================

[[moods]]
name = "orchestral-dark"
aliases = ["orch-dark", "ominous", "dramatic"]
description = "Ominous dramatic brass and timpani"
default_key = "Dm"
default_tempo = 60
tempo_range = [50, 70]

[moods.dynamics]
velocity_base = 50
velocity_range = 40        # Wide dynamic range for drama
build_curve = "crescendo"

[moods.harmony]
scale = "harmonic_minor"
mode = "aeolian"
chord_style = "close"      # dense voicings
progression = ["i", "iv", "V", "i"]
tension_intervals = [1, 6]  # minor 2nd, tritone

[[moods.layers]]
name = "low_strings_drone"
instruments = ["cello", "contrabass"]
role = "foundation"
register = { base = "C2", range = 12 }
pattern = "sustained"
probability = 1.0
velocity_offset = 0

[[moods.layers]]
name = "brass_swells"
instruments = "@brass"
role = "accent"
register = { base = "C3", range = 18 }
pattern = { type = "sparse", density = 0.3, trigger = "downbeat" }
probability = 0.8
velocity_offset = 20

[[moods.layers]]
name = "timpani_hits"
instruments = ["timpani"]
role = "accent"
register = { base = "C2", range = 12 }
pattern = { type = "rhythmic", rhythm = "@fanfare" }
probability = 0.7
velocity_offset = 30

[[moods.layers]]
name = "tremolo_tension"
instruments = ["tremolo_strings"]
role = "texture"
register = { base = "C4", range = 12 }
pattern = { type = "tremolo", speed = "fast" }
probability = 0.5
velocity_offset = -5

# ============================================================
# MOOD: orchestral-energetic
# ============================================================

[[moods]]
name = "orchestral-energetic"
aliases = ["orch-energy", "driving", "action"]
description = "Driving full orchestra with percussion"
default_key = "C"
default_tempo = 130
tempo_range = [120, 145]

[moods.dynamics]
velocity_base = 80
velocity_range = 30
build_curve = "flat"       # Consistently high energy

[moods.harmony]
scale = "major"
mode = "mixolydian"        # Slightly edgy major
chord_style = "power"
progression = ["I", "IV", "V", "I"]

[[moods.layers]]
name = "full_strings"
instruments = "@strings_ensemble"
role = "foundation"
register = { base = "C3", range = 24 }
pattern = { type = "rhythmic", rhythm = "@quarter_notes" }
probability = 1.0
velocity_offset = 0

[[moods.layers]]
name = "brass_fanfare"
instruments = "@brass"
role = "melody"
register = { base = "C4", range = 12 }
pattern = { type = "melodic", contour = "ascending", phrase_length = [2, 4] }
probability = 0.9
velocity_offset = 15

[[moods.layers]]
name = "percussion_drive"
instruments = "@percussion"
role = "rhythm"
register = { base = "C3", range = 12 }
pattern = { type = "rhythmic", rhythm = "@quarter_notes" }
probability = 1.0
velocity_offset = 10

[[moods.layers]]
name = "woodwind_flourish"
instruments = "@woodwinds"
role = "accent"
register = { base = "C5", range = 12 }
pattern = { type = "sparse", density = 0.4, trigger = "phrase_end" }
probability = 0.6
velocity_offset = 5

# ============================================================
# MOOD: orchestral-baroque
# ============================================================

[[moods]]
name = "orchestral-baroque"
aliases = ["baroque", "classical", "period"]
description = "Period-authentic harpsichord and counterpoint"
default_key = "D"
default_tempo = 90
tempo_range = [80, 100]

[moods.dynamics]
velocity_base = 70
velocity_range = 15        # Baroque had limited dynamics
build_curve = "terraced"   # Sudden dynamic shifts

[moods.harmony]
scale = "major"
mode = "ionian"
chord_style = "baroque"    # Figured bass style
progression = ["I", "V", "vi", "IV", "I", "V", "I"]

[[moods.layers]]
name = "harpsichord_continuo"
instruments = ["harpsichord"]
role = "foundation"
register = { base = "C3", range = 24 }
pattern = { type = "arpeggiated", intervals = [0, 4, 7, 4], rhythm = [0.0, 0.5, 1.0, 1.5] }
probability = 1.0
velocity_offset = 0

[[moods.layers]]
name = "violin_melody"
instruments = ["violin"]
role = "melody"
register = { base = "G4", range = 12 }
pattern = { type = "melodic", contour = "baroque_sequence", phrase_length = [4, 8] }
probability = 1.0
velocity_offset = 10

[[moods.layers]]
name = "cello_bass"
instruments = ["cello"]
role = "bass"
register = { base = "C2", range = 12 }
pattern = { type = "walking", style = "baroque_bass" }
probability = 1.0
velocity_offset = -5

[[moods.layers]]
name = "flute_counterpoint"
instruments = ["flute", "oboe"]
role = "counter_melody"
register = { base = "C5", range = 12 }
pattern = { type = "melodic", contour = "contrary", phrase_length = [4, 8] }
probability = 0.7
velocity_offset = 5
```

---

### 6.3 Rust Data Structures

```rust
/// A mood pack containing multiple related moods with shared base definitions.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MoodPack {
    pub pack: PackMeta,
    #[serde(default)]
    pub base: BaseDefs,
    pub moods: Vec<MoodDef>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PackMeta {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

/// Shared definitions inherited by all moods in the pack.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct BaseDefs {
    #[serde(default)]
    pub instruments: HashMap<String, Vec<String>>,
    #[serde(default)]
    pub scales: HashMap<String, Vec<i8>>,
    #[serde(default)]
    pub rhythms: HashMap<String, Vec<f64>>,
}

/// A single mood definition.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MoodDef {
    pub name: String,
    #[serde(default)]
    pub aliases: Vec<String>,
    pub description: String,
    pub default_key: String,
    pub default_tempo: u16,
    #[serde(default)]
    pub tempo_range: Option<(u16, u16)>,
    pub dynamics: DynamicsDef,
    pub harmony: HarmonyDef,
    pub layers: Vec<LayerDef>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DynamicsDef {
    pub velocity_base: u8,
    pub velocity_range: u8,
    #[serde(default = "default_build_curve")]
    pub build_curve: BuildCurve,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub enum BuildCurve {
    Flat,
    Crescendo,
    Decrescendo,
    Wave,
    Terraced,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HarmonyDef {
    pub scale: String,              // Reference to base.scales or literal
    #[serde(default)]
    pub mode: Option<String>,
    #[serde(default)]
    pub chord_style: Option<String>,
    #[serde(default)]
    pub progression: Vec<String>,   // Roman numerals
    #[serde(default)]
    pub tension_intervals: Vec<i8>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LayerDef {
    pub name: String,
    pub instruments: InstrumentRef,  // "@pool_name" or ["inst1", "inst2"]
    pub role: LayerRole,
    pub register: RegisterDef,
    pub pattern: PatternDef,
    #[serde(default = "default_probability")]
    pub probability: f64,
    #[serde(default)]
    pub velocity_offset: i8,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum InstrumentRef {
    PoolRef(String),           // "@strings_ensemble"
    Literal(Vec<String>),      // ["violin", "viola"]
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub enum LayerRole {
    Foundation,
    Melody,
    CounterMelody,
    Bass,
    Rhythm,
    Accent,
    Texture,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RegisterDef {
    pub base: String,    // "C3"
    pub range: u8,       // Semitones
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum PatternDef {
    Simple(String),      // "sustained"
    Complex {
        #[serde(rename = "type")]
        pattern_type: PatternType,
        #[serde(flatten)]
        params: PatternParams,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub enum PatternType {
    Sustained,
    Arpeggiated,
    Rhythmic,
    Melodic,
    Sparse,
    Walking,
    Tremolo,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct PatternParams {
    #[serde(default)]
    pub intervals: Option<Vec<i8>>,
    #[serde(default)]
    pub rhythm: Option<RhythmRef>,
    #[serde(default)]
    pub contour: Option<String>,
    #[serde(default)]
    pub phrase_length: Option<Vec<u8>>,
    #[serde(default)]
    pub density: Option<f64>,
    #[serde(default)]
    pub trigger: Option<String>,
    #[serde(default)]
    pub style: Option<String>,
    #[serde(default)]
    pub speed: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum RhythmRef {
    PoolRef(String),       // "@quarter_notes"
    Literal(Vec<f64>),     // [0.0, 0.5, 1.0]
}
```

---

### 6.4 Seed Variation System

Seeds control variation **within** a mood, not between moods:

```rust
impl MoodDef {
    /// Generate note sequences using seed for variation.
    pub fn generate(&self, config: &PresetConfig, base: &BaseDefs) -> Vec<NoteSequence> {
        let variation = PresetVariation::from_seed(config.seed);
        let mut sequences = Vec::new();

        for (i, layer) in self.layers.iter().enumerate() {
            // Seed determines: which instrument from pool, layer inclusion, velocities
            if variation.layer_probs[i] < layer.probability {
                let instrument = self.pick_instrument(layer, &variation, i, base);
                let notes = self.generate_layer(layer, &variation, config, base);
                sequences.push(NoteSequence { instrument, notes, .. });
            }
        }

        sequences
    }

    fn pick_instrument(&self, layer: &LayerDef, var: &PresetVariation, idx: usize, base: &BaseDefs) -> String {
        let pool = layer.resolve_instruments(base);
        let index = var.instrument_indices[idx] as usize % pool.len();
        pool[index].clone()
    }
}
```

**What seeds vary**:
- Which instrument is picked from a layer's pool
- Whether optional layers are included (probability threshold)
- Velocity variations within the allowed range
- Melodic contour selection (from 16 patterns)
- Phrase lengths within allowed range
- Rest placement
- Tempo micro-variations (±15%)

**What seeds do NOT vary**:
- The mood's character (dark stays dark)
- Layer roles and structure
- Harmonic language (scale, progression)
- Register ranges
- Instrument pools available

---

### 6.5 MoodRegistry

```rust
pub struct MoodRegistry {
    /// Built-in moods (compiled Rust)
    builtin: HashMap<String, Box<dyn MoodGenerator>>,
    /// User moods from TOML packs
    packs: HashMap<String, MoodPack>,
    /// Flattened mood lookup (name -> pack + mood index)
    mood_index: HashMap<String, (String, usize)>,
}

impl MoodRegistry {
    pub fn load() -> Result<Self, Error> {
        let mut registry = Self::load_builtin();

        // Load user packs from ~/.midi-cli-rs/moods/*.toml
        let mood_dir = dirs::config_dir()
            .unwrap_or_default()
            .join("midi-cli-rs")
            .join("moods");

        if mood_dir.exists() {
            for entry in fs::read_dir(&mood_dir)? {
                let path = entry?.path();
                if path.extension() == Some("toml".as_ref()) {
                    let pack: MoodPack = toml::from_str(&fs::read_to_string(&path)?)?;
                    registry.register_pack(pack)?;
                }
            }
        }

        Ok(registry)
    }

    pub fn list_moods(&self) -> Vec<MoodInfo> {
        // Returns all moods: builtin + user-defined
    }

    pub fn get_mood(&self, name: &str) -> Option<&dyn MoodGenerator> {
        // Lookup by name or alias
    }
}
```

---

### 6.6 CLI Commands

```bash
# List all moods (builtin + user packs)
midi-cli-rs moods
# Output:
#   BUILTIN MOODS:
#   suspense        Am   Tense low drone with tremolo
#   eerie           Dm   Sparse unsettling atmosphere
#   upbeat          C    Energetic rhythmic feel
#   calm            G    Peaceful sustained pads
#   ambient         Em   Evolving drone textures
#   jazz            F    Nightclub trio style
#
#   USER MOODS (from ~/.midi-cli-rs/moods/):
#   orchestral-relaxing   G    Peaceful flowing strings and woodwinds
#   orchestral-dark       Dm   Ominous dramatic brass and timpani
#   orchestral-energetic  C    Driving full orchestra
#   orchestral-baroque    D    Period-authentic counterpoint

# Generate with user mood
midi-cli-rs preset -m orchestral-dark -d 10 --seed 42 -o dark.wav

# Export builtin mood as editable TOML
midi-cli-rs mood export jazz -o ~/.midi-cli-rs/moods/my-jazz.toml

# Validate a mood pack
midi-cli-rs mood validate ~/.midi-cli-rs/moods/custom.toml

# Show mood details
midi-cli-rs mood info orchestral-dark
# Output:
#   Name: orchestral-dark
#   Pack: orchestral
#   Key: Dm  Tempo: 60 (50-70)
#   Description: Ominous dramatic brass and timpani
#   Layers:
#     1. low_strings_drone (foundation) - cello, contrabass
#     2. brass_swells (accent) - french_horn, trumpet, trombone, tuba
#     3. timpani_hits (accent) - timpani
#     4. tremolo_tension (texture) - tremolo_strings

# Create new mood pack interactively (future)
midi-cli-rs mood new my-pack
```

---

### 6.7 Planned Mood Packs

| Pack | Moods | Character |
|------|-------|-----------|
| **orchestral** | relaxing, dark, energetic, baroque | Classical orchestra variations |
| **show** | ballad, uptempo, dramatic, comedic | Broadway/musical theater |
| **electronic** | ambient, driving, glitch, chillwave | Synth-based styles |
| **world** | asian, celtic, middle-eastern, latin | Ethnic scales and instruments |
| **cinematic** | tension, action, romance, wonder | Film score emotions |
| **retro** | lo-fi, synthwave, chiptune, disco | Period electronic styles |

---

### Tasks

- [ ] **6.1** Define MoodPack TOML schema (this document)
- [ ] **6.2** Implement MoodPack/MoodDef structs with serde
- [ ] **6.3** Implement reference resolution (@pool_name)
- [ ] **6.4** Create MoodRegistry with builtin + user loading
- [ ] **6.5** Implement MoodDef::generate() from TOML definition
- [ ] **6.6** Add `mood export` CLI command
- [ ] **6.7** Add `mood validate` CLI command
- [ ] **6.8** Add `mood info` CLI command
- [ ] **6.9** Create orchestral.toml example pack
- [ ] **6.10** Create show.toml example pack
- [ ] **6.11** Web UI mood pack browser
- [ ] **6.12** Web UI mood pack editor

### Acceptance Criteria

- [ ] User moods from TOML work identically to builtin moods
- [ ] `midi-cli-rs moods` lists all available moods
- [ ] Seeds vary output within a mood, not the mood character
- [ ] Builtin moods can be exported and modified
- [ ] Invalid TOML packs produce clear error messages
- [ ] 15+ moods available (6 builtin + orchestral pack + show pack)
