# Project Status: midi-cli-rs

**Last Updated**: 2026-02-16

## Current Phase

**Phase 1: Core Implementation** - COMPLETE

## Overall Progress

```
[########--] 80% Complete
```

| Phase | Status | Progress |
|-------|--------|----------|
| Phase 0: Documentation | Complete | 100% |
| Phase 1: Core MIDI Generation | Complete | 100% |
| Phase 2: CLI Interface | Complete | 100% |
| Phase 3: WAV Rendering | Complete | 100% |
| Phase 4: Mood Presets | Not Started | 0% |
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

### Tests

- 39 unit tests passing
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

### Phase 4: Mood Presets

1. Implement MoodGenerator trait
2. Implement preset generators:
   - Suspense (minor key, drones, tremolo)
   - Eerie (sparse, dissonant, wide intervals)
   - Upbeat (major key, rhythmic)
   - Calm (sustained pads, arpeggios)
   - Ambient (drones, textures)
3. Add `preset` command

### Phase 5: Polish

1. Comprehensive error messages
2. README with full documentation
3. More test coverage
4. Performance optimization

## Notes for AI Agents

**Working Example**:
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
