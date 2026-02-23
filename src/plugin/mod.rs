//! Native plugin system for loading custom mood generators.
//!
//! This module provides support for loading mood generators from
//! dynamic libraries (.dylib on macOS, .so on Linux, .dll on Windows).
//!
//! # Architecture
//!
//! Plugins are discovered via TOML configuration files that include a
//! `[pack.native]` section pointing to a shared library. The library
//! must export specific C-compatible functions.
//!
//! # Required Plugin Exports
//!
//! ```c
//! // Return plugin metadata
//! PluginInfo plugin_info(void);
//!
//! // Generate music sequences
//! PluginResult* plugin_generate(const PluginConfig* config);
//!
//! // Free result allocated by plugin_generate
//! void plugin_free_result(PluginResult* result);
//!
//! // Optional: validate configuration
//! PluginError plugin_validate_config(const PluginConfig* config);
//! ```
//!
//! # Example TOML Configuration
//!
//! ```toml
//! [pack]
//! name = "euclidean"
//! version = "1.0.0"
//! description = "Euclidean rhythm generator"
//!
//! [pack.native]
//! library = "libeuclidean_mood"  # Auto-resolves to .dylib/.so/.dll
//!
//! [[moods]]
//! name = "euclidean"
//! description = "Polyrhythmic patterns using Bjorklund's algorithm"
//! default_key = "Am"
//! default_tempo = 100
//! ```
//!
//! # Safety
//!
//! Loading native plugins involves unsafe operations. Plugins must:
//! - Use the correct API version (PLUGIN_API_VERSION)
//! - Return valid pointers from plugin_generate
//! - Properly free memory when plugin_free_result is called
//! - Be thread-safe (multiple generate calls may happen concurrently)

pub mod discovery;
pub mod ffi_types;
pub mod loader;

// Re-export commonly used types
pub use discovery::{discover_native_plugins, is_native_mood, parse_native_plugin, NativePluginInfo};
pub use ffi_types::{
    PluginConfig, PluginError, PluginInfo, PluginNote, PluginResult, PluginSequence,
    PLUGIN_API_VERSION,
};
pub use loader::{LoadedPlugin, PluginLoadError, PluginMetadata, PluginRegistry};

use crate::{NoteSequence, PresetConfig};
use std::path::Path;
use std::sync::OnceLock;

/// Global plugin registry for lazy-loaded plugins.
static PLUGIN_REGISTRY: OnceLock<PluginRegistry> = OnceLock::new();

/// Get the global plugin registry.
pub fn global_registry() -> &'static PluginRegistry {
    PLUGIN_REGISTRY.get_or_init(PluginRegistry::new)
}

/// Generate music using a native plugin.
///
/// This function:
/// 1. Looks up the mood in discovered native plugins
/// 2. Loads the plugin library if not already loaded
/// 3. Calls the plugin's generate function
/// 4. Returns the generated note sequences
///
/// # Arguments
/// * `mood_name` - The mood name to generate
/// * `config` - Generation configuration
/// * `moods_dir` - Directory containing plugin TOML files
///
/// # Returns
/// * `Ok(sequences)` - Generated note sequences
/// * `Err(error)` - If plugin not found or generation failed
pub fn generate_with_native_plugin(
    mood_name: &str,
    config: &PresetConfig,
    moods_dir: &Path,
) -> Result<Vec<NoteSequence>, PluginLoadError> {
    // Find the native plugin for this mood
    let plugin_info = is_native_mood(mood_name, moods_dir)
        .ok_or_else(|| PluginLoadError::NotFound(moods_dir.join(mood_name)))?;

    let registry = global_registry();

    // Load plugin if not already loaded
    if !registry.contains(&plugin_info.pack_name) {
        if !plugin_info.library_path.exists() {
            return Err(PluginLoadError::NotFound(plugin_info.library_path));
        }

        // Safety: We trust that plugins in the moods directory are valid
        unsafe {
            registry.load(&plugin_info.pack_name, &plugin_info.library_path)?;
        }
    }

    // Generate using the plugin
    registry.generate(&plugin_info.pack_name, config)
}

/// Check if a mood name is provided by a native plugin.
pub fn is_native_plugin_mood(mood_name: &str, moods_dir: &Path) -> bool {
    is_native_mood(mood_name, moods_dir).is_some()
}

/// List all native plugin moods in a directory.
pub fn list_native_plugin_moods(moods_dir: &Path) -> Vec<(String, String, String)> {
    let mut moods = Vec::new();

    for plugin in discover_native_plugins(moods_dir) {
        for mood_name in plugin.mood_names {
            moods.push((
                mood_name,
                plugin.pack_name.clone(),
                plugin
                    .description
                    .clone()
                    .unwrap_or_else(|| "Native plugin mood".to_string()),
            ));
        }
    }

    moods
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_global_registry_initialization() {
        let registry = global_registry();
        assert!(registry.list().is_empty());
    }

    #[test]
    fn test_is_native_plugin_mood_empty_dir() {
        let temp_dir = std::env::temp_dir().join("test_native_mood");
        let _ = std::fs::create_dir_all(&temp_dir);
        assert!(!is_native_plugin_mood("test", &temp_dir));
        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
