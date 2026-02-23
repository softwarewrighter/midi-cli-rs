//! Web server module for the MIDI CLI web UI.
//!
//! This module provides an Axum-based REST API server that serves:
//! - REST API endpoints for preset CRUD and audio generation
//! - Static files for the Yew WASM frontend
//! - Generated audio files for playback

pub mod api;
pub mod state;

use axum::{
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use std::path::PathBuf;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;

use state::AppState;

/// Run the web server on the specified port.
pub async fn run_server(
    port: u16,
    static_dir: PathBuf,
    data_dir: Option<PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    let state = AppState::load_or_create(data_dir)?;

    // Build the API routes
    let api_routes = Router::new()
        // Preset routes
        .route("/presets", get(api::list_presets).post(api::create_preset))
        .route(
            "/presets/:id",
            get(api::get_preset)
                .put(api::update_preset)
                .delete(api::delete_preset),
        )
        .route("/generate/:id", post(api::generate_audio))
        .route("/moods", get(api::list_moods))
        // Melody routes
        .route("/melodies", get(api::list_melodies).post(api::create_melody))
        .route(
            "/melodies/:id",
            get(api::get_melody)
                .put(api::update_melody)
                .delete(api::delete_melody),
        )
        .route("/melodies/:id/generate", post(api::generate_melody_audio))
        .route("/instruments", get(api::list_instruments))
        // Plugin routes
        .route("/plugins", get(api::list_plugins).post(api::upload_plugin))
        .route("/plugins/:name", axum::routing::delete(api::delete_plugin));

    // CORS configuration for development
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Build the main app
    let output_dir = state.output_dir.clone();
    let app = Router::new()
        .nest("/api", api_routes)
        .nest_service("/audio", ServeDir::new(&output_dir))
        .fallback_service(ServeDir::new(&static_dir).append_index_html_on_directories(true))
        .layer(cors)
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    eprintln!("Starting web server at http://{}", addr);
    eprintln!("  Static files: {}", static_dir.display());
    eprintln!("  Audio output: {}", output_dir.display());
    eprintln!("Open in browser to use the web UI");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
