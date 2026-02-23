//! REST API handlers for the web server.

use crate::server::state::{
    AppState, ErrorResponse, GenerateResponse, MelodyRequest, PresetRequest, SavedMelody,
    SavedPreset,
};
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

    // Log the command being run
    eprintln!("[API] Running preset generation: {:?}", cmd);

    // Run generation and capture output
    let output = cmd.output().map_err(|e| {
        eprintln!("[API ERROR] Failed to spawn generator process: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to run generator: {}", e),
            }),
        )
    })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        eprintln!("[API ERROR] Preset generation failed:");
        eprintln!("  Exit code: {:?}", output.status.code());
        eprintln!("  Stdout: {}", stdout);
        eprintln!("  Stderr: {}", stderr);
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Audio generation failed: {}", stderr.trim()),
            }),
        ));
    }

    eprintln!("[API] Preset generation succeeded: {}", filename);

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

// ============================================================================
// Melody API endpoints
// ============================================================================

/// GET /api/melodies - List all saved melodies.
pub async fn list_melodies(State(state): State<Arc<AppState>>) -> Json<Vec<SavedMelody>> {
    let melodies = state.melodies.read().await;
    let mut list: Vec<SavedMelody> = melodies.values().cloned().collect();
    list.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Json(list)
}

/// POST /api/melodies - Create a new melody.
pub async fn create_melody(
    State(state): State<Arc<AppState>>,
    Json(req): Json<MelodyRequest>,
) -> Result<(StatusCode, Json<SavedMelody>), (StatusCode, Json<ErrorResponse>)> {
    let id = uuid::Uuid::new_v4().to_string();
    let melody = req.into_melody(id);

    let mut melodies = state.melodies.write().await;
    melodies.insert(melody.id.clone(), melody.clone());
    drop(melodies);

    if let Err(e) = state.save().await {
        eprintln!("Failed to save melodies: {}", e);
    }

    Ok((StatusCode::CREATED, Json(melody)))
}

/// GET /api/melodies/:id - Get a single melody.
pub async fn get_melody(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<SavedMelody>, (StatusCode, Json<ErrorResponse>)> {
    let melodies = state.melodies.read().await;
    melodies
        .get(&id)
        .cloned()
        .map(Json)
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: format!("Melody not found: {}", id),
                }),
            )
        })
}

/// PUT /api/melodies/:id - Update an existing melody.
pub async fn update_melody(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<MelodyRequest>,
) -> Result<Json<SavedMelody>, (StatusCode, Json<ErrorResponse>)> {
    let mut melodies = state.melodies.write().await;
    let existing = melodies.get(&id).ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: format!("Melody not found: {}", id),
            }),
        )
    })?;

    let updated = SavedMelody {
        id: id.clone(),
        name: req.name,
        notes: req.notes,
        key: req.key,
        tempo: req.tempo,
        instrument: req.instrument,
        attack: req.attack,
        decay: req.decay,
        created_at: existing.created_at.clone(),
        last_generated: existing.last_generated.clone(),
    };

    melodies.insert(id, updated.clone());
    drop(melodies);

    if let Err(e) = state.save().await {
        eprintln!("Failed to save melodies: {}", e);
    }

    Ok(Json(updated))
}

/// DELETE /api/melodies/:id - Delete a melody.
pub async fn delete_melody(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let mut melodies = state.melodies.write().await;
    if melodies.remove(&id).is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: format!("Melody not found: {}", id),
            }),
        ));
    }
    drop(melodies);

    if let Err(e) = state.save().await {
        eprintln!("Failed to save melodies: {}", e);
    }

    Ok(StatusCode::NO_CONTENT)
}

/// POST /api/melodies/:id/generate - Generate audio for a melody.
pub async fn generate_melody_audio(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<GenerateResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Get the melody
    let melodies = state.melodies.read().await;
    let melody = melodies.get(&id).ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: format!("Melody not found: {}", id),
            }),
        )
    })?.clone();
    drop(melodies);

    // Generate unique filename
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S").to_string();
    let filename = format!("melody_{}_{}.wav", id, timestamp);
    let output_path = state.output_dir.join(&filename);

    // Convert notes to CLI format: "PITCH:DURATION:VELOCITY[@OFFSET],..."
    // Rests are handled by advancing the offset without adding a note
    let mut notes_str = String::new();
    let mut offset = 0.0f64;
    for note in &melody.notes {
        if note.pitch == "rest" {
            // For rests, just advance the offset (silence)
            offset += note.duration;
            continue;
        }
        // Only add comma separator if we already have notes
        if !notes_str.is_empty() {
            notes_str.push(',');
        }
        notes_str.push_str(&format!(
            "{}:{}:{}@{}",
            note.pitch, note.duration, note.velocity, offset
        ));
        offset += note.duration;
    }

    if notes_str.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Melody has no playable notes".to_string(),
            }),
        ));
    }

    // Build CLI command
    let mut cmd = Command::new(std::env::current_exe().unwrap_or_else(|_| "midi-cli-rs".into()));
    cmd.arg("generate")
        .arg("--notes")
        .arg(&notes_str)
        .arg("-i")
        .arg(&melody.instrument)
        .arg("-t")
        .arg(melody.tempo.to_string())
        .arg("-o")
        .arg(&output_path);

    // Log the command being run
    eprintln!("[API] Running melody generation: {:?}", cmd);
    eprintln!("[API] Notes string: {}", notes_str);

    // Run generation and capture output
    let output = cmd.output().map_err(|e| {
        eprintln!("[API ERROR] Failed to spawn generator process: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to run generator: {}", e),
            }),
        )
    })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        eprintln!("[API ERROR] Melody generation failed:");
        eprintln!("  Exit code: {:?}", output.status.code());
        eprintln!("  Stdout: {}", stdout);
        eprintln!("  Stderr: {}", stderr);
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Audio generation failed: {}", stderr.trim()),
            }),
        ));
    }

    eprintln!("[API] Melody generation succeeded: {}", filename);

    // Update last_generated timestamp
    let generated_at = chrono::Utc::now().to_rfc3339();
    {
        let mut melodies = state.melodies.write().await;
        if let Some(m) = melodies.get_mut(&id) {
            m.last_generated = Some(generated_at.clone());
        }
    }
    let _ = state.save().await;

    Ok(Json(GenerateResponse {
        preset_id: id,
        audio_url: format!("/audio/{}", filename),
        generated_at,
    }))
}

/// GET /api/instruments - List available instruments.
pub async fn list_instruments() -> impl IntoResponse {
    Json(midi_cli_rs::INSTRUMENT_MAP
        .iter()
        .map(|(name, num)| InstrumentInfo { name, program: *num })
        .collect::<Vec<_>>())
}

#[derive(serde::Serialize)]
struct InstrumentInfo {
    name: &'static str,
    program: u8,
}

// ============================================================================
// Plugin (MoodPack) API
// ============================================================================

/// GET /api/plugins - List installed mood pack plugins.
pub async fn list_plugins() -> impl IntoResponse {
    let moods_dir = get_moods_dir();

    if !moods_dir.exists() {
        return Json(Vec::<MoodPackInfo>::new());
    }

    let mut plugins = Vec::new();

    if let Ok(entries) = std::fs::read_dir(&moods_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "toml") {
                if let Some(info) = parse_mood_pack_info(&path) {
                    plugins.push(info);
                }
            }
        }
    }

    // Sort by name
    plugins.sort_by(|a, b| a.name.cmp(&b.name));
    Json(plugins)
}

/// POST /api/plugins - Upload a new mood pack plugin.
pub async fn upload_plugin(
    Json(req): Json<UploadPluginRequest>,
) -> Result<(StatusCode, Json<MoodPackInfo>), (StatusCode, Json<ErrorResponse>)> {
    // Validate TOML
    let pack: toml::Value = toml::from_str(&req.content).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: format!("Invalid TOML: {}", e),
            }),
        )
    })?;

    // Extract pack name
    let pack_name = pack
        .get("pack")
        .and_then(|p| p.get("name"))
        .and_then(|n| n.as_str())
        .ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: "Missing [pack] name field".to_string(),
                }),
            )
        })?;

    // Validate pack has moods
    let moods = pack.get("moods").and_then(|m| m.as_array()).ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Missing [[moods]] array".to_string(),
            }),
        )
    })?;

    if moods.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Mood pack must contain at least one mood".to_string(),
            }),
        ));
    }

    // Create moods directory if needed
    let moods_dir = get_moods_dir();
    std::fs::create_dir_all(&moods_dir).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to create moods directory: {}", e),
            }),
        )
    })?;

    // Write the file
    let filename = format!("{}.toml", pack_name);
    let filepath = moods_dir.join(&filename);
    std::fs::write(&filepath, &req.content).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to write plugin file: {}", e),
            }),
        )
    })?;

    // Parse and return info
    let info = parse_mood_pack_info(&filepath).ok_or_else(|| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Failed to parse saved plugin".to_string(),
            }),
        )
    })?;

    Ok((StatusCode::CREATED, Json(info)))
}

/// DELETE /api/plugins/:name - Remove a mood pack plugin.
pub async fn delete_plugin(
    Path(name): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let moods_dir = get_moods_dir();
    let filepath = moods_dir.join(format!("{}.toml", name));

    if !filepath.exists() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: format!("Plugin not found: {}", name),
            }),
        ));
    }

    std::fs::remove_file(&filepath).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to delete plugin: {}", e),
            }),
        )
    })?;

    Ok(StatusCode::NO_CONTENT)
}

/// Get the moods plugin directory path.
fn get_moods_dir() -> std::path::PathBuf {
    if let Some(home) = std::env::var_os("HOME") {
        std::path::PathBuf::from(home).join(".midi-cli-rs").join("moods")
    } else {
        std::path::PathBuf::from(".midi-cli-rs").join("moods")
    }
}

/// Parse a mood pack TOML file and extract info.
fn parse_mood_pack_info(path: &std::path::Path) -> Option<MoodPackInfo> {
    let content = std::fs::read_to_string(path).ok()?;
    let pack: toml::Value = toml::from_str(&content).ok()?;

    let pack_meta = pack.get("pack")?;
    let name = pack_meta.get("name")?.as_str()?.to_string();
    let version = pack_meta
        .get("version")
        .and_then(|v| v.as_str())
        .unwrap_or("1.0")
        .to_string();
    let author = pack_meta
        .get("author")
        .and_then(|a| a.as_str())
        .map(|s| s.to_string());
    let description = pack_meta
        .get("description")
        .and_then(|d| d.as_str())
        .map(|s| s.to_string());

    let moods_array = pack.get("moods")?.as_array()?;
    let moods: Vec<PluginMoodInfo> = moods_array
        .iter()
        .filter_map(|m| {
            let name = m.get("name")?.as_str()?.to_string();
            let default_key = m
                .get("default_key")
                .and_then(|k| k.as_str())
                .unwrap_or("C")
                .to_string();
            let default_tempo = m
                .get("default_tempo")
                .and_then(|t| t.as_integer())
                .unwrap_or(120) as u16;
            let description = m
                .get("description")
                .and_then(|d| d.as_str())
                .map(|s| s.to_string());
            Some(PluginMoodInfo {
                name,
                default_key,
                default_tempo,
                description,
            })
        })
        .collect();

    Some(MoodPackInfo {
        name,
        version,
        author,
        description,
        mood_count: moods.len(),
        moods,
        file_path: Some(path.to_string_lossy().to_string()),
    })
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct MoodPackInfo {
    pub name: String,
    pub version: String,
    pub author: Option<String>,
    pub description: Option<String>,
    pub mood_count: usize,
    pub moods: Vec<PluginMoodInfo>,
    pub file_path: Option<String>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct PluginMoodInfo {
    pub name: String,
    pub default_key: String,
    pub default_tempo: u16,
    pub description: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct UploadPluginRequest {
    pub content: String,
    #[serde(default)]
    pub filename: Option<String>,
}
