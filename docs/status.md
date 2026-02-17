# Project Status: midi-cli-rs

**Last Updated**: 2026-02-16

## Current Phase

**Phase 4: Mood Presets** - COMPLETE

## Overall Progress

```
[#########-] 90% Complete
```

| Phase | Status | Progress |
|-------|--------|----------|
| Phase 0: Documentation | Complete | 100% |
| Phase 1: Core MIDI Generation | Complete | 100% |
| Phase 2: CLI Interface | Complete | 100% |
| Phase 3: WAV Rendering | Complete | 100% |
| Phase 4: Mood Presets | Complete | 100% |
| Phase 5: Polish | Not Started | 0% |

## Completed Work

### Phase 0: Documentation (2026-02-16)

- [x] Created docs/prd.md - Product requirements document
- [x] Created docs/architecture.md - System architecture with licensing analysis
- [x] Created docs/design.md - Detailed design decisions
- [x] Created docs/plan.md - Implementation plan with phases
- [x] Created docs/status.md - This file

### Phase 1-3: Core Implementation (2026-02-16)

- [x] Added dependencies (midly, clap, serde, thiserror, rand)
- [x] Implemented Note struct with parsing ("C4:1:80" format)
- [x] Implemented NoteSequence with instrument mapping
- [x] Implemented MIDI file writer using midly crate
- [x] Implemented CLI with clap (generate, render, instruments, info commands)
- [x] Implemented FluidSynth integration for WAV rendering
- [x] Created demo scripts (scripts/demo-generate.sh)
- [x] Created preview directory with index.html and 10 sample WAVs

### Phase 4: Mood Presets (2026-02-16)

- [x] Implemented MoodGenerator trait in src/preset/mod.rs
- [x] Implemented Key enum with musical theory (root notes, chord tones)
- [x] Implemented Mood enum with 5 presets
- [x] Created Suspense preset (low drones, tremolo strings, sparse piano)
- [x] Created Eerie preset (diminished pad, bells, breath texture)
- [x] Created Upbeat preset (rhythmic chords, bass line, melody)
- [x] Created Calm preset (major 7th pad, gentle harp arpeggio)
- [x] Created Ambient preset (layered drones, pentatonic tones)
- [x] Added `preset` CLI command with mood, duration, key, intensity, seed options
- [x] Added `moods` CLI command to list available presets

### Tests

- 58 unit tests passing (19 new preset tests)
- Zero clippy warnings
- Code formatted

## Working CLI Commands

```bash
# Generate MIDI from notes
midi-cli-rs generate --notes "C4:1:80,E4:1:80,G4:1:80" -i piano -o output.mid

# Generate WAV (includes MIDI + FluidSynth render)
midi-cli-rs generate --notes "C4:1:80,E4:0.5:100@1" -i strings -o output.wav

# JSON input
echo '{"tempo":120,"instrument":"piano","notes":[...]}' | midi-cli-rs generate --json -o out.wav

# Generate using mood preset
midi-cli-rs preset --mood suspense --duration 5 -o suspense.wav
midi-cli-rs preset -m upbeat -d 7 --key C --intensity 80 --seed 42 -o intro.wav

# List available mood presets
midi-cli-rs moods

# List instruments
midi-cli-rs instruments

# Show MIDI file info
midi-cli-rs info file.mid

# Render existing MIDI to WAV
midi-cli-rs render -i input.mid -o output.wav
```

## Demo Samples Generated

10 audio samples in `preview/` directory:
1. C Major Chord (arpeggiated) - piano
2. Simple melody - piano
3. Low drone - cello
4. String pad - strings
5. Upbeat rhythm - piano
6. Bass line - electric bass
7. Bells/chimes - vibraphone
8. Minor key (eerie) - strings
9. JSON input demo - piano
10. Flute melody - flute

Open `preview/index.html` in a browser to listen.

## Dependencies

### Rust Crates (all MIT/Apache-2.0)

| Crate | Purpose |
|-------|---------|
| midly | MIDI generation |
| clap | CLI parsing |
| serde/serde_json | JSON input |
| thiserror | Error types |
| rand | Randomization (for presets) |
| tempfile | Test fixtures |

### External

| Tool | Purpose | Status |
|------|---------|--------|
| FluidSynth | WAV rendering | Required for WAV output |
| TimGM6mb.sf2 | SoundFont | Bundled in soundfonts/ |

## Next Steps

### Phase 5: Polish

1. Comprehensive error messages
2. README with full documentation
3. More test coverage
4. Performance optimization
5. Update demo scripts with preset examples

## Notes for AI Agents

**Using Mood Presets** (recommended for quick results):
```bash
# Generate a 5-second suspenseful intro
midi-cli-rs preset --mood suspense --duration 5 -o intro.wav

# Generate upbeat outro with specific key and seed for reproducibility
midi-cli-rs preset -m upbeat -d 7 --key C --intensity 80 --seed 42 -o outro.wav

# Available moods: suspense, eerie, upbeat, calm, ambient
midi-cli-rs moods  # List all presets with descriptions
```

**Manual Note Control**:
```bash
# Generate a 3-second piano intro
midi-cli-rs generate \
    --notes "C4:0.5:80@0,E4:0.5:80@0.5,G4:0.5:80@1,C5:1.5:90@1.5" \
    --instrument piano \
    --tempo 120 \
    --output intro.wav
```

**Note Format**: `PITCH:DURATION:VELOCITY[@OFFSET]`
- PITCH: C4, F#3, Bb5 (note name + octave)
- DURATION: beats (1.0 = quarter note)
- VELOCITY: 0-127
- OFFSET: start time in beats (optional)

**Combining Tracks**: Generate separate WAV files, combine with ffmpeg:
```bash
ffmpeg -i track1.wav -i track2.wav -filter_complex amix=inputs=2 combined.wav
```
