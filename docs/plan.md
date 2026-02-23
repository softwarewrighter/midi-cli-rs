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

## Phase 6: Future Mood Enhancements

**Goal**: Extensible mood system with user-defined presets and plugin architecture

### Planned Moods

#### Show/Musical (Broadway/Hollywood Style)
- **Character**: Dramatic, theatrical, emotionally expressive
- **Layers**:
  - Orchestra strings (lush, sweeping)
  - Brass fanfares and accents
  - Piano accompaniment (arpeggiated, rhythmic)
  - Vocal-range melody line (for singability)
- **Patterns**: Big crescendos, dramatic pauses, key changes
- **Keys**: Bb, Eb, F (common Broadway keys)

#### Orchestral Family
Sub-moods sharing orchestral instrumentation with different characters:

| Sub-mood | Character | Tempo | Key | Emphasis |
|----------|-----------|-------|-----|----------|
| orchestral-relaxing | Peaceful, flowing | 60-80 | G, D | Strings, woodwinds, harp |
| orchestral-dark | Ominous, dramatic | 50-70 | Dm, Am | Low brass, timpani, cello |
| orchestral-energetic | Exciting, driving | 120-140 | C, A | Full orchestra, percussion |
| orchestral-baroque | Period-authentic | 80-100 | D, G | Harpsichord, strings, counterpoint |

#### Additional Future Moods
- **cinematic**: Film score style with dynamic arcs
- **electronic**: Synth-based with arpeggios and pads
- **world**: Ethnic scales and instruments (pentatonic, modes)
- **lo-fi**: Jazzy chords with tape-style warmth
- **chiptune**: 8-bit style with square/triangle waves

### 6.1 Mood Pack Architecture

**Goal**: Allow mood presets to be defined as data files rather than compiled code

#### MoodPack Format (TOML/JSON)

```toml
# ~/.midi-cli-rs/moods/cinematic.toml
[mood]
name = "cinematic"
description = "Epic film score style with dynamic builds"
aliases = ["film", "movie", "epic"]
default_key = "Dm"
default_tempo = 90

[[layers]]
name = "strings_pad"
instrument_pool = ["strings", "tremolo_strings", "string_ensemble_1"]
register = "mid"        # low, mid, high
role = "foundation"     # foundation, melody, accent, texture
probability = 1.0       # always include
pattern = "sustained"   # sustained, arpeggiated, rhythmic, sparse

[[layers]]
name = "brass_hits"
instrument_pool = ["french_horn", "trombone", "trumpet"]
register = "mid"
role = "accent"
probability = 0.7
pattern = "sparse"
trigger = "downbeat"    # downbeat, upbeat, random

[[layers]]
name = "timpani"
instrument_pool = ["timpani", "orchestral_hit"]
register = "low"
role = "accent"
probability = 0.5
pattern = "rhythmic"

[dynamics]
build_curve = "crescendo"  # crescendo, decrescendo, wave, flat
intensity_map = { low = 0.3, mid = 0.6, high = 1.0 }

[harmony]
scale = "natural_minor"
chord_progression = ["i", "VI", "III", "VII"]  # Roman numeral
tension_intervals = [6, 11]  # tritone, major 7th
```

#### Rust Struct Serialization

```rust
#[derive(Serialize, Deserialize, Clone)]
pub struct MoodPack {
    pub mood: MoodMeta,
    pub layers: Vec<LayerDef>,
    pub dynamics: DynamicsDef,
    pub harmony: HarmonyDef,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct LayerDef {
    pub name: String,
    pub instrument_pool: Vec<String>,
    pub register: Register,
    pub role: LayerRole,
    pub probability: f64,
    pub pattern: PatternType,
    #[serde(default)]
    pub trigger: Option<TriggerType>,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum PatternType {
    Sustained,
    Arpeggiated { intervals: Vec<i8>, rhythm: Vec<f64> },
    Rhythmic { pattern: Vec<f64> },
    Sparse { density: f64 },
    Walking { style: WalkingStyle },
    Melodic { contour: Vec<i8> },
}
```

### 6.2 MoodPack Loading

```rust
// Load built-in moods
let builtin = MoodRegistry::load_builtin();

// Load user mood packs from ~/.midi-cli-rs/moods/
let user_packs = MoodRegistry::load_from_dir("~/.midi-cli-rs/moods/")?;

// CLI: list all available moods
midi-cli-rs moods --all

// CLI: use a mood pack
midi-cli-rs preset --mood cinematic -d 10 -o epic.wav

// CLI: export built-in mood as editable pack
midi-cli-rs mood-export jazz --output ~/.midi-cli-rs/moods/my-jazz.toml

// CLI: validate a mood pack
midi-cli-rs mood-validate ~/.midi-cli-rs/moods/custom.toml
```

### 6.3 Interactive Mood Editor (Web UI)

Extend the web UI to support mood pack editing:

- Visual layer editor with drag-and-drop
- Instrument pool selector with audio preview
- Pattern designer with piano roll visualization
- Real-time preview while editing
- Export to TOML/JSON for sharing

### 6.4 Mood Pack Repository

Future: Community mood pack sharing

```bash
# Install mood pack from repository
midi-cli-rs mood-install epic-orchestral

# List available mood packs
midi-cli-rs mood-search "orchestral"

# Share your mood pack
midi-cli-rs mood-publish ~/.midi-cli-rs/moods/my-mood.toml
```

### Tasks

- [ ] **6.1** Define MoodPack serialization format (TOML)
- [ ] **6.2** Implement MoodPack struct with serde
- [ ] **6.3** Create MoodRegistry for loading packs from disk
- [ ] **6.4** Implement `mood-export` CLI command
- [ ] **6.5** Implement `mood-validate` CLI command
- [ ] **6.6** Add user mood directory scanning
- [ ] **6.7** Implement Show/Musical preset (compiled)
- [ ] **6.8** Implement Orchestral family presets (compiled)
- [ ] **6.9** Web UI mood editor component
- [ ] **6.10** Mood pack repository integration

### Acceptance Criteria

- [ ] User can create custom moods via TOML files
- [ ] Built-in moods can be exported and modified
- [ ] Custom moods produce valid, musical output
- [ ] Web UI allows visual mood editing
- [ ] 10+ moods available (built-in + examples)
