//! HTTP client for communicating with the Axum server API.

use gloo_net::http::Request;
use serde::{Deserialize, Serialize};

const API_BASE: &str = "/api";

// ============================================================================
// Preset types
// ============================================================================

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

// ============================================================================
// Melody types
// ============================================================================

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct MelodyNote {
    pub pitch: String,
    pub duration: f64,
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

impl MelodyNote {
    pub fn rest(duration: f64) -> Self {
        Self {
            pitch: "rest".to_string(),
            duration,
            velocity: 0,
        }
    }

    pub fn is_rest(&self) -> bool {
        self.pitch == "rest"
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SavedMelody {
    pub id: String,
    pub name: String,
    pub notes: Vec<MelodyNote>,
    pub key: String,
    pub tempo: u16,
    pub instrument: String,
    pub attack: u8,
    pub decay: u8,
    pub created_at: String,
    pub last_generated: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MelodyRequest {
    pub name: String,
    pub notes: Vec<MelodyNote>,
    pub key: String,
    pub tempo: u16,
    pub instrument: String,
    pub attack: u8,
    pub decay: u8,
}

impl Default for MelodyRequest {
    fn default() -> Self {
        Self {
            name: String::new(),
            notes: vec![MelodyNote::default()],
            key: "C".to_string(),
            tempo: 120,
            instrument: "piano".to_string(),
            attack: 0,
            decay: 64,
        }
    }
}

// ============================================================================
// Common types
// ============================================================================

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GenerateResponse {
    pub preset_id: String,
    pub audio_url: String,
    pub generated_at: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InstrumentInfo {
    pub name: String,
    pub program: u8,
}

#[derive(Deserialize)]
struct ErrorResponse {
    error: String,
}

// ============================================================================
// API Client
// ============================================================================

pub struct ApiClient;

impl ApiClient {
    /// Extract error message from response body, or fall back to status code
    async fn extract_error(response: gloo_net::http::Response, context: &str) -> String {
        let status = response.status();
        match response.json::<ErrorResponse>().await {
            Ok(err) => format!("{}: {}", context, err.error),
            Err(_) => format!("{}: HTTP {}", context, status),
        }
    }
}

impl ApiClient {
    // Preset endpoints
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
            Err(Self::extract_error(response, "Failed to create preset").await)
        }
    }

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
            Err(Self::extract_error(response, "Failed to update preset").await)
        }
    }

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

    pub async fn generate_preset_audio(id: &str) -> Result<GenerateResponse, String> {
        let response = Request::post(&format!("{}/generate/{}", API_BASE, id))
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if response.ok() {
            response.json().await.map_err(|e| e.to_string())
        } else {
            Err(Self::extract_error(response, "Generate failed").await)
        }
    }

    // Melody endpoints
    pub async fn list_melodies() -> Result<Vec<SavedMelody>, String> {
        let response = Request::get(&format!("{}/melodies", API_BASE))
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if response.ok() {
            response.json().await.map_err(|e| e.to_string())
        } else {
            Err(format!("Failed to fetch melodies: {}", response.status()))
        }
    }

    pub async fn create_melody(req: &MelodyRequest) -> Result<SavedMelody, String> {
        let response = Request::post(&format!("{}/melodies", API_BASE))
            .json(req)
            .map_err(|e| e.to_string())?
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if response.ok() {
            response.json().await.map_err(|e| e.to_string())
        } else {
            Err(Self::extract_error(response, "Failed to create melody").await)
        }
    }

    pub async fn update_melody(id: &str, req: &MelodyRequest) -> Result<SavedMelody, String> {
        let response = Request::put(&format!("{}/melodies/{}", API_BASE, id))
            .json(req)
            .map_err(|e| e.to_string())?
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if response.ok() {
            response.json().await.map_err(|e| e.to_string())
        } else {
            Err(Self::extract_error(response, "Failed to update melody").await)
        }
    }

    pub async fn delete_melody(id: &str) -> Result<(), String> {
        let response = Request::delete(&format!("{}/melodies/{}", API_BASE, id))
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if response.ok() || response.status() == 204 {
            Ok(())
        } else {
            Err(format!("Failed to delete melody: {}", response.status()))
        }
    }

    pub async fn generate_melody_audio(id: &str) -> Result<GenerateResponse, String> {
        let response = Request::post(&format!("{}/melodies/{}/generate", API_BASE, id))
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if response.ok() {
            response.json().await.map_err(|e| e.to_string())
        } else {
            Err(Self::extract_error(response, "Generate failed").await)
        }
    }

    pub async fn list_instruments() -> Result<Vec<InstrumentInfo>, String> {
        let response = Request::get(&format!("{}/instruments", API_BASE))
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if response.ok() {
            response.json().await.map_err(|e| e.to_string())
        } else {
            Err(format!("Failed to fetch instruments: {}", response.status()))
        }
    }
}
