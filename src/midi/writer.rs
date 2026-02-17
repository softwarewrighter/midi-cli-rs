//! MIDI file writer using midly crate
//!
//! Generates Standard MIDI Files (SMF) from note sequences.

use super::NoteSequence;
use midly::{Format, Header, MidiMessage, Smf, Timing, Track, TrackEvent, TrackEventKind};
use std::fs::File;
use std::io::Write;
use std::path::Path;
use thiserror::Error;

/// Ticks per quarter note (standard resolution)
const TICKS_PER_BEAT: u16 = 480;

/// Errors that can occur when writing MIDI files
#[derive(Debug, Error)]
pub enum MidiWriteError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("No sequences provided")]
    EmptySequences,
}

/// Convert beats to MIDI ticks
fn beats_to_ticks(beats: f64) -> u32 {
    (beats * TICKS_PER_BEAT as f64) as u32
}

/// Write sequences to a MIDI file
pub fn write_midi(sequences: &[NoteSequence], path: &Path) -> Result<(), MidiWriteError> {
    if sequences.is_empty() {
        return Err(MidiWriteError::EmptySequences);
    }

    // Use tempo from first sequence
    let tempo = sequences[0].tempo;

    // Create MIDI file structure
    let mut tracks: Vec<Track> = Vec::new();

    // Track 0: Tempo and time signature
    let mut tempo_track: Track = Vec::new();

    // Set tempo (microseconds per beat)
    let microseconds_per_beat = 60_000_000 / tempo as u32;
    tempo_track.push(TrackEvent {
        delta: 0.into(),
        kind: TrackEventKind::Meta(midly::MetaMessage::Tempo(microseconds_per_beat.into())),
    });

    // Time signature: 4/4
    tempo_track.push(TrackEvent {
        delta: 0.into(),
        kind: TrackEventKind::Meta(midly::MetaMessage::TimeSignature(4, 2, 24, 8)),
    });

    // End of track
    tempo_track.push(TrackEvent {
        delta: 0.into(),
        kind: TrackEventKind::Meta(midly::MetaMessage::EndOfTrack),
    });

    tracks.push(tempo_track);

    // Add a track for each sequence
    for seq in sequences {
        let track = build_track(seq);
        tracks.push(track);
    }

    // Create SMF
    let smf = Smf {
        header: Header {
            format: Format::Parallel,
            timing: Timing::Metrical(TICKS_PER_BEAT.into()),
        },
        tracks,
    };

    // Write to file
    let mut file = File::create(path)?;
    let mut buffer = Vec::new();
    smf.write_std(&mut buffer)
        .map_err(|e| std::io::Error::other(format!("MIDI write error: {e}")))?;
    file.write_all(&buffer)?;

    Ok(())
}

/// Build a MIDI track from a note sequence
fn build_track(seq: &NoteSequence) -> Track<'static> {
    let mut track: Track = Vec::new();
    let channel = seq.channel.into();

    // Program change (instrument selection)
    track.push(TrackEvent {
        delta: 0.into(),
        kind: TrackEventKind::Midi {
            channel,
            message: MidiMessage::ProgramChange {
                program: seq.instrument.into(),
            },
        },
    });

    // Build events list: collect all note-on and note-off events
    let mut events: Vec<(u32, bool, u8, u8)> = Vec::new(); // (tick, is_note_on, pitch, velocity)

    for note in &seq.notes {
        let start_tick = beats_to_ticks(note.offset);
        let end_tick = beats_to_ticks(note.offset + note.duration);

        events.push((start_tick, true, note.pitch, note.velocity));
        events.push((end_tick, false, note.pitch, 0));
    }

    // Sort by tick, note-offs before note-ons at same tick
    events.sort_by(|a, b| {
        if a.0 != b.0 {
            a.0.cmp(&b.0)
        } else {
            // Note-off (false) before note-on (true)
            a.1.cmp(&b.1)
        }
    });

    // Convert to delta times and add to track
    let mut last_tick = 0u32;
    for (tick, is_note_on, pitch, velocity) in events {
        let delta = tick.saturating_sub(last_tick);
        last_tick = tick;

        let message = if is_note_on {
            MidiMessage::NoteOn {
                key: pitch.into(),
                vel: velocity.into(),
            }
        } else {
            MidiMessage::NoteOff {
                key: pitch.into(),
                vel: 0.into(),
            }
        };

        track.push(TrackEvent {
            delta: delta.into(),
            kind: TrackEventKind::Midi { channel, message },
        });
    }

    // End of track
    track.push(TrackEvent {
        delta: 0.into(),
        kind: TrackEventKind::Meta(midly::MetaMessage::EndOfTrack),
    });

    track
}

/// Write a single sequence to a MIDI file
pub fn write_midi_single(seq: &NoteSequence, path: &Path) -> Result<(), MidiWriteError> {
    write_midi(std::slice::from_ref(seq), path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::midi::Note;
    use tempfile::tempdir;

    #[test]
    fn test_beats_to_ticks() {
        assert_eq!(beats_to_ticks(1.0), 480);
        assert_eq!(beats_to_ticks(0.5), 240);
        assert_eq!(beats_to_ticks(2.0), 960);
        assert_eq!(beats_to_ticks(0.25), 120);
    }

    #[test]
    fn test_write_simple_midi() {
        let temp = tempdir().unwrap();
        let path = temp.path().join("test.mid");

        let notes = vec![
            Note::new(60, 1.0, 80, 0.0),
            Note::new(64, 1.0, 80, 1.0),
            Note::new(67, 1.0, 80, 2.0),
        ];
        let seq = NoteSequence::new(notes, 0, 120);

        write_midi_single(&seq, &path).unwrap();

        // Verify file exists and has content
        assert!(path.exists());
        let content = std::fs::read(&path).unwrap();
        assert!(!content.is_empty());

        // Verify it starts with MIDI header
        assert_eq!(&content[0..4], b"MThd");
    }

    #[test]
    fn test_write_midi_with_instrument() {
        let temp = tempdir().unwrap();
        let path = temp.path().join("test_instrument.mid");

        let notes = vec![Note::new(60, 2.0, 100, 0.0)];
        let seq = NoteSequence::new(notes, 40, 90); // violin at 90 BPM

        write_midi_single(&seq, &path).unwrap();
        assert!(path.exists());
    }

    #[test]
    fn test_write_empty_sequences_error() {
        let temp = tempdir().unwrap();
        let path = temp.path().join("empty.mid");

        let result = write_midi(&[], &path);
        assert!(result.is_err());
    }

    #[test]
    fn test_write_multi_track() {
        let temp = tempdir().unwrap();
        let path = temp.path().join("multi.mid");

        let seq1 = NoteSequence::new(vec![Note::new(60, 1.0, 80, 0.0)], 0, 120);
        let seq2 = NoteSequence::new(vec![Note::new(48, 2.0, 100, 0.0)], 33, 120);

        write_midi(&[seq1, seq2], &path).unwrap();

        // Parse the file back to verify structure
        let content = std::fs::read(&path).unwrap();
        let smf = Smf::parse(&content).unwrap();

        // Should have 3 tracks: tempo + 2 instrument tracks
        assert_eq!(smf.tracks.len(), 3);
    }

    #[test]
    fn test_round_trip_parse() {
        let temp = tempdir().unwrap();
        let path = temp.path().join("roundtrip.mid");

        let notes = vec![
            Note::new(60, 1.0, 80, 0.0),
            Note::new(62, 0.5, 90, 1.0),
            Note::new(64, 0.5, 100, 1.5),
        ];
        let seq = NoteSequence::new(notes, 0, 120);

        write_midi_single(&seq, &path).unwrap();

        // Parse it back
        let content = std::fs::read(&path).unwrap();
        let smf = Smf::parse(&content).unwrap();

        // Verify header
        assert_eq!(smf.header.timing, Timing::Metrical(480.into()));

        // Should have tempo track + 1 instrument track
        assert_eq!(smf.tracks.len(), 2);
    }
}
