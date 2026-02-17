# Implementation Plan: midi-cli-rs

## Phase Overview

| Phase | Focus | Deliverables | Status |
|-------|-------|--------------|--------|
| 1 | Core MIDI Generation | Note parsing, MIDI output | Planned |
| 2 | CLI Interface | clap-based CLI, JSON input | Planned |
| 3 | WAV Rendering | FluidSynth integration | Planned |
| 4 | Mood Presets | Suspense, eerie, upbeat, calm, ambient | Planned |
| 5 | Polish | Error handling, docs, tests | Planned |

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

- [ ] AI agent can generate 5-second intro music with single CLI call
- [ ] Generation + rendering completes in < 2 seconds
- [ ] Zero licensing concerns for commercial use
- [ ] 5 distinct mood presets available
