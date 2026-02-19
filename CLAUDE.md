# Claude Code Project Instructions

## Audio Playback

**Always use `afplay` for playing audio files on macOS**, not `open`.

- `afplay` is lightweight and plays audio directly in the terminal
- `open` launches iTunes/Music.app which is heavy and slow

```bash
# Correct
afplay /path/to/audio.wav

# Incorrect - do not use
open /path/to/audio.wav
```

## Project: midi-cli-rs

A Rust CLI tool for AI coding agents to generate MIDI music with mood presets.

### Key Commands

```bash
# Generate audio with a mood preset
./target/release/midi-cli-rs preset -m jazz -d 5 --seed 42 -o output.wav

# Show verbose generation info
./target/release/midi-cli-rs preset -m ambient -d 5 --seed 1 -o output.wav -v

# List available moods
./target/release/midi-cli-rs moods
```

### Moods

suspense, eerie, upbeat, calm, ambient, jazz

### Seed Behavior

- `--seed 1` (default): Reproducible output
- `--seed 0`: Random seed each time
- `--seed N`: Specific seed for exact reproduction
