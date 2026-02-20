# midi-cli-rs

CLI tool for AI coding agents to generate MIDI music programmatically.

Generate short music samples (intros, outros, background tracks) for video content using mood presets or explicit note specifications.

## Features

- **Mood Presets**: Generate complete compositions with a single command (suspense, eerie, upbeat, calm, ambient, jazz)
- **Note Control**: Specify exact notes with pitch, duration, velocity, and timing
- **Multiple Formats**: Output MIDI files or render directly to WAV audio
- **JSON Input**: Support for complex multi-track arrangements via stdin
- **Reproducible**: Use seed values for consistent output across runs
- **35+ Instruments**: Full General MIDI instrument support

## Quick Start

```bash
# Generate a 5-second suspenseful intro
midi-cli-rs preset --mood suspense --duration 5 -o intro.wav

# Generate upbeat outro with specific key
midi-cli-rs preset -m upbeat -d 7 --key C --seed 42 -o outro.wav

# Specify exact notes
midi-cli-rs generate --notes "C4:1:80,E4:0.5:100@1,G4:0.5:100@1.5" -i piano -o melody.wav

# List available options
midi-cli-rs moods       # Available mood presets
midi-cli-rs instruments # Available instruments
midi-cli-rs --help      # Full help with AI agent instructions
```

## Installation

### Prerequisites

- **Rust 2024 edition** (for building)
- **FluidSynth** (for WAV output)
  - macOS: `brew install fluid-synth`
  - Ubuntu: `apt install fluidsynth`

### Build from Source

```bash
git clone https://github.com/softwarewrighter/midi-cli-rs.git
cd midi-cli-rs
cargo build --release
```

The binary will be at `./target/release/midi-cli-rs`.

### Verify Installation

```bash
./target/release/midi-cli-rs -V
./target/release/midi-cli-rs moods
```

## Usage

### Mood Presets (Recommended)

| Mood | Default Key | Character |
|------|-------------|-----------|
| `suspense` | Am | Low drones, tremolo strings, tension |
| `eerie` | Dm | Sparse tones, diminished harmony |
| `upbeat` | C | Rhythmic chords, energetic |
| `calm` | G | Warm pads, gentle arpeggios |
| `ambient` | Em | Textural drones, pentatonic bells |
| `jazz` | F | Walking bass, piano comping, swing |

```bash
midi-cli-rs preset \
    --mood suspense \
    --duration 5 \
    --intensity 70 \
    --seed 0 \            # 0 = random, 1 = default reproducible
    --verbose \           # Show layers and note counts
    --output intro.wav
```

### Manual Note Generation

Note format: `PITCH:DURATION:VELOCITY[@OFFSET]`

- **PITCH**: Note name + octave (C4, F#3, Bb5) or MIDI number (60)
- **DURATION**: Length in beats (1.0 = quarter note)
- **VELOCITY**: Volume 0-127
- **OFFSET**: Start time in beats (optional)

```bash
# Arpeggio
midi-cli-rs generate \
    --notes "C4:0.5:80@0,E4:0.5:80@0.5,G4:0.5:80@1,C5:1:90@1.5" \
    -i piano -t 120 -o arpeggio.wav

# Chord (simultaneous notes)
midi-cli-rs generate \
    --notes "C4:2:70@0,E4:2:70@0,G4:2:70@0" \
    -i strings -o chord.wav
```

### JSON Input (Multi-Track)

```bash
echo '{"tempo":90,"instrument":"piano","notes":[
  {"pitch":"C4","duration":0.5,"velocity":80,"offset":0},
  {"pitch":"E4","duration":0.5,"velocity":80,"offset":0.5},
  {"pitch":"G4","duration":1,"velocity":90,"offset":1}
]}' | midi-cli-rs generate --json -o output.wav
```

### Post-Processing

Combine tracks with ffmpeg:

```bash
ffmpeg -i track1.wav -i track2.wav -filter_complex amix=inputs=2 combined.wav
ffmpeg -i input.wav -af "afade=t=in:d=0.5,afade=t=out:st=4:d=1" faded.wav
```

## Web UI

A browser-based interface for creating and managing music presets and melodies.

```bash
# Build and run the web server
cargo build --release
./target/release/midi-cli-rs serve  # Starts on http://127.0.0.1:3105
```

### Presets Tab
Create mood-based compositions with adjustable parameters:
- **Mood, Key, Duration, Intensity, Tempo** - Core composition settings
- **Seed** - Use 0 for random, or click the clock button to fill with ms-since-epoch for unique reproducible results

![Presets Tab](images/screenshot-presets.png?ts=1771545798000)

### Melodies Tab
Compose note-by-note with keyboard shortcuts:
- `a-g` - Set note pitch
- `r` - Rest
- `Tab` / `Shift+Tab` - Navigate notes
- `[` / `]` - Adjust duration
- `+` / `-` - Change octave
- `Esc` - Exit note editing mode

![Melodies Tab](images/screenshot-melodies.png?ts=1771545798000)

## Documentation

| Document | Description |
|----------|-------------|
| [docs/usage.md](docs/usage.md) | Comprehensive usage guide for AI agents |
| [docs/architecture.md](docs/architecture.md) | System architecture and design |
| [docs/design.md](docs/design.md) | Detailed design decisions |
| [docs/prd.md](docs/prd.md) | Product requirements |
| [docs/status.md](docs/status.md) | Project status and progress |

## Dependencies

All Rust dependencies use permissive licenses (MIT/Apache-2.0):

| Crate | Purpose |
|-------|---------|
| midly | MIDI file generation |
| clap | CLI argument parsing |
| serde | JSON serialization |
| rand | Randomization for presets |
| chrono | Timestamp formatting |

External:
- **FluidSynth** (LGPL-2.1): WAV rendering engine, called as subprocess

> **Note on FluidSynth licensing:** This tool executes FluidSynth as a separate process (`fluidsynth` command), not as a linked library. Subprocess execution does not trigger LGPL's copyleft requirements—your use of this MIT-licensed CLI tool and any audio you generate are not affected by FluidSynth's LGPL license. LGPL only applies if you modify and redistribute FluidSynth itself.

## SoundFonts

SoundFonts (.sf2 files) contain the audio samples used to render MIDI to audio. **Be aware of SoundFont licenses before using rendered audio commercially.**

### License Considerations

| SoundFont | License | Commercial Audio | Notes |
|-----------|---------|------------------|-------|
| GeneralUser GS | Permissive | Yes | Explicitly allows commercial music production |
| FluidR3_GM | MIT | Yes | Clear commercial use rights |
| MuseScore_General | MIT | Yes | High quality, larger file (~200MB) |
| TimGM6mb | GPL v2 | Unclear | Debate over whether rendered audio is "derivative" |

### Specifying a SoundFont

```bash
# Use a specific soundfont
midi-cli-rs preset -m jazz -d 5 --soundfont /path/to/FluidR3_GM.sf2 -o out.wav

# Auto-detection searches common paths:
# ./soundfonts/, /opt/homebrew/share/soundfonts/, /usr/share/sounds/sf2/
```

### Recommended SoundFonts for Commercial Use

For unambiguous commercial licensing of rendered audio:

1. **GeneralUser GS** (~30MB) - [Download](https://schristiancollins.com/generaluser.php)
   - Explicitly permits commercial music production
2. **FluidR3_GM** (~140MB) - MIT licensed
   - Available via `brew install fluid-synth` or package managers

## License

MIT License - See [LICENSE](LICENSE) for details.

Copyright (c) 2026 Michael A Wright
