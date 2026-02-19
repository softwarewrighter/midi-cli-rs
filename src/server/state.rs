//! Application state and storage for the web server.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Application state shared across all request handlers.
pub struct AppState {
    /// Saved presets indexed by ID.
    pub presets: RwLock<HashMap<String, SavedPreset>>,
    /// Saved melodies indexed by ID.
    pub melodies: RwLock<HashMap<String, SavedMelody>>,
    /// Path to the storage JSON file.
    pub storage_path: PathBuf,
    /// Directory for generated audio files.
    pub output_dir: PathBuf,
}

impl AppState {
    /// Load state from disk or create new state.
    pub fn load_or_create() -> Result<Arc<Self>, std::io::Error> {
        let config_dir = dirs_config_dir();
        std::fs::create_dir_all(&config_dir)?;

        let storage_path = config_dir.join("storage.json");
        let output_dir = PathBuf::from("generated");
        std::fs::create_dir_all(&output_dir)?;

        let (presets, melodies) = if storage_path.exists() {
            let content = std::fs::read_to_string(&storage_path)?;
            let storage: AppStorage = serde_json::from_str(&content).unwrap_or_default();
            (
                storage.presets.into_iter().map(|p| (p.id.clone(), p)).collect(),
                storage.melodies.into_iter().map(|m| (m.id.clone(), m)).collect(),
            )
        } else {
            (HashMap::new(), HashMap::new())
        };

        Ok(Arc::new(Self {
            presets: RwLock::new(presets),
            melodies: RwLock::new(melodies),
            storage_path,
            output_dir,
        }))
    }

    /// Persist all data to disk.
    pub async fn save(&self) -> Result<(), std::io::Error> {
        let presets = self.presets.read().await;
        let melodies = self.melodies.read().await;
        let storage = AppStorage {
            presets: presets.values().cloned().collect(),
            melodies: melodies.values().cloned().collect(),
        };
        let json = serde_json::to_string_pretty(&storage)?;
        std::fs::write(&self.storage_path, json)?;
        Ok(())
    }
}

/// Get the configuration directory for midi-cli-rs.
fn dirs_config_dir() -> PathBuf {
    if let Some(home) = std::env::var_os("HOME") {
        PathBuf::from(home).join(".midi-cli-rs")
    } else {
        PathBuf::from(".midi-cli-rs")
    }
}

/// Storage format for the JSON file.
#[derive(Serialize, Deserialize, Default)]
pub struct AppStorage {
    #[serde(default)]
    pub presets: Vec<SavedPreset>,
    #[serde(default)]
    pub melodies: Vec<SavedMelody>,
}

// Legacy support - read old presets.json format
#[derive(Serialize, Deserialize, Default)]
pub struct PresetStorage {
    pub presets: Vec<SavedPreset>,
}

/// A saved preset configuration.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SavedPreset {
    pub id: String,
    pub name: String,
    pub mood: String,
    pub duration: f64,
    pub key: Option<String>,
    pub intensity: u8,
    pub tempo: u16,
    pub seed: i64,
    pub created_at: String,
    pub last_generated: Option<String>,
}

/// Request body for creating/updating a preset.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PresetRequest {
    pub name: String,
    pub mood: String,
    pub duration: f64,
    pub key: Option<String>,
    pub intensity: u8,
    pub tempo: u16,
    pub seed: i64,
}

impl PresetRequest {
    pub fn into_preset(self, id: String) -> SavedPreset {
        SavedPreset {
            id,
            name: self.name,
            mood: self.mood,
            duration: self.duration,
            key: self.key,
            intensity: self.intensity,
            tempo: self.tempo,
            seed: self.seed,
            created_at: chrono::Utc::now().to_rfc3339(),
            last_generated: None,
        }
    }
}

/// A single note or rest in a melody.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct MelodyNote {
    /// Note pitch: "C4", "D#5", "rest", etc.
    pub pitch: String,
    /// Duration in beats (0.25 = sixteenth, 0.5 = eighth, 1.0 = quarter, etc.)
    pub duration: f64,
    /// Velocity 0-127 (0 for rests).
    pub velocity: u8,
}

impl Default for MelodyNote {
    fn default() -> Self {
        Self {
            pitch: "C4".to_string(),
            duration: 1.0,
            velocity: 80,
        }
    }
}

/// A saved melody with notes and settings.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SavedMelody {
    pub id: String,
    pub name: String,
    pub notes: Vec<MelodyNote>,
    /// Musical key (e.g., "C", "Am", "F#m").
    pub key: String,
    /// Tempo in BPM.
    pub tempo: u16,
    /// GM instrument name or number.
    pub instrument: String,
    /// Attack time in ms (0-127 scaled).
    pub attack: u8,
    /// Decay/release time in ms (0-127 scaled).
    pub decay: u8,
    pub created_at: String,
    pub last_generated: Option<String>,
}

/// Request body for creating/updating a melody.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MelodyRequest {
    pub name: String,
    pub notes: Vec<MelodyNote>,
    pub key: String,
    pub tempo: u16,
    pub instrument: String,
    #[serde(default)]
    pub attack: u8,
    #[serde(default = "default_decay")]
    pub decay: u8,
}

fn default_decay() -> u8 {
    64
}

impl MelodyRequest {
    pub fn into_melody(self, id: String) -> SavedMelody {
        SavedMelody {
            id,
            name: self.name,
            notes: self.notes,
            key: self.key,
            tempo: self.tempo,
            instrument: self.instrument,
            attack: self.attack,
            decay: self.decay,
            created_at: chrono::Utc::now().to_rfc3339(),
            last_generated: None,
        }
    }
}

/// Response containing a generated audio file path.
#[derive(Serialize, Deserialize, Debug)]
pub struct GenerateResponse {
    pub preset_id: String,
    pub audio_url: String,
    pub generated_at: String,
}

/// Error response body.
#[derive(Serialize, Deserialize, Debug)]
pub struct ErrorResponse {
    pub error: String,
}
