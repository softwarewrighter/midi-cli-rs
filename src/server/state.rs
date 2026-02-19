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
    /// Path to the presets JSON file.
    pub storage_path: PathBuf,
    /// Directory for generated audio files.
    pub output_dir: PathBuf,
}

impl AppState {
    /// Load state from disk or create new state.
    pub fn load_or_create() -> Result<Arc<Self>, std::io::Error> {
        let config_dir = dirs_config_dir();
        std::fs::create_dir_all(&config_dir)?;

        let storage_path = config_dir.join("presets.json");
        let output_dir = PathBuf::from("generated");
        std::fs::create_dir_all(&output_dir)?;

        let presets = if storage_path.exists() {
            let content = std::fs::read_to_string(&storage_path)?;
            let storage: PresetStorage = serde_json::from_str(&content).unwrap_or_default();
            storage
                .presets
                .into_iter()
                .map(|p| (p.id.clone(), p))
                .collect()
        } else {
            HashMap::new()
        };

        Ok(Arc::new(Self {
            presets: RwLock::new(presets),
            storage_path,
            output_dir,
        }))
    }

    /// Persist presets to disk.
    pub async fn save(&self) -> Result<(), std::io::Error> {
        let presets = self.presets.read().await;
        let storage = PresetStorage {
            presets: presets.values().cloned().collect(),
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

/// Storage format for presets JSON file.
#[derive(Serialize, Deserialize, Default)]
pub struct PresetStorage {
    pub presets: Vec<SavedPreset>,
}

/// A saved preset configuration.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SavedPreset {
    /// Unique identifier (UUID).
    pub id: String,
    /// User-given name for the preset.
    pub name: String,
    /// Mood preset type.
    pub mood: String,
    /// Duration in seconds.
    pub duration: f64,
    /// Musical key (optional).
    pub key: Option<String>,
    /// Intensity 0-100.
    pub intensity: u8,
    /// Tempo in BPM.
    pub tempo: u16,
    /// Random seed for reproducibility.
    pub seed: i64,
    /// ISO 8601 timestamp when created.
    pub created_at: String,
    /// ISO 8601 timestamp of last generation.
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
    /// Convert to a SavedPreset with a new ID and timestamp.
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
