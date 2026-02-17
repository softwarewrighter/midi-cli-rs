# Architecture: midi-cli-rs

## System Overview

```
+------------------+     +------------------+     +------------------+
|   AI Agent       | --> |   midi-cli-rs    | --> |   Output Files   |
| (Claude Code,    |     |   (Rust CLI)     |     |   (.mid, .wav)   |
|  Gemini, etc.)   |     +------------------+     +------------------+
+------------------+              |
       |                          v
       |                  +------------------+
       |                  |   FluidSynth     |
       |                  |   (External)     |
       +----------------> +------------------+
       Post-processing           |
       (sox, ffmpeg)             v
                          +------------------+
                          |   SoundFont      |
                          |   (.sf2 file)    |
                          +------------------+
```

## Licensing Strategy

**Critical Requirement**: All output audio must be commercially usable (YouTube monetization, etc.) with no licensing encumbrances.

### Library Licenses

| Component | License | Commercial Use | Notes |
|-----------|---------|----------------|-------|
| **midly** | MIT/Apache-2.0 | Yes | Rust MIDI library, fully permissive |
| **clap** | MIT/Apache-2.0 | Yes | CLI parsing |
| **serde** | MIT/Apache-2.0 | Yes | JSON parsing |
| **FluidSynth** | LGPL-2.1 | Yes | Audio renderer; LGPL allows commercial use |
| **FluidR3_GM** SoundFont | MIT | Yes | Free SoundFont with permissive license |
| **MuseScore_General** SoundFont | MIT | Yes | Alternative free SoundFont |

### Output Ownership

- **MIDI files**: You own 100%. They're deterministic outputs from your note data.
- **WAV files**: You own 100%. FluidSynth's LGPL license doesn't affect outputs.
- **SoundFont selection**: Use MIT-licensed SoundFonts to avoid any ambiguity.

### Avoiding Licensing Issues

1. **No AI-generated music models** - Avoids training data copyright concerns
2. **No copyrighted samples** - Only algorithmic synthesis
3. **Permissive SoundFonts only** - MIT/CC0 licensed
4. **Document provenance** - Record seed, parameters for reproducibility

## Component Architecture

### Core Modules

```
src/
|-- main.rs              # CLI entry point, argument parsing
|-- lib.rs               # Public API for library use
|-- midi/
|   |-- mod.rs           # MIDI generation module
|   |-- note.rs          # Note representation (pitch, duration, velocity)
|   |-- sequence.rs      # Note sequence builder
|   |-- writer.rs        # MIDI file output using midly
|-- render/
|   |-- mod.rs           # Audio rendering module
|   |-- fluidsynth.rs    # FluidSynth integration
|   |-- soundfont.rs     # SoundFont management
|-- preset/
|   |-- mod.rs           # Mood preset module
|   |-- suspense.rs      # Suspense mood generator
|   |-- eerie.rs         # Eerie mood generator
|   |-- upbeat.rs        # Upbeat mood generator
|   |-- calm.rs          # Calm mood generator
|   |-- ambient.rs       # Ambient mood generator
|-- input/
|   |-- mod.rs           # Input parsing module
|   |-- cli.rs           # CLI argument parsing
|   |-- json.rs          # JSON input parsing
```

### Data Flow

```
1. INPUT PARSING
   CLI args / JSON stdin
          |
          v
   +-------------+
   | InputParser |  --> NoteSequence { notes: Vec<Note>, instrument, tempo }
   +-------------+

2. MIDI GENERATION
   NoteSequence
          |
          v
   +-------------+
   | MidiBuilder |  --> MidiTrack { events: Vec<MidiEvent> }
   +-------------+
          |
          v
   +-------------+
   | MidiWriter  |  --> output.mid (file)
   +-------------+

3. AUDIO RENDERING (optional)
   output.mid + SoundFont
          |
          v
   +-------------+
   | FluidSynth  |  --> output.wav (file)
   +-------------+
```

## Key Data Structures

### Note

```rust
pub struct Note {
    /// MIDI pitch (0-127, 60 = C4)
    pub pitch: u8,

    /// Duration in beats (1.0 = quarter note at given tempo)
    pub duration: f64,

    /// Velocity/volume (0-127)
    pub velocity: u8,

    /// Start time in beats from sequence start
    pub start_time: f64,
}
```

### NoteSequence

```rust
pub struct NoteSequence {
    /// Notes in the sequence
    pub notes: Vec<Note>,

    /// Instrument (GM program number 0-127)
    pub instrument: u8,

    /// MIDI channel (0-15)
    pub channel: u8,

    /// Tempo in BPM
    pub tempo: u16,
}
```

### GenerationConfig

```rust
pub struct GenerationConfig {
    /// Output file path
    pub output: PathBuf,

    /// Output format (MIDI, WAV, or both)
    pub format: OutputFormat,

    /// Random seed for reproducibility
    pub seed: Option<u64>,

    /// SoundFont path for WAV rendering
    pub soundfont: Option<PathBuf>,
}
```

## CLI Design

### Command Structure

```
midi-cli-rs [OPTIONS] <COMMAND>

Commands:
  generate     Generate MIDI from explicit notes
  preset       Generate MIDI using a mood preset
  render       Render existing MIDI to WAV
  instruments  List available instruments
  help         Show help

Global Options:
  -v, --verbose    Verbose output
  -q, --quiet      Suppress non-error output
      --version    Show version
```

### Generate Command

```bash
midi-cli-rs generate [OPTIONS]

Options:
  -n, --notes <NOTES>       Notes as "pitch:duration:velocity,..."
                            Example: "C4:1:80,E4:0.5:100,G4:0.5:100"
  -j, --json                Read JSON from stdin
  -i, --instrument <INST>   Instrument name or number (default: piano)
  -t, --tempo <BPM>         Tempo in BPM (default: 120)
  -o, --output <FILE>       Output file (.mid or .wav)
  -s, --seed <NUM>          Random seed
      --soundfont <PATH>    SoundFont for WAV rendering
```

### Preset Command

```bash
midi-cli-rs preset [OPTIONS]

Options:
  -m, --mood <MOOD>         Mood: suspense, eerie, upbeat, calm, ambient
  -d, --duration <SECS>     Duration in seconds (default: 5)
  -k, --key <KEY>           Musical key (default: Am for minor, C for major)
      --intensity <0-100>   Energy level (default: 50)
  -o, --output <FILE>       Output file
  -s, --seed <NUM>          Random seed
      --soundfont <PATH>    SoundFont for WAV rendering
```

## FluidSynth Integration

### Approach: External Process

FluidSynth will be invoked as an external process rather than linked as a library:

**Advantages**:
- Simpler build (no C library linking)
- LGPL isolation (clearer license boundary)
- Easier cross-platform support
- User can install FluidSynth via package manager

**Implementation**:
```rust
fn render_to_wav(midi_path: &Path, wav_path: &Path, soundfont: &Path) -> Result<()> {
    Command::new("fluidsynth")
        .args([
            "-ni",                        // Non-interactive, no MIDI input
            soundfont.to_str().unwrap(),  // SoundFont file
            midi_path.to_str().unwrap(),  // Input MIDI
            "-F", wav_path.to_str().unwrap(),  // Output WAV
            "-r", "44100",                // Sample rate
        ])
        .status()?;
    Ok(())
}
```

### SoundFont Strategy

1. **Default**: Expect FluidR3_GM or MuseScore_General in standard locations
2. **Custom**: Allow `--soundfont` flag for user-specified SoundFont
3. **Bundle option**: Future consideration to bundle a minimal SoundFont

Standard locations checked:
- `/usr/share/sounds/sf2/`
- `/usr/share/soundfonts/`
- `~/.local/share/soundfonts/`
- `./soundfonts/`

## Mood Preset Architecture

Each mood preset implements a trait:

```rust
pub trait MoodGenerator {
    /// Generate a note sequence for this mood
    fn generate(
        &self,
        duration_secs: f64,
        key: Key,
        intensity: u8,
        seed: u64,
    ) -> Vec<NoteSequence>;

    /// Suggested instruments for this mood
    fn default_instruments(&self) -> Vec<Instrument>;
}
```

### Suspense Preset Example

```rust
impl MoodGenerator for SuspenseGenerator {
    fn generate(&self, duration: f64, key: Key, intensity: u8, seed: u64) -> Vec<NoteSequence> {
        let mut rng = StdRng::seed_from_u64(seed);
        let mut sequences = vec![];

        // Low drone on cello
        sequences.push(self.generate_drone(duration, key, &mut rng));

        // High tremolo on violins
        sequences.push(self.generate_tremolo(duration, key, intensity, &mut rng));

        // Optional: sparse piano hits
        if intensity > 50 {
            sequences.push(self.generate_hits(duration, key, &mut rng));
        }

        sequences
    }
}
```

## Error Handling

```rust
#[derive(Debug, thiserror::Error)]
pub enum MidiCliError {
    #[error("Invalid note format: {0}")]
    InvalidNote(String),

    #[error("FluidSynth not found. Install with: brew install fluid-synth")]
    FluidSynthNotFound,

    #[error("SoundFont not found: {0}")]
    SoundFontNotFound(PathBuf),

    #[error("Invalid instrument: {0}")]
    InvalidInstrument(String),

    #[error("MIDI write error: {0}")]
    MidiWriteError(#[from] midly::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}
```

## Testing Strategy

### Unit Tests
- Note parsing: `"C4:1:80"` -> Note { pitch: 60, duration: 1.0, velocity: 80 }
- Sequence building
- Preset generation (deterministic with seed)

### Integration Tests
- Full CLI invocation
- MIDI file validity (parse generated files)
- WAV rendering (if FluidSynth available)

### Example Test

```rust
#[test]
fn test_note_parsing() {
    let note = Note::parse("C4:1:80").unwrap();
    assert_eq!(note.pitch, 60);
    assert_eq!(note.duration, 1.0);
    assert_eq!(note.velocity, 80);
}

#[test]
fn test_suspense_deterministic() {
    let gen = SuspenseGenerator::new();
    let seq1 = gen.generate(5.0, Key::Am, 50, 42);
    let seq2 = gen.generate(5.0, Key::Am, 50, 42);
    assert_eq!(seq1, seq2);  // Same seed = same output
}
```

## Performance Considerations

- **MIDI generation**: Pure computation, sub-millisecond for typical sequences
- **File I/O**: Write MIDI directly to file, no intermediate buffering needed
- **FluidSynth**: External process, ~100-500ms for 5-second clips
- **Memory**: Keep note sequences in memory; typical sequences < 1KB

## Future Extensions

1. **MIDI input transformation**: Read existing MIDI, apply transformations
2. **More presets**: Jazzy, new-age, dramatic, playful
3. **Layering hints**: Output metadata for AI to know how to combine tracks
4. **Direct FluidSynth linking**: Optional feature flag for faster rendering
5. **Web API wrapper**: For remote/containerized usage
