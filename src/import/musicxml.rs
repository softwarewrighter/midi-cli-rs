//! MusicXML parser
//!
//! Parses MusicXML files into ImportedMelody structures.
//! MusicXML is the standard interchange format for music notation software.
//!
//! # Supported Features
//! - Uncompressed .musicxml files
//! - Compressed .mxl files (ZIP archive)
//! - Pitch parsing (step, octave, alter)
//! - Duration calculation
//! - Key signature
//! - Tempo from metronome marks
//! - Rests
//!
//! # Limitations
//! - Only parses first part (monophonic melody)
//! - Ignores dynamics, articulations, ornaments
//! - Ignores tied notes (each note played separately)

use super::{ImportError, ImportedMelody, ImportedNote, MelodyImporter};
use std::io::Read;
use std::path::Path;

/// MusicXML parser
pub struct MusicXmlParser;

impl MelodyImporter for MusicXmlParser {
    fn parse_file(path: &Path) -> Result<ImportedMelody, ImportError> {
        if !path.exists() {
            return Err(ImportError::FileNotFound(path.display().to_string()));
        }

        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        match ext.as_str() {
            "mxl" => {
                // Compressed MusicXML
                let content = extract_mxl(path)?;
                Self::parse_string(&content)
            }
            "musicxml" | "xml" => {
                // Uncompressed MusicXML
                let content = std::fs::read_to_string(path)?;
                Self::parse_string(&content)
            }
            _ => Err(ImportError::UnsupportedFormat(format!(
                "Unknown extension: {}",
                ext
            ))),
        }
    }

    fn parse_string(content: &str) -> Result<ImportedMelody, ImportError> {
        let options = roxmltree::ParsingOptions {
            allow_dtd: true,
            ..Default::default()
        };
        let doc = roxmltree::Document::parse_with_options(content, options)
            .map_err(|e| ImportError::Xml(e.to_string()))?;

        let root = doc.root_element();
        let mut melody = ImportedMelody::new();
        let mut divisions = 1; // divisions per quarter note
        let mut current_offset = 0.0;
        let mut key_fifths = 0i8; // positive = sharps, negative = flats

        // Get work title
        if let Some(work) = root.children().find(|n| n.has_tag_name("work")) {
            if let Some(title) = work.children().find(|n| n.has_tag_name("work-title")) {
                melody.title = title.text().map(|s| s.to_string());
            }
        }

        // Also check movement-title
        if melody.title.is_none() {
            if let Some(movement) = root
                .children()
                .find(|n| n.has_tag_name("movement-title"))
            {
                melody.title = movement.text().map(|s| s.to_string());
            }
        }

        // Find first part
        let part = root
            .children()
            .find(|n| n.has_tag_name("part"))
            .ok_or_else(|| ImportError::InvalidMusicXml("No part found".to_string()))?;

        // Parse measures
        for measure in part.children().filter(|n| n.has_tag_name("measure")) {
            for element in measure.children() {
                match element.tag_name().name() {
                    "attributes" => {
                        // Get divisions
                        if let Some(div) = element.children().find(|n| n.has_tag_name("divisions"))
                        {
                            if let Some(text) = div.text() {
                                divisions = text.parse().unwrap_or(1);
                            }
                        }

                        // Get key signature
                        if let Some(key) = element.children().find(|n| n.has_tag_name("key")) {
                            if let Some(fifths) =
                                key.children().find(|n| n.has_tag_name("fifths"))
                            {
                                if let Some(text) = fifths.text() {
                                    key_fifths = text.parse().unwrap_or(0);
                                }
                            }
                            // Convert fifths to key name
                            melody.key = Some(fifths_to_key(key_fifths));
                        }

                        // Get time signature
                        if let Some(time) = element.children().find(|n| n.has_tag_name("time")) {
                            let beats = time
                                .children()
                                .find(|n| n.has_tag_name("beats"))
                                .and_then(|n| n.text())
                                .and_then(|t| t.parse().ok())
                                .unwrap_or(4);
                            let beat_type = time
                                .children()
                                .find(|n| n.has_tag_name("beat-type"))
                                .and_then(|n| n.text())
                                .and_then(|t| t.parse().ok())
                                .unwrap_or(4);
                            melody.time_signature = (beats, beat_type);
                        }
                    }

                    "direction" => {
                        // Check for tempo
                        if let Some(sound) = element
                            .descendants()
                            .find(|n| n.has_tag_name("sound"))
                        {
                            if let Some(tempo) = sound.attribute("tempo") {
                                melody.tempo = tempo.parse().ok();
                            }
                        }

                        // Also check metronome
                        if melody.tempo.is_none() {
                            if let Some(metronome) = element
                                .descendants()
                                .find(|n| n.has_tag_name("metronome"))
                            {
                                if let Some(per_minute) = metronome
                                    .children()
                                    .find(|n| n.has_tag_name("per-minute"))
                                {
                                    if let Some(text) = per_minute.text() {
                                        melody.tempo = text.parse().ok();
                                    }
                                }
                            }
                        }
                    }

                    "note" => {
                        // Check if this is a chord (simultaneous with previous)
                        let is_chord = element.children().any(|n| n.has_tag_name("chord"));

                        // Get duration
                        let duration_divs = element
                            .children()
                            .find(|n| n.has_tag_name("duration"))
                            .and_then(|n| n.text())
                            .and_then(|t| t.parse::<i32>().ok())
                            .unwrap_or(divisions);

                        let duration_beats = duration_divs as f64 / divisions as f64;

                        // Check if rest
                        if element.children().any(|n| n.has_tag_name("rest")) {
                            melody.notes.push(ImportedNote::rest(duration_beats, current_offset));
                            if !is_chord {
                                current_offset += duration_beats;
                            }
                            continue;
                        }

                        // Get pitch
                        if let Some(pitch) = element.children().find(|n| n.has_tag_name("pitch")) {
                            let step = pitch
                                .children()
                                .find(|n| n.has_tag_name("step"))
                                .and_then(|n| n.text())
                                .unwrap_or("C");

                            let octave: i8 = pitch
                                .children()
                                .find(|n| n.has_tag_name("octave"))
                                .and_then(|n| n.text())
                                .and_then(|t| t.parse().ok())
                                .unwrap_or(4);

                            let alter: i8 = pitch
                                .children()
                                .find(|n| n.has_tag_name("alter"))
                                .and_then(|n| n.text())
                                .and_then(|t| t.parse().ok())
                                .unwrap_or(0);

                            let midi_pitch = step_to_midi(step, octave, alter);

                            // For chords, use same offset as previous note
                            let note_offset = if is_chord && !melody.notes.is_empty() {
                                melody.notes.last().map(|n| n.offset).unwrap_or(current_offset)
                            } else {
                                current_offset
                            };

                            melody.notes.push(ImportedNote::new(
                                midi_pitch,
                                duration_beats,
                                note_offset,
                            ));

                            if !is_chord {
                                current_offset += duration_beats;
                            }
                        }
                    }

                    "forward" => {
                        // Move forward without playing
                        let duration_divs = element
                            .children()
                            .find(|n| n.has_tag_name("duration"))
                            .and_then(|n| n.text())
                            .and_then(|t| t.parse::<i32>().ok())
                            .unwrap_or(0);
                        current_offset += duration_divs as f64 / divisions as f64;
                    }

                    "backup" => {
                        // Move backward (for multiple voices)
                        let duration_divs = element
                            .children()
                            .find(|n| n.has_tag_name("duration"))
                            .and_then(|n| n.text())
                            .and_then(|t| t.parse::<i32>().ok())
                            .unwrap_or(0);
                        current_offset -= duration_divs as f64 / divisions as f64;
                        if current_offset < 0.0 {
                            current_offset = 0.0;
                        }
                    }

                    _ => {}
                }
            }
        }

        if melody.notes.is_empty() {
            return Err(ImportError::NoNotes);
        }

        Ok(melody)
    }
}

impl MusicXmlParser {
    /// Parse MusicXML from a file
    pub fn parse_file(path: &Path) -> Result<ImportedMelody, ImportError> {
        <Self as MelodyImporter>::parse_file(path)
    }

    /// Parse MusicXML from a string
    pub fn parse_string(content: &str) -> Result<ImportedMelody, ImportError> {
        <Self as MelodyImporter>::parse_string(content)
    }
}

/// Extract MusicXML content from a compressed .mxl file
fn extract_mxl(path: &Path) -> Result<String, ImportError> {
    let file = std::fs::File::open(path)?;
    let mut archive =
        zip::ZipArchive::new(file).map_err(|e| ImportError::Zip(e.to_string()))?;

    // First, try to find META-INF/container.xml which points to the main file
    let root_file = if let Ok(container) = archive.by_name("META-INF/container.xml") {
        let content: String = container
            .bytes()
            .filter_map(|b| b.ok())
            .map(|b| b as char)
            .collect();

        // Parse container.xml to find rootfile
        if let Ok(doc) = roxmltree::Document::parse(&content) {
            doc.descendants()
                .find(|n| n.has_tag_name("rootfile"))
                .and_then(|n| n.attribute("full-path"))
                .map(|s| s.to_string())
        } else {
            None
        }
    } else {
        None
    };

    // Try the root file from container, or common names
    let xml_files = [
        root_file.as_deref(),
        Some("score.xml"),
        Some("musicXML.xml"),
    ];

    for name in xml_files.into_iter().flatten() {
        if let Ok(mut file) = archive.by_name(name) {
            let mut content = String::new();
            file.read_to_string(&mut content)?;
            return Ok(content);
        }
    }

    // Last resort: find any .xml file
    for i in 0..archive.len() {
        if let Ok(mut file) = archive.by_index(i) {
            if file.name().ends_with(".xml") && !file.name().contains("META-INF") {
                let mut content = String::new();
                file.read_to_string(&mut content)?;
                return Ok(content);
            }
        }
    }

    Err(ImportError::Zip(
        "No MusicXML file found in archive".to_string(),
    ))
}

/// Convert step (C-B), octave, and alter to MIDI pitch
fn step_to_midi(step: &str, octave: i8, alter: i8) -> u8 {
    let semitone = match step.to_uppercase().as_str() {
        "C" => 0,
        "D" => 2,
        "E" => 4,
        "F" => 5,
        "G" => 7,
        "A" => 9,
        "B" => 11,
        _ => 0,
    };

    let midi = (octave as i16 + 1) * 12 + semitone as i16 + alter as i16;
    midi.clamp(0, 127) as u8
}

/// Convert key fifths to key name
fn fifths_to_key(fifths: i8) -> String {
    match fifths {
        -7 => "Cb".to_string(),
        -6 => "Gb".to_string(),
        -5 => "Db".to_string(),
        -4 => "Ab".to_string(),
        -3 => "Eb".to_string(),
        -2 => "Bb".to_string(),
        -1 => "F".to_string(),
        0 => "C".to_string(),
        1 => "G".to_string(),
        2 => "D".to_string(),
        3 => "A".to_string(),
        4 => "E".to_string(),
        5 => "B".to_string(),
        6 => "F#".to_string(),
        7 => "C#".to_string(),
        _ => "C".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_step_to_midi() {
        assert_eq!(step_to_midi("C", 4, 0), 60); // Middle C
        assert_eq!(step_to_midi("A", 4, 0), 69); // A440
        assert_eq!(step_to_midi("C", 4, 1), 61); // C#4
        assert_eq!(step_to_midi("D", 4, -1), 61); // Db4
        assert_eq!(step_to_midi("C", 0, 0), 12); // C0
    }

    #[test]
    fn test_fifths_to_key() {
        assert_eq!(fifths_to_key(0), "C");
        assert_eq!(fifths_to_key(1), "G");
        assert_eq!(fifths_to_key(2), "D");
        assert_eq!(fifths_to_key(-1), "F");
        assert_eq!(fifths_to_key(-2), "Bb");
    }

    #[test]
    fn test_simple_musicxml() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE score-partwise PUBLIC "-//Recordare//DTD MusicXML 3.1 Partwise//EN" "http://www.musicxml.org/dtds/partwise.dtd">
<score-partwise version="3.1">
  <work>
    <work-title>Test</work-title>
  </work>
  <part-list>
    <score-part id="P1">
      <part-name>Piano</part-name>
    </score-part>
  </part-list>
  <part id="P1">
    <measure number="1">
      <attributes>
        <divisions>1</divisions>
        <key>
          <fifths>0</fifths>
        </key>
        <time>
          <beats>4</beats>
          <beat-type>4</beat-type>
        </time>
      </attributes>
      <note>
        <pitch>
          <step>C</step>
          <octave>4</octave>
        </pitch>
        <duration>1</duration>
      </note>
      <note>
        <pitch>
          <step>D</step>
          <octave>4</octave>
        </pitch>
        <duration>1</duration>
      </note>
      <note>
        <pitch>
          <step>E</step>
          <octave>4</octave>
        </pitch>
        <duration>1</duration>
      </note>
      <note>
        <pitch>
          <step>F</step>
          <octave>4</octave>
        </pitch>
        <duration>1</duration>
      </note>
    </measure>
  </part>
</score-partwise>"#;

        let melody = MusicXmlParser::parse_string(xml).unwrap();
        assert_eq!(melody.title, Some("Test".to_string()));
        assert_eq!(melody.key, Some("C".to_string()));
        assert_eq!(melody.time_signature, (4, 4));
        assert_eq!(melody.note_count(), 4);

        let pitches: Vec<u8> = melody.notes.iter().filter_map(|n| n.pitch).collect();
        assert_eq!(pitches, vec![60, 62, 64, 65]); // C4, D4, E4, F4
    }

    #[test]
    fn test_with_rests() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<score-partwise version="3.1">
  <part-list>
    <score-part id="P1"><part-name>P</part-name></score-part>
  </part-list>
  <part id="P1">
    <measure number="1">
      <attributes>
        <divisions>1</divisions>
      </attributes>
      <note>
        <pitch><step>C</step><octave>4</octave></pitch>
        <duration>1</duration>
      </note>
      <note>
        <rest/>
        <duration>1</duration>
      </note>
      <note>
        <pitch><step>E</step><octave>4</octave></pitch>
        <duration>1</duration>
      </note>
    </measure>
  </part>
</score-partwise>"#;

        let melody = MusicXmlParser::parse_string(xml).unwrap();
        assert_eq!(melody.notes.len(), 3);
        assert_eq!(melody.notes[0].pitch, Some(60)); // C
        assert_eq!(melody.notes[1].pitch, None); // rest
        assert_eq!(melody.notes[2].pitch, Some(64)); // E
    }

    #[test]
    fn test_with_accidentals() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<score-partwise version="3.1">
  <part-list>
    <score-part id="P1"><part-name>P</part-name></score-part>
  </part-list>
  <part id="P1">
    <measure number="1">
      <attributes>
        <divisions>1</divisions>
      </attributes>
      <note>
        <pitch><step>C</step><alter>1</alter><octave>4</octave></pitch>
        <duration>1</duration>
      </note>
      <note>
        <pitch><step>D</step><alter>-1</alter><octave>4</octave></pitch>
        <duration>1</duration>
      </note>
    </measure>
  </part>
</score-partwise>"#;

        let melody = MusicXmlParser::parse_string(xml).unwrap();
        let pitches: Vec<u8> = melody.notes.iter().filter_map(|n| n.pitch).collect();
        assert_eq!(pitches, vec![61, 61]); // C#4 and Db4 are same pitch
    }
}
