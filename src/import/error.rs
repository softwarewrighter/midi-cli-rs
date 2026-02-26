//! Import errors for ABC and MusicXML parsing

use thiserror::Error;

/// Errors that can occur when importing melodies
#[derive(Debug, Error)]
pub enum ImportError {
    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid ABC notation: {0}")]
    InvalidAbc(String),

    #[error("Invalid MusicXML: {0}")]
    InvalidMusicXml(String),

    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),

    #[error("No notes found in file")]
    NoNotes,

    #[error("ZIP error: {0}")]
    Zip(String),

    #[error("XML parsing error: {0}")]
    Xml(String),
}
