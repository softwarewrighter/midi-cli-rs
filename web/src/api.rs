//! HTTP client for communicating with the Axum server API.

use gloo_net::http::Request;
use serde::{Deserialize, Serialize};

/// Base URL for API requests (same origin in production).
const API_BASE: &str = "/api";

/// A saved preset configuration.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
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
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct PresetRequest {
    pub name: String,
    pub mood: String,
    pub duration: f64,
    pub key: Option<String>,
    pub intensity: u8,
    pub tempo: u16,
    pub seed: i64,
}

/// Response from generating audio.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GenerateResponse {
    pub preset_id: String,
    pub audio_url: String,
    pub generated_at: String,
}

/// Available mood preset info.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MoodInfo {
    pub name: String,
    pub key: String,
    pub description: String,
}

/// API client for the midi-cli server.
pub struct ApiClient;

impl ApiClient {
    /// Fetch all saved presets.
    pub async fn list_presets() -> Result<Vec<SavedPreset>, String> {
        let response = Request::get(&format!("{}/presets", API_BASE))
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if response.ok() {
            response.json().await.map_err(|e| e.to_string())
        } else {
            Err(format!("Failed to fetch presets: {}", response.status()))
        }
    }

    /// Create a new preset.
    pub async fn create_preset(req: &PresetRequest) -> Result<SavedPreset, String> {
        let response = Request::post(&format!("{}/presets", API_BASE))
            .json(req)
            .map_err(|e| e.to_string())?
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if response.ok() {
            response.json().await.map_err(|e| e.to_string())
        } else {
            Err(format!("Failed to create preset: {}", response.status()))
        }
    }

    /// Update an existing preset.
    pub async fn update_preset(id: &str, req: &PresetRequest) -> Result<SavedPreset, String> {
        let response = Request::put(&format!("{}/presets/{}", API_BASE, id))
            .json(req)
            .map_err(|e| e.to_string())?
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if response.ok() {
            response.json().await.map_err(|e| e.to_string())
        } else {
            Err(format!("Failed to update preset: {}", response.status()))
        }
    }

    /// Delete a preset.
    pub async fn delete_preset(id: &str) -> Result<(), String> {
        let response = Request::delete(&format!("{}/presets/{}", API_BASE, id))
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if response.ok() || response.status() == 204 {
            Ok(())
        } else {
            Err(format!("Failed to delete preset: {}", response.status()))
        }
    }

    /// Generate audio for a preset.
    pub async fn generate_audio(id: &str) -> Result<GenerateResponse, String> {
        let response = Request::post(&format!("{}/generate/{}", API_BASE, id))
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if response.ok() {
            response.json().await.map_err(|e| e.to_string())
        } else {
            Err(format!("Failed to generate audio: {}", response.status()))
        }
    }

    /// Fetch available moods.
    pub async fn list_moods() -> Result<Vec<MoodInfo>, String> {
        let response = Request::get(&format!("{}/moods", API_BASE))
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if response.ok() {
            response.json().await.map_err(|e| e.to_string())
        } else {
            Err(format!("Failed to fetch moods: {}", response.status()))
        }
    }
}
