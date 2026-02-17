# midi-cli-rs Usage Guide for AI Coding Agents

This guide is specifically designed for AI coding agents to generate incidental music (intros, outros, background tracks) for video content.

## Quick Reference

```bash
# Fastest approach: Use mood presets
midi-cli-rs preset --mood suspense --duration 5 -o intro.wav

# Precise control: Specify exact notes
midi-cli-rs generate --notes "C4:1:80,E4:0.5:100@1" -i piano -o melody.wav

# List available options
midi-cli-rs moods       # List mood presets
midi-cli-rs instruments # List GM instruments
midi-cli-rs --help      # Full help
```

## Mood Presets (Recommended)

Use presets for instant, professionally-designed compositions:

| Mood | Default Key | Character | Best For |
|------|-------------|-----------|----------|
| `suspense` | Am | Low drones, tremolo strings, tension | Thriller, mystery intros |
| `eerie` | Dm | Sparse tones, diminished harmony, creepy | Horror, dark content |
| `upbeat` | C | Rhythmic chords, bass, energetic | Tech reviews, tutorials |
| `calm` | G | Warm pads, gentle arpeggios | Lifestyle, meditation |
| `ambient` | Em | Textural drones, pentatonic bells | Documentary, nature |
| `jazz` | F | Walking bass, piano comping, swing | Nightclub, sophisticated |

### Preset Parameters

```bash
midi-cli-rs preset \
    --mood suspense \      # Required: suspense|eerie|upbeat|calm|ambient|jazz
    --duration 5 \         # Seconds (default: 5, typical: 3-15)
    --key Am \             # Optional: C|Cm|D|Dm|Eb|E|Em|F|Fm|G|Gm|A|Am|Bb|B|Bm
    --intensity 70 \       # 0-100: affects layering (default: 50)
    --tempo 90 \           # BPM (default: 90)
    --seed 1 \             # Default: 1 (reproducible), use 0 for random
    --verbose \            # Show generation details
    --output intro.wav     # .wav or .mid
```

### Seed Behavior

```bash
# Default (seed=1): Same output every time - reproducible
midi-cli-rs preset -m jazz -d 8 -o intro.wav
midi-cli-rs preset -m jazz -d 8 -o intro2.wav  # Identical to above

# Random seed (seed=0): Different output each time
midi-cli-rs preset -m jazz -d 8 --seed 0 -o take1.wav
# Output shows: Generated Jazz preset (seed: 1739587234, key: F)
#                                       ^^^ save this to replicate

# Specific seed: Exact reproduction
midi-cli-rs preset -m jazz -d 8 --seed 1739587234 -o take1-copy.wav
```

Use `--seed 0` when you want variety across similar videos, then note the seed shown in output to replicate a good result.

### Verbose Mode

Use `-v/--verbose` to see detailed generation info:

```bash
midi-cli-rs preset -m jazz -d 5 --seed 0 -v -o test.mid
```

Output:
```
--- Preset Generation Details ---
Mood: Jazz
Key: F (root MIDI note: 65)
Duration: 5.0s (7.5 beats at 90 BPM)
Intensity: 50/100
Seed: 1771297485 (random)
Layers: 4
  Layer 1: 8 notes, instrument 32 (acoustic bass)
  Layer 2: 6 notes, instrument 4 (electric_piano)
  Layer 3: 15 notes, instrument 14 (tubular_bells)
  Layer 4: 4 notes, instrument 115 (woodblock)
---------------------------------
```

Useful for debugging, understanding layer composition, and tuning intensity levels.

## Manual Note Generation

For precise control over every note:

### Note Format

```
PITCH:DURATION:VELOCITY[@OFFSET]
```

| Component | Description | Examples |
|-----------|-------------|----------|
| PITCH | Note name + octave, or MIDI number | `C4`, `F#3`, `Bb5`, `60` |
| DURATION | Length in beats | `1.0` = quarter note, `0.5` = eighth |
| VELOCITY | Volume 0-127 | `80` = normal, `100+` = accented |
| OFFSET | Start time in beats (optional) | `@0`, `@1.5`, `@2` |

### Examples

```bash
# Simple melody (notes play sequentially)
midi-cli-rs generate \
    --notes "C4:0.5:80,D4:0.5:70,E4:0.5:80,G4:1:90" \
    -i piano -t 120 -o melody.wav

# Chord (same offset = simultaneous)
midi-cli-rs generate \
    --notes "C4:2:70@0,E4:2:70@0,G4:2:70@0" \
    -i strings -t 80 -o chord.wav

# Arpeggio (staggered offsets)
midi-cli-rs generate \
    --notes "C4:0.5:80@0,E4:0.5:80@0.5,G4:0.5:80@1,C5:1:90@1.5" \
    -i piano -t 100 -o arpeggio.wav

# Bass line
midi-cli-rs generate \
    --notes "C2:1:100@0,G2:0.5:80@1,C2:0.5:90@1.5,E2:1:100@2" \
    -i bass -t 100 -o bassline.wav
```

## JSON Input (Multi-Track)

For complex arrangements, use JSON via stdin:

```bash
cat <<'EOF' | midi-cli-rs generate --json -o complex.wav
{
  "tracks": [
    {
      "tempo": 90,
      "instrument": "strings",
      "notes": [
        {"pitch": "C3", "duration": 4, "velocity": 50, "offset": 0},
        {"pitch": "G3", "duration": 4, "velocity": 50, "offset": 0}
      ]
    },
    {
      "tempo": 90,
      "instrument": "piano",
      "notes": [
        {"pitch": "C4", "duration": 0.5, "velocity": 80, "offset": 0},
        {"pitch": "E4", "duration": 0.5, "velocity": 80, "offset": 0.5},
        {"pitch": "G4", "duration": 0.5, "velocity": 80, "offset": 1}
      ]
    }
  ]
}
EOF
```

### Single-Track JSON

```bash
echo '{"tempo":120,"instrument":"piano","notes":[
  {"pitch":"C4","duration":0.5,"velocity":80,"offset":0},
  {"pitch":"E4","duration":0.5,"velocity":80,"offset":0.5},
  {"pitch":"G4","duration":1,"velocity":90,"offset":1}
]}' | midi-cli-rs generate --json -o simple.wav
```

## Common Instruments

| Name | GM# | Character |
|------|-----|-----------|
| `piano` | 0 | Bright, percussive |
| `strings` | 48 | Warm, sustained |
| `cello` | 42 | Deep, rich |
| `bass` | 33 | Electric bass |
| `flute` | 73 | Airy, melodic |
| `vibraphone` | 11 | Bell-like, ambient |
| `pad` | 89 | Warm synth pad |
| `choir` | 52 | Ethereal voices |

Use `midi-cli-rs instruments` for the complete list.

## Post-Processing with External Tools

Generate separate tracks and combine:

```bash
# Generate layers
midi-cli-rs preset -m ambient -d 10 --intensity 30 -o bg.wav
midi-cli-rs generate --notes "G5:2:50@2,A5:2:50@4" -i vibraphone -o accent.wav

# Mix with ffmpeg
ffmpeg -i bg.wav -i accent.wav \
    -filter_complex amix=inputs=2:duration=longest \
    -o combined.wav

# Fade in/out
ffmpeg -i combined.wav \
    -af "afade=t=in:st=0:d=0.5,afade=t=out:st=9:d=1" \
    -o final.wav

# Adjust volume
ffmpeg -i input.wav -af "volume=0.8" -o quieter.wav

# Convert to MP3
ffmpeg -i input.wav -codec:a libmp3lame -qscale:a 2 output.mp3
```

## Typical Workflows

### Video Intro (5 seconds)

```bash
# Option 1: Quick suspenseful intro
midi-cli-rs preset -m suspense -d 5 --intensity 60 -o intro.wav

# Option 2: Upbeat tech intro
midi-cli-rs preset -m upbeat -d 5 --key C --intensity 80 -o intro.wav
```

### Video Outro (7 seconds)

```bash
# Calm fadeout
midi-cli-rs preset -m calm -d 7 --key G --intensity 40 -o outro.wav
```

### Background Music (longer)

```bash
# Ambient background (15 seconds)
midi-cli-rs preset -m ambient -d 15 --intensity 30 -o background.wav

# Lower volume for voice-over
ffmpeg -i background.wav -af "volume=0.3" -o bg-quiet.wav
```

### Transition Sting (2-3 seconds)

```bash
# Quick attention-getter
midi-cli-rs generate \
    --notes "C4:0.25:100@0,E4:0.25:100@0.25,G4:0.5:110@0.5,C5:1:120@1" \
    -i piano -t 140 -o sting.wav
```

## Error Handling

| Error | Cause | Solution |
|-------|-------|----------|
| `FluidSynth not found` | Missing dependency | `brew install fluid-synth` (macOS) |
| `No SoundFont found` | Missing audio font | Use `--soundfont path/to/file.sf2` |
| `Unknown instrument` | Invalid name | Run `midi-cli-rs instruments` |
| `Unknown mood` | Invalid preset | Run `midi-cli-rs moods` |

## Version Information

```bash
midi-cli-rs -V  # Shows version, copyright, license, build info
```

## Dependencies

- **FluidSynth**: Required for WAV output
  - macOS: `brew install fluid-synth`
  - Ubuntu: `apt install fluidsynth`
- **SoundFont**: Auto-detected from common paths, or specify with `--soundfont`

## Output

- Messages go to stderr (can be suppressed with `2>/dev/null`)
- File paths are reported for generated files
- Exit code 0 on success, non-zero on error
