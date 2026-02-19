//! REST API handlers for the web server.

use crate::server::state::{AppState, ErrorResponse, GenerateResponse, PresetRequest, SavedPreset};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use std::process::Command;
use std::sync::Arc;

/// GET /api/presets - List all saved presets.
pub async fn list_presets(State(state): State<Arc<AppState>>) -> Json<Vec<SavedPreset>> {
    let presets = state.presets.read().await;
    let mut list: Vec<SavedPreset> = presets.values().cloned().collect();
    // Sort by creation time, newest first
    list.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Json(list)
}

/// POST /api/presets - Create a new preset.
pub async fn create_preset(
    State(state): State<Arc<AppState>>,
    Json(req): Json<PresetRequest>,
) -> Result<(StatusCode, Json<SavedPreset>), (StatusCode, Json<ErrorResponse>)> {
    // Validate mood
    if !is_valid_mood(&req.mood) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: format!("Invalid mood: {}. Valid moods: suspense, eerie, upbeat, calm, ambient, jazz", req.mood),
            }),
        ));
    }

    let id = uuid::Uuid::new_v4().to_string();
    let preset = req.into_preset(id);

    let mut presets = state.presets.write().await;
    presets.insert(preset.id.clone(), preset.clone());
    drop(presets);

    if let Err(e) = state.save().await {
        eprintln!("Failed to save presets: {}", e);
    }

    Ok((StatusCode::CREATED, Json(preset)))
}

/// GET /api/presets/:id - Get a single preset.
pub async fn get_preset(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<SavedPreset>, (StatusCode, Json<ErrorResponse>)> {
    let presets = state.presets.read().await;
    presets
        .get(&id)
        .cloned()
        .map(Json)
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: format!("Preset not found: {}", id),
                }),
            )
        })
}

/// PUT /api/presets/:id - Update an existing preset.
pub async fn update_preset(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<PresetRequest>,
) -> Result<Json<SavedPreset>, (StatusCode, Json<ErrorResponse>)> {
    // Validate mood
    if !is_valid_mood(&req.mood) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: format!("Invalid mood: {}. Valid moods: suspense, eerie, upbeat, calm, ambient, jazz", req.mood),
            }),
        ));
    }

    let mut presets = state.presets.write().await;
    let existing = presets.get(&id).ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: format!("Preset not found: {}", id),
            }),
        )
    })?;

    let updated = SavedPreset {
        id: id.clone(),
        name: req.name,
        mood: req.mood,
        duration: req.duration,
        key: req.key,
        intensity: req.intensity,
        tempo: req.tempo,
        seed: req.seed,
        created_at: existing.created_at.clone(),
        last_generated: existing.last_generated.clone(),
    };

    presets.insert(id, updated.clone());
    drop(presets);

    if let Err(e) = state.save().await {
        eprintln!("Failed to save presets: {}", e);
    }

    Ok(Json(updated))
}

/// DELETE /api/presets/:id - Delete a preset.
pub async fn delete_preset(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let mut presets = state.presets.write().await;
    if presets.remove(&id).is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: format!("Preset not found: {}", id),
            }),
        ));
    }
    drop(presets);

    if let Err(e) = state.save().await {
        eprintln!("Failed to save presets: {}", e);
    }

    Ok(StatusCode::NO_CONTENT)
}

/// POST /api/generate/:id - Generate audio for a preset.
pub async fn generate_audio(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<GenerateResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Get the preset
    let presets = state.presets.read().await;
    let preset = presets.get(&id).ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: format!("Preset not found: {}", id),
            }),
        )
    })?.clone();
    drop(presets);

    // Generate unique filename
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S").to_string();
    let filename = format!("{}_{}.wav", id, timestamp);
    let output_path = state.output_dir.join(&filename);

    // Build CLI command
    let mut cmd = Command::new(std::env::current_exe().unwrap_or_else(|_| "midi-cli-rs".into()));
    cmd.arg("preset")
        .arg("-m")
        .arg(&preset.mood)
        .arg("-d")
        .arg(preset.duration.to_string())
        .arg("--intensity")
        .arg(preset.intensity.to_string())
        .arg("-t")
        .arg(preset.tempo.to_string())
        .arg("-s")
        .arg(preset.seed.to_string())
        .arg("-o")
        .arg(&output_path);

    if let Some(ref key) = preset.key {
        cmd.arg("-k").arg(key);
    }

    // Run generation
    let status = cmd.status().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to run generator: {}", e),
            }),
        )
    })?;

    if !status.success() {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Audio generation failed".to_string(),
            }),
        ));
    }

    // Update last_generated timestamp
    let generated_at = chrono::Utc::now().to_rfc3339();
    {
        let mut presets = state.presets.write().await;
        if let Some(p) = presets.get_mut(&id) {
            p.last_generated = Some(generated_at.clone());
        }
    }
    let _ = state.save().await;

    Ok(Json(GenerateResponse {
        preset_id: id,
        audio_url: format!("/audio/{}", filename),
        generated_at,
    }))
}

/// GET /api/moods - List available moods.
pub async fn list_moods() -> impl IntoResponse {
    Json(vec![
        MoodInfo { name: "suspense", key: "Am", description: "Tense mood with low drones and tremolo strings" },
        MoodInfo { name: "eerie", key: "Dm", description: "Creepy mood with sparse tones and diminished harmony" },
        MoodInfo { name: "upbeat", key: "C", description: "Energetic mood with rhythmic patterns" },
        MoodInfo { name: "calm", key: "G", description: "Peaceful mood with sustained pads and arpeggios" },
        MoodInfo { name: "ambient", key: "Em", description: "Atmospheric mood with drones and pentatonic tones" },
        MoodInfo { name: "jazz", key: "F", description: "Nightclub trio with walking bass and piano comping" },
    ])
}

#[derive(serde::Serialize)]
struct MoodInfo {
    name: &'static str,
    key: &'static str,
    description: &'static str,
}

/// Check if a mood name is valid.
fn is_valid_mood(mood: &str) -> bool {
    matches!(
        mood.to_lowercase().as_str(),
        "suspense" | "eerie" | "upbeat" | "calm" | "ambient" | "jazz"
    )
}
