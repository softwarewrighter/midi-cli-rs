//! midi-cli-rs: CLI tool for AI agents to generate MIDI music
//!
//! Generate MIDI files and WAV audio from note specifications or mood presets.

use clap::{Parser, Subcommand};
use midi_cli_rs::{
    JsonSequenceInput, Key, Mood, Note, NoteSequence, PresetConfig, generate_mood,
    resolve_instrument, write_midi,
};
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

/// CLI tool for AI agents to generate MIDI music programmatically
#[derive(Parser)]
#[command(name = "midi-cli-rs")]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate MIDI/audio from explicit notes
    Generate {
        /// Notes as "PITCH:DURATION:VELOCITY[@OFFSET],..."
        /// Example: "C4:1:80,E4:0.5:100@1,G4:0.5:100@1.5"
        #[arg(short, long)]
        notes: Option<String>,

        /// Read JSON note data from stdin
        #[arg(short, long)]
        json: bool,

        /// Instrument name or GM program number (0-127)
        #[arg(short, long, default_value = "piano")]
        instrument: String,

        /// Tempo in BPM
        #[arg(short, long, default_value = "120")]
        tempo: u16,

        /// Output file path (.mid or .wav)
        #[arg(short, long)]
        output: PathBuf,

        /// SoundFont file for WAV rendering
        #[arg(long)]
        soundfont: Option<PathBuf>,
    },

    /// Generate MIDI/audio using a mood preset
    Preset {
        /// Mood preset (suspense, eerie, upbeat, calm, ambient)
        #[arg(short, long)]
        mood: String,

        /// Duration in seconds
        #[arg(short, long, default_value = "5")]
        duration: f64,

        /// Musical key (C, Cm, D, Dm, E, Em, F, Fm, G, Gm, A, Am, B, Bm)
        #[arg(short, long)]
        key: Option<String>,

        /// Intensity level (0-100)
        #[arg(long, default_value = "50")]
        intensity: u8,

        /// Tempo in BPM
        #[arg(short, long, default_value = "90")]
        tempo: u16,

        /// Random seed for reproducibility
        #[arg(short, long)]
        seed: Option<u64>,

        /// Output file path (.mid or .wav)
        #[arg(short, long)]
        output: PathBuf,

        /// SoundFont file for WAV rendering
        #[arg(long)]
        soundfont: Option<PathBuf>,
    },

    /// Render existing MIDI file to WAV
    Render {
        /// Input MIDI file
        #[arg(short, long)]
        input: PathBuf,

        /// Output WAV file
        #[arg(short, long)]
        output: PathBuf,

        /// SoundFont file for rendering
        #[arg(long)]
        soundfont: Option<PathBuf>,
    },

    /// List available instruments
    Instruments,

    /// List available mood presets
    Moods,

    /// Show information about a MIDI file
    Info {
        /// MIDI file to inspect
        file: PathBuf,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    match run(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("ERROR: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    match cli.command {
        Commands::Generate {
            notes,
            json,
            instrument,
            tempo,
            output,
            soundfont,
        } => {
            let sequences = if json {
                // Read JSON from stdin
                let mut input = String::new();
                io::stdin().read_to_string(&mut input)?;
                let json_input: JsonSequenceInput = serde_json::from_str(&input)?;
                json_input.to_sequences()?
            } else if let Some(notes_str) = notes {
                // Parse notes from CLI argument
                let parsed_notes = Note::parse_many(&notes_str)?;
                let inst = resolve_instrument(&instrument).ok_or_else(|| {
                    format!("Unknown instrument: {instrument}. Use 'instruments' command to list.")
                })?;
                vec![NoteSequence::new(parsed_notes, inst, tempo)]
            } else {
                return Err("Either --notes or --json must be specified".into());
            };

            if sequences.is_empty() {
                return Err("No notes to generate".into());
            }

            // Determine output format from extension
            let ext = output.extension().and_then(|s| s.to_str()).unwrap_or("mid");

            let midi_path = if ext == "wav" {
                output.with_extension("mid")
            } else {
                output.clone()
            };

            // Write MIDI file
            write_midi(&sequences, &midi_path)?;
            eprintln!("Generated MIDI: {}", midi_path.display());

            // Render to WAV if requested
            if ext == "wav" {
                render_wav(&midi_path, &output, soundfont.as_ref())?;
                eprintln!("Rendered WAV: {}", output.display());
            }

            Ok(())
        }

        Commands::Preset {
            mood,
            duration,
            key,
            intensity,
            tempo,
            seed,
            output,
            soundfont,
        } => {
            // Parse mood
            let mood_enum = Mood::parse(&mood).ok_or_else(|| {
                format!("Unknown mood: {mood}. Available: suspense, eerie, upbeat, calm, ambient")
            })?;

            // Parse key (or use mood default)
            let key_enum = if let Some(k) = key {
                Key::parse(&k)
                    .ok_or_else(|| format!("Unknown key: {k}. Examples: C, Am, F#m, Bb"))?
            } else {
                mood_enum.default_key()
            };

            // Create config
            let config = PresetConfig {
                duration_secs: duration,
                key: key_enum,
                intensity: intensity.min(100),
                seed: seed.unwrap_or_else(|| {
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_secs())
                        .unwrap_or(42)
                }),
                tempo,
            };

            // Generate sequences
            let sequences = generate_mood(mood_enum, &config);

            if sequences.is_empty() {
                return Err("No sequences generated".into());
            }

            // Determine output format from extension
            let ext = output.extension().and_then(|s| s.to_str()).unwrap_or("mid");

            let midi_path = if ext == "wav" {
                output.with_extension("mid")
            } else {
                output.clone()
            };

            // Write MIDI file
            write_midi(&sequences, &midi_path)?;
            eprintln!(
                "Generated {:?} preset (seed: {}, key: {:?}): {}",
                mood_enum,
                config.seed,
                key_enum,
                midi_path.display()
            );

            // Render to WAV if requested
            if ext == "wav" {
                render_wav(&midi_path, &output, soundfont.as_ref())?;
                eprintln!("Rendered WAV: {}", output.display());
            }

            Ok(())
        }

        Commands::Render {
            input,
            output,
            soundfont,
        } => {
            render_wav(&input, &output, soundfont.as_ref())?;
            eprintln!("Rendered WAV: {}", output.display());
            Ok(())
        }

        Commands::Instruments => {
            println!("Available instruments:\n");
            println!("{:<20} GM PROGRAM", "NAME");
            println!("{:-<32}", "");
            for (name, num) in midi_cli_rs::INSTRUMENT_MAP {
                println!("{name:<20} {num}");
            }
            println!("\nYou can also use program numbers directly (0-127).");
            Ok(())
        }

        Commands::Moods => {
            println!("Available mood presets:\n");
            println!("{:<12} {:<8} DESCRIPTION", "MOOD", "KEY");
            println!("{:-<60}", "");
            println!(
                "{:<12} {:<8} Tense mood with low drones and tremolo strings",
                "suspense", "Am"
            );
            println!(
                "{:<12} {:<8} Creepy mood with sparse tones and diminished harmony",
                "eerie", "Dm"
            );
            println!(
                "{:<12} {:<8} Energetic mood with rhythmic patterns",
                "upbeat", "C"
            );
            println!(
                "{:<12} {:<8} Peaceful mood with sustained pads and arpeggios",
                "calm", "G"
            );
            println!(
                "{:<12} {:<8} Atmospheric mood with drones and pentatonic tones",
                "ambient", "Em"
            );
            println!("\nUsage: midi-cli-rs preset --mood suspense --duration 5 -o out.wav");
            println!("       midi-cli-rs preset -m upbeat -d 7 --key C --seed 42 -o intro.wav");
            Ok(())
        }

        Commands::Info { file } => {
            let content = std::fs::read(&file)?;
            let smf = midly::Smf::parse(&content)?;

            println!("MIDI File: {}", file.display());
            println!("Format: {:?}", smf.header.format);
            println!("Timing: {:?}", smf.header.timing);
            println!("Tracks: {}", smf.tracks.len());

            for (i, track) in smf.tracks.iter().enumerate() {
                let events = track.len();
                println!("  Track {i}: {events} events");
            }

            Ok(())
        }
    }
}

/// Render MIDI file to WAV using FluidSynth
fn render_wav(
    midi_path: &Path,
    wav_path: &Path,
    soundfont: Option<&PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Find FluidSynth
    let fluidsynth = find_fluidsynth()?;

    // Find SoundFont
    let sf = if let Some(sf) = soundfont {
        sf.to_path_buf()
    } else {
        find_soundfont()?
    };

    // Run FluidSynth
    // Usage: fluidsynth [options] soundfont.sf2 midifile.mid
    // -F option must come before soundfont and midi file
    let status = Command::new(&fluidsynth)
        .args([
            "-ni", // Non-interactive, no shell
            "-g",
            "1.0", // Gain
            "-r",
            "44100", // Sample rate
            "-F",
            wav_path.to_str().unwrap(),  // Output WAV file
            sf.to_str().unwrap(),        // SoundFont file
            midi_path.to_str().unwrap(), // Input MIDI file
        ])
        .status()?;

    if !status.success() {
        return Err(format!("FluidSynth failed with status: {status}").into());
    }

    Ok(())
}

/// Find FluidSynth binary
fn find_fluidsynth() -> Result<PathBuf, Box<dyn std::error::Error>> {
    // Check if fluidsynth is in PATH
    if Command::new("fluidsynth").arg("--version").output().is_ok() {
        return Ok(PathBuf::from("fluidsynth"));
    }

    // Check common locations
    let paths = [
        "/opt/homebrew/bin/fluidsynth",
        "/usr/local/bin/fluidsynth",
        "/usr/bin/fluidsynth",
    ];

    for path in paths {
        let p = PathBuf::from(path);
        if p.exists() {
            return Ok(p);
        }
    }

    Err("FluidSynth not found. Install with:\n  macOS: brew install fluid-synth\n  Ubuntu: apt install fluidsynth".into())
}

/// Find a SoundFont file
fn find_soundfont() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let paths = [
        // Project local (preferred)
        "./soundfonts/TimGM6mb.sf2",
        "./soundfonts/default.sf2",
        "./soundfonts/FluidR3_GM.sf2",
        // macOS Homebrew
        "/opt/homebrew/share/soundfonts/default.sf2",
        "/opt/homebrew/share/sounds/sf2/FluidR3_GM.sf2",
        "/usr/local/share/soundfonts/default.sf2",
        // Linux
        "/usr/share/sounds/sf2/FluidR3_GM.sf2",
        "/usr/share/soundfonts/FluidR3_GM.sf2",
        "/usr/share/soundfonts/default.sf2",
        "/usr/share/soundfonts/freepats-general-midi.sf2",
        "/usr/share/soundfonts/TimGM6mb.sf2",
    ];

    for path in paths {
        let p = PathBuf::from(path);
        if p.exists() {
            return Ok(p);
        }
    }

    Err("No SoundFont found. Install FluidR3_GM or specify --soundfont.\n  macOS: brew install fluid-synth (includes SoundFont)\n  Ubuntu: apt install fluid-soundfont-gm".into())
}
