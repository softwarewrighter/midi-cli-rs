# Design Document: midi-cli-rs

## Design Principles

1. **AI-First Interface**: Every feature must be usable via CLI without human interaction
2. **Determinism**: Same inputs always produce identical outputs
3. **Composability**: Single-instrument output encourages layering via external tools
4. **Fail Fast**: Clear error messages with actionable remediation
5. **Zero Config**: Sensible defaults; configuration only when needed

## Note Representation

### Note String Format

Format: `PITCH:DURATION:VELOCITY[@OFFSET]`

| Component | Required | Description | Example |
|-----------|----------|-------------|---------|
| PITCH | Yes | Note name + octave | C4, F#3, Bb5 |
| DURATION | Yes | Duration in beats | 0.5, 1, 2.0 |
| VELOCITY | Yes | Volume 0-127 | 80, 100 |
| OFFSET | No | Start time in beats | @0, @2.5 |

**Examples**:
```
C4:1:80              # C4, 1 beat, velocity 80, starts at 0
E4:0.5:100@1         # E4, half beat, velocity 100, starts at beat 1
F#3:2:60@0           # F#3, 2 beats, velocity 60, starts at beat 0
Bb5:0.25:90@1.5      # Bb5, quarter beat, velocity 90, starts at beat 1.5
```

### Pitch Notation

```
Note names: C, D, E, F, G, A, B
Accidentals: # (sharp), b (flat)
Octaves: 0-10 (C4 = middle C = MIDI 60)

Conversion: MIDI = (octave + 1) * 12 + semitone
  C4 = (4 + 1) * 12 + 0 = 60
  A4 = (4 + 1) * 12 + 9 = 69 (440 Hz reference)
```

### JSON Input Format

```json
{
  "tempo": 120,
  "instrument": "piano",
  "channel": 0,
  "notes": [
    {"pitch": "C4", "duration": 1.0, "velocity": 80, "offset": 0.0},
    {"pitch": "E4", "duration": 0.5, "velocity": 100, "offset": 1.0},
    {"pitch": "G4", "duration": 0.5, "velocity": 100, "offset": 1.5}
  ]
}
```

Or with multiple tracks:

```json
{
  "tracks": [
    {
      "instrument": "strings",
      "channel": 0,
      "notes": [...]
    },
    {
      "instrument": "piano",
      "channel": 1,
      "notes": [...]
    }
  ],
  "tempo": 90
}
```

## Instrument Mapping

### Named Instruments to GM Program Numbers

```rust
const INSTRUMENT_MAP: &[(&str, u8)] = &[
    // Pianos
    ("piano", 0),
    ("acoustic_piano", 0),
    ("bright_piano", 1),
    ("electric_piano", 4),
    ("honky_tonk", 3),

    // Strings
    ("strings", 48),
    ("violin", 40),
    ("viola", 41),
    ("cello", 42),
    ("contrabass", 43),
    ("tremolo_strings", 44),
    ("pizzicato_strings", 45),
    ("harp", 46),

    // Woodwinds
    ("flute", 73),
    ("oboe", 68),
    ("clarinet", 71),
    ("bassoon", 70),

    // Brass
    ("trumpet", 56),
    ("trombone", 57),
    ("french_horn", 60),
    ("tuba", 58),

    // Synth
    ("synth_pad", 88),
    ("synth_lead", 80),
    ("pad_warm", 89),
    ("pad_choir", 91),

    // Percussion (channel 10)
    ("drums", 0),  // Special: uses channel 10

    // Ambient
    ("atmosphere", 99),
    ("rain", 96),
    ("soundtrack", 97),
];
```

## Mood Preset Algorithms

### Suspense

**Characteristics**: Minor key, low register, slow movement, tremolo, dissonance

**Algorithm**:
```
1. Base: Sustained low drone (root + fifth)
   - Instrument: Cello or Contrabass
   - Notes: Root (2 octaves below middle C), Fifth
   - Duration: Full clip length
   - Velocity: 40-60 (quiet but present)

2. Layer: High tremolo
   - Instrument: Tremolo Strings
   - Notes: Minor second intervals (creates tension)
   - Pattern: Rapid alternation (16th notes)
   - Velocity: 20-40 (subtle)

3. Optional (intensity > 60): Piano stabs
   - Instrument: Piano
   - Notes: Dissonant cluster (root + minor 2nd + tritone)
   - Timing: Sparse, unpredictable (using seeded RNG)
   - Velocity: 60-90 (accent)
```

### Eerie

**Characteristics**: Sparse, wide intervals, reverb-friendly, minor/diminished

**Algorithm**:
```
1. Base: Sustained pad
   - Instrument: Pad (Warm) or Atmosphere
   - Notes: Diminished chord, spread across octaves
   - Duration: Full clip, with slow fade-in

2. Layer: High bell tones
   - Instrument: Celesta or Glockenspiel
   - Notes: Single notes from diminished scale
   - Timing: Sparse (1-3 notes total)
   - Velocity: 30-50

3. Optional: Wind/breath sounds
   - Instrument: Breath Noise (#121) or similar
   - Notes: Chromatic movement
   - Very low velocity
```

### Upbeat

**Characteristics**: Major key, rhythmic, higher register, clear pulse

**Algorithm**:
```
1. Base: Rhythmic chord pattern
   - Instrument: Acoustic Piano
   - Notes: Major triad, rhythmic pattern (e.g., 1-and-2-and-3-and-4)
   - Velocity: 70-90

2. Layer: Bass line
   - Instrument: Acoustic Bass or Electric Bass
   - Notes: Root-fifth pattern, follow chord changes
   - Duration: Quarter notes

3. Optional (intensity > 50): Melody hint
   - Instrument: Synth Lead or Brass
   - Notes: Major scale run or arpeggio
   - Timing: 2-4 notes total
```

### Calm

**Characteristics**: Major or modal, slow, consonant, pad-like

**Algorithm**:
```
1. Base: Sustained pad chord
   - Instrument: Pad (Warm) or Strings
   - Notes: Major 7th chord, open voicing
   - Duration: Full clip with slow attack/release

2. Layer: Gentle arpeggio
   - Instrument: Harp or Acoustic Guitar (nylon)
   - Notes: Chord tones, ascending pattern
   - Velocity: 40-60
   - Timing: Slow, even spacing
```

### Ambient

**Characteristics**: Textural, non-rhythmic, drones, evolving

**Algorithm**:
```
1. Base: Multi-layered drone
   - Instrument: Pad (Warm) + Strings
   - Notes: Root + fifth, spread across 2 octaves
   - Duration: Full clip
   - Velocity: Slowly evolving (random walk)

2. Layer: Sporadic tones
   - Instrument: Vibraphone or Bells
   - Notes: Pentatonic scale
   - Timing: Random, sparse (seeded)
   - Velocity: 20-40
```

## MIDI File Structure

Using Standard MIDI File Format 1 (multiple tracks):

```
Header Chunk:
  - Format: 1 (multiple tracks, synchronous)
  - Tracks: 1 + number of instruments
  - Division: 480 ticks per quarter note

Track 0 (Tempo/Meta):
  - Tempo event
  - Time signature (4/4)
  - End of track

Track 1-N (Instruments):
  - Program change (instrument selection)
  - Note on/off events
  - End of track
```

### MIDI Event Timing

```rust
fn beats_to_ticks(beats: f64, ticks_per_beat: u16) -> u32 {
    (beats * ticks_per_beat as f64) as u32
}

// Example: 0.5 beats at 480 ticks/beat = 240 ticks
```

## FluidSynth Integration Details

### Command Construction

```rust
fn build_fluidsynth_command(
    midi_path: &Path,
    wav_path: &Path,
    soundfont: &Path,
) -> Command {
    let mut cmd = Command::new("fluidsynth");
    cmd.args([
        "-ni",                              // Non-interactive
        "-g", "1.0",                        // Gain (volume)
        "-r", "44100",                      // Sample rate
        soundfont.to_str().unwrap(),        // SoundFont
        midi_path.to_str().unwrap(),        // Input MIDI
        "-F", wav_path.to_str().unwrap(),   // Output file
    ]);
    cmd
}
```

### SoundFont Discovery

```rust
fn find_soundfont() -> Result<PathBuf, MidiCliError> {
    let search_paths = [
        // Linux paths
        PathBuf::from("/usr/share/sounds/sf2/FluidR3_GM.sf2"),
        PathBuf::from("/usr/share/soundfonts/FluidR3_GM.sf2"),
        PathBuf::from("/usr/share/soundfonts/default.sf2"),
        // macOS (Homebrew)
        PathBuf::from("/opt/homebrew/share/soundfonts/default.sf2"),
        PathBuf::from("/usr/local/share/soundfonts/default.sf2"),
        // User local
        dirs::data_dir().unwrap().join("soundfonts/default.sf2"),
        // Project local
        PathBuf::from("./soundfonts/default.sf2"),
    ];

    for path in &search_paths {
        if path.exists() {
            return Ok(path.clone());
        }
    }

    Err(MidiCliError::SoundFontNotFound(
        "No SoundFont found. Install FluidR3_GM or specify --soundfont".into()
    ))
}
```

## Output File Handling

### Automatic Format Detection

```rust
fn determine_output_format(path: &Path) -> OutputFormat {
    match path.extension().and_then(|s| s.to_str()) {
        Some("mid") | Some("midi") => OutputFormat::Midi,
        Some("wav") => OutputFormat::Wav,
        Some("both") => OutputFormat::Both,
        _ => OutputFormat::Midi,  // Default
    }
}
```

### Dual Output

When both formats requested:
```bash
midi-cli-rs generate --notes "..." --output intro.wav
# Produces:
#   intro.mid  (intermediate, kept for reference)
#   intro.wav  (final rendered audio)
```

## Randomization with Reproducibility

All random elements use a seeded RNG:

```rust
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

pub struct PresetContext {
    rng: StdRng,
    seed: u64,
}

impl PresetContext {
    pub fn new(seed: u64) -> Self {
        Self {
            rng: StdRng::seed_from_u64(seed),
            seed,
        }
    }

    /// Generate timing variation (+/- range in beats)
    pub fn timing_variation(&mut self, range: f64) -> f64 {
        self.rng.gen_range(-range..range)
    }

    /// Generate velocity variation (+/- range)
    pub fn velocity_variation(&mut self, base: u8, range: u8) -> u8 {
        let delta = self.rng.gen_range(-(range as i16)..=(range as i16));
        (base as i16 + delta).clamp(1, 127) as u8
    }
}
```

**AI Usage**: Always specify `--seed` for reproducible results:
```bash
# Generate, review, regenerate identical output
midi-cli-rs preset --mood suspense --seed 12345 --output test.wav

# Document seed in video project metadata for future regeneration
```

## Error Messages for AI Agents

Errors are formatted for easy parsing by AI:

```rust
impl MidiCliError {
    pub fn format_for_agent(&self) -> String {
        match self {
            MidiCliError::InvalidNote(s) => format!(
                "ERROR: Invalid note format '{}'\n\
                 EXPECTED: PITCH:DURATION:VELOCITY (e.g., C4:1:80)\n\
                 FIX: Check pitch name, ensure numeric duration and velocity",
                s
            ),
            MidiCliError::FluidSynthNotFound => format!(
                "ERROR: FluidSynth not installed\n\
                 FIX: Install FluidSynth:\n\
                   macOS: brew install fluid-synth\n\
                   Ubuntu: apt install fluidsynth\n\
                   Fedora: dnf install fluidsynth"
            ),
            // ... other errors
        }
    }
}
```

## Configuration File (Future)

Optional `~/.config/midi-cli-rs/config.toml`:

```toml
[defaults]
tempo = 120
soundfont = "/path/to/preferred.sf2"
output_format = "wav"

[instruments]
# Custom instrument aliases
warm_strings = 48
dark_piano = 1
```

## Validation Rules

### Note Validation

```rust
fn validate_note(note: &Note) -> Result<(), MidiCliError> {
    // Pitch in valid MIDI range
    if note.pitch > 127 {
        return Err(MidiCliError::InvalidNote(
            format!("Pitch {} out of range (0-127)", note.pitch)
        ));
    }

    // Duration positive
    if note.duration <= 0.0 {
        return Err(MidiCliError::InvalidNote(
            "Duration must be positive".into()
        ));
    }

    // Velocity in range
    if note.velocity > 127 {
        return Err(MidiCliError::InvalidNote(
            format!("Velocity {} out of range (0-127)", note.velocity)
        ));
    }

    Ok(())
}
```

### Sequence Validation

```rust
fn validate_sequence(seq: &NoteSequence) -> Result<(), MidiCliError> {
    // At least one note
    if seq.notes.is_empty() {
        return Err(MidiCliError::EmptySequence);
    }

    // Tempo reasonable
    if seq.tempo < 20 || seq.tempo > 300 {
        return Err(MidiCliError::InvalidTempo(seq.tempo));
    }

    // Validate each note
    for note in &seq.notes {
        validate_note(note)?;
    }

    Ok(())
}
```

## Example: Full Generation Pipeline

```rust
pub fn generate_and_render(
    notes: &str,
    instrument: &str,
    tempo: u16,
    output: &Path,
    soundfont: Option<&Path>,
    seed: Option<u64>,
) -> Result<(), MidiCliError> {
    // 1. Parse notes
    let parsed_notes = parse_note_string(notes)?;

    // 2. Build sequence
    let sequence = NoteSequence {
        notes: parsed_notes,
        instrument: resolve_instrument(instrument)?,
        channel: 0,
        tempo,
    };

    // 3. Validate
    validate_sequence(&sequence)?;

    // 4. Generate MIDI
    let midi_path = if output.extension() == Some("wav".as_ref()) {
        output.with_extension("mid")
    } else {
        output.to_path_buf()
    };
    write_midi(&sequence, &midi_path)?;

    // 5. Render to WAV if requested
    if output.extension() == Some("wav".as_ref()) {
        let sf = soundfont
            .map(PathBuf::from)
            .or_else(|| find_soundfont().ok())
            .ok_or(MidiCliError::SoundFontNotFound("".into()))?;

        render_wav(&midi_path, output, &sf)?;
    }

    Ok(())
}
```
