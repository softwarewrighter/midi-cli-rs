# Product Requirements Document: midi-cli-rs

## Overview

**midi-cli-rs** is a Rust command-line tool designed for AI coding agents to programmatically generate short music samples (5-15 seconds) for video intro/outro soundtracks. The tool outputs MIDI files and optionally renders them to WAV audio using FluidSynth.

## Problem Statement

AI coding agents need to create mood-appropriate background music for short videos. Current solutions either:
- Require complex AI model infrastructure (GPU, licensing concerns)
- Lack programmatic control suitable for automation
- Don't support batch/scriptable workflows

## Target Users

**Primary**: AI coding agents (Claude Code, Gemini CLI, Codex, etc.)
**Secondary**: Human developers automating music generation pipelines

## Goals

1. **Automatable**: Fully controllable via CLI arguments or JSON input
2. **Deterministic**: Same inputs produce identical outputs (seed-based)
3. **License-safe**: No AI-generated content licensing concerns
4. **Composable**: Single-instrument output that can be layered in post-processing
5. **Fast**: Generate clips in milliseconds, not minutes

## Non-Goals

- Full DAW functionality
- Real-time playback
- Multi-track mixing (handled by external tools like sox/ffmpeg)
- Complex audio effects (reverb, compression - use post-processing)

## Features

### Core Features (MVP)

#### F1: Note-Level MIDI Generation
- Accept notes via CLI arguments: `--notes "C4:0.5:80,E4:0.25:100"`
- Accept notes via JSON stdin for complex sequences
- Control pitch, duration, velocity, timing
- Output standard MIDI files

#### F2: Instrument Selection
- Support General MIDI instrument programs (0-127)
- Named instrument shortcuts: `--instrument piano`, `--instrument strings`
- Multiple channels for layering

#### F3: WAV Rendering
- Integrate FluidSynth for MIDI-to-WAV rendering
- Support custom SoundFont files
- Include a default free SoundFont (FluidR3_GM or similar)

#### F4: Mood Presets
Built-in algorithmic generators for common moods:
- `suspense`: Minor key, tremolo strings, low drones
- `eerie`: Dissonant intervals, sparse, reverberant
- `upbeat`: Major key, faster tempo, rhythmic
- `calm`: Slow, consonant, pad-like
- `ambient`: Drones, textures, minimal melody

#### F5: Deterministic Output
- Seed parameter for reproducible random elements
- Document exact parameters for regeneration

### Future Features

- **Multi-track melody composition**: Web UI support for composing melodies with multiple tracks/instruments that combine into a single output file
- **Time signature control**: Support for 3/4 (waltz), 4/4 (standard), 6/8, and other time signatures
- MIDI file input for transformation (reverse, invert, transpose)
- Additional mood presets
- Tempo/key modulation over time
- Integration with public domain melody sources

## Technical Requirements

### T1: Input Formats

**CLI Arguments** (simple cases):
```bash
midi-cli-rs generate \
  --notes "C4:0.5:80,E4:0.25:100,G4:0.25:100" \
  --instrument piano \
  --tempo 120 \
  --output intro.mid
```

**JSON stdin** (complex sequences):
```bash
echo '{"notes":[...], "instrument": "strings"}' | midi-cli-rs generate --json -
```

### T2: Output Formats

- `.mid` - Standard MIDI file (Type 0 or Type 1)
- `.wav` - PCM audio rendered via FluidSynth
- Both formats can be output simultaneously

### T3: Performance

- Generate 5-second MIDI clip: < 10ms
- Render 5-second WAV (with FluidSynth): < 500ms
- Memory usage: < 50MB

### T4: Dependencies

- **midly**: MIDI file parsing/writing (Rust native)
- **FluidSynth**: Audio rendering (external binary or library binding)
- **clap**: CLI argument parsing
- **serde**: JSON parsing

## Command Structure

```
midi-cli-rs <COMMAND>

Commands:
  generate    Generate MIDI/audio from notes or presets
  preset      Generate audio using a mood preset
  instruments List available instruments
  info        Display MIDI file information
  help        Show help

Generate Options:
  --notes <NOTES>       Note sequence (pitch:duration:velocity,...)
  --json                Read note data from JSON stdin
  --instrument <NAME>   Instrument name or GM program number
  --tempo <BPM>         Tempo in beats per minute (default: 120)
  --duration <SECS>     Total duration in seconds
  --seed <NUM>          Random seed for reproducibility
  --output <FILE>       Output file path (.mid or .wav)
  --soundfont <PATH>    Custom SoundFont file for WAV rendering

Preset Options:
  --mood <MOOD>         Mood preset (suspense, eerie, upbeat, calm, ambient)
  --duration <SECS>     Duration in seconds (default: 5)
  --intensity <0-100>   Intensity/energy level (default: 50)
  --key <KEY>           Musical key (C, Cm, D, Dm, etc.)
```

## Example Workflows

### AI Agent: Generate Suspenseful Intro

```bash
# Generate 5-second suspense stinger
midi-cli-rs preset --mood suspense --duration 5 --output suspense.wav

# Or with full control
midi-cli-rs generate \
  --notes "E2:4:40,B2:4:40" \  # Low drone
  --instrument "strings" \
  --tempo 60 \
  --output drone.mid

midi-cli-rs generate \
  --notes "E5:0.1:30,F5:0.1:30,E5:0.1:30" \  # High tremolo
  --instrument "violin" \
  --output tremolo.mid

# Combine with ffmpeg
ffmpeg -i drone.wav -i tremolo.wav -filter_complex amix=inputs=2 suspense.wav
```

### AI Agent: Generate Upbeat Outro

```bash
midi-cli-rs preset --mood upbeat --duration 7 --key C --output outro.wav
```

## Success Metrics

1. AI agents can generate appropriate intro/outro music without human intervention
2. Generated clips are indistinguishable from stock music in quality
3. Full pipeline (generate + render) completes in < 2 seconds
4. Zero licensing concerns for commercial use

## Open Questions

1. Should we bundle a SoundFont or require external installation?
2. Should FluidSynth be a hard dependency or optional?
3. What's the minimum set of mood presets for MVP?

## Timeline

- Phase 1 (MVP): Note-level generation + MIDI output
- Phase 2: WAV rendering with FluidSynth
- Phase 3: Mood presets
- Phase 4: Refinement and additional presets
