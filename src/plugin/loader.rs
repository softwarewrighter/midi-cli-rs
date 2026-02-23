//! Plugin loader for native dynamic libraries.
//!
//! Handles loading .dylib/.so/.dll files and calling plugin functions safely.

use super::ffi_types::*;
use crate::{Key, Note, NoteSequence, PresetConfig};
use std::collections::HashMap;
use std::ffi::CStr;
use std::path::{Path, PathBuf};
use std::sync::RwLock;
use thiserror::Error;

/// Errors that can occur when loading or using plugins.
#[derive(Debug, Error)]
pub enum PluginLoadError {
    #[error("Library not found: {0}")]
    NotFound(PathBuf),

    #[error("Failed to load library: {0}")]
    LoadFailed(String),

    #[error("Missing required symbol: {0}")]
    MissingSymbol(String),

    #[error("API version mismatch: plugin has {0}, expected {1}")]
    VersionMismatch(u32, u32),

    #[error("Plugin returned error: {0}")]
    PluginError(String),

    #[error("Invalid plugin data: {0}")]
    InvalidData(String),
}

/// A loaded native plugin with its library handle and function pointers.
pub struct LoadedPlugin {
    /// Keep the library alive while function pointers are in use
    #[allow(dead_code)]
    library: libloading::Library,

    /// Plugin metadata
    pub info: PluginMetadata,

    /// Path to the loaded library
    pub path: PathBuf,

    // Function pointers
    fn_info: unsafe extern "C" fn() -> PluginInfo,
    fn_generate: unsafe extern "C" fn(*const PluginConfig) -> *mut PluginResult,
    fn_free_result: unsafe extern "C" fn(*mut PluginResult),
    fn_validate: Option<unsafe extern "C" fn(*const PluginConfig) -> PluginError>,
}

/// Plugin metadata extracted from PluginInfo.
#[derive(Debug, Clone)]
pub struct PluginMetadata {
    pub name: String,
    pub version: String,
    pub author: Option<String>,
    pub description: Option<String>,
}

impl LoadedPlugin {
    /// Load a plugin from a dynamic library file.
    ///
    /// # Safety
    /// The library must export the required plugin symbols with correct signatures.
    pub unsafe fn load(path: impl AsRef<Path>) -> Result<Self, PluginLoadError> {
        let path = path.as_ref();

        // Resolve platform-specific library path
        let resolved_path = resolve_library_path(path);
        if !resolved_path.exists() {
            return Err(PluginLoadError::NotFound(resolved_path));
        }

        // Load the library
        let library = unsafe {
            libloading::Library::new(&resolved_path)
                .map_err(|e| PluginLoadError::LoadFailed(e.to_string()))?
        };

        // Load required symbols and immediately dereference to get raw fn pointers
        let fn_info: unsafe extern "C" fn() -> PluginInfo = unsafe {
            *library
                .get::<unsafe extern "C" fn() -> PluginInfo>(b"plugin_info\0")
                .map_err(|_| PluginLoadError::MissingSymbol("plugin_info".to_string()))?
        };

        let fn_generate: unsafe extern "C" fn(*const PluginConfig) -> *mut PluginResult = unsafe {
            *library
                .get::<unsafe extern "C" fn(*const PluginConfig) -> *mut PluginResult>(
                    b"plugin_generate\0",
                )
                .map_err(|_| PluginLoadError::MissingSymbol("plugin_generate".to_string()))?
        };

        let fn_free_result: unsafe extern "C" fn(*mut PluginResult) = unsafe {
            *library
                .get::<unsafe extern "C" fn(*mut PluginResult)>(b"plugin_free_result\0")
                .map_err(|_| PluginLoadError::MissingSymbol("plugin_free_result".to_string()))?
        };

        // Load optional validate symbol
        let fn_validate: Option<unsafe extern "C" fn(*const PluginConfig) -> PluginError> =
            unsafe {
                library
                    .get::<unsafe extern "C" fn(*const PluginConfig) -> PluginError>(
                        b"plugin_validate_config\0",
                    )
                    .ok()
                    .map(|s| *s)
            };

        // Get and verify plugin info
        let raw_info = unsafe { fn_info() };
        if raw_info.api_version != PLUGIN_API_VERSION {
            return Err(PluginLoadError::VersionMismatch(
                raw_info.api_version,
                PLUGIN_API_VERSION,
            ));
        }

        // Extract metadata from raw info
        let info = PluginMetadata {
            name: unsafe { cstr_to_string(raw_info.name) }
                .unwrap_or_else(|| "unknown".to_string()),
            version: unsafe { cstr_to_string(raw_info.version) }
                .unwrap_or_else(|| "0.0.0".to_string()),
            author: unsafe { cstr_to_string(raw_info.author) },
            description: unsafe { cstr_to_string(raw_info.description) },
        };

        Ok(Self {
            library,
            info,
            path: resolved_path,
            fn_info,
            fn_generate,
            fn_free_result,
            fn_validate,
        })
    }

    /// Generate music using the plugin.
    pub fn generate(&self, config: &PresetConfig) -> Result<Vec<NoteSequence>, PluginLoadError> {
        // Convert Rust config to C config
        let c_config = preset_config_to_plugin_config(config);

        // Optionally validate
        if let Some(validate) = self.fn_validate {
            let err = unsafe { validate(&c_config) };
            if err != PluginError::Ok {
                return Err(PluginLoadError::PluginError(format!(
                    "Validation failed: {:?}",
                    err
                )));
            }
        }

        // Call generate
        let result_ptr = unsafe { (self.fn_generate)(&c_config) };
        if result_ptr.is_null() {
            return Err(PluginLoadError::PluginError(
                "Plugin returned null result".to_string(),
            ));
        }

        // Process result
        let result = unsafe { &*result_ptr };

        // Check for errors
        if result.error != PluginError::Ok {
            let error_msg = unsafe { cstr_to_string(result.error_message) }
                .unwrap_or_else(|| format!("{:?}", result.error));

            // Free the result before returning error
            unsafe { (self.fn_free_result)(result_ptr) };

            return Err(PluginLoadError::PluginError(error_msg));
        }

        // Convert sequences to Rust types
        let sequences = unsafe { convert_plugin_sequences(result, config.tempo) }?;

        // Free the plugin result
        unsafe { (self.fn_free_result)(result_ptr) };

        Ok(sequences)
    }

    /// Get plugin info (refreshed from the plugin).
    pub fn get_info(&self) -> PluginMetadata {
        let raw_info = unsafe { (self.fn_info)() };
        PluginMetadata {
            name: unsafe { cstr_to_string(raw_info.name) }
                .unwrap_or_else(|| "unknown".to_string()),
            version: unsafe { cstr_to_string(raw_info.version) }
                .unwrap_or_else(|| "0.0.0".to_string()),
            author: unsafe { cstr_to_string(raw_info.author) },
            description: unsafe { cstr_to_string(raw_info.description) },
        }
    }
}

// LoadedPlugin is Send because:
// - The library handle is valid as long as the plugin lives
// - Function pointers are valid for the library's lifetime
unsafe impl Send for LoadedPlugin {}

// LoadedPlugin is Sync because:
// - The function pointers are stateless and can be called from any thread
// - Plugin implementations must be thread-safe for their own state
unsafe impl Sync for LoadedPlugin {}

/// Thread-safe registry of loaded plugins.
pub struct PluginRegistry {
    plugins: RwLock<HashMap<String, LoadedPlugin>>,
}

impl PluginRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            plugins: RwLock::new(HashMap::new()),
        }
    }

    /// Load and register a plugin.
    ///
    /// # Safety
    /// The library must export valid plugin symbols.
    pub unsafe fn load(&self, name: &str, path: impl AsRef<Path>) -> Result<(), PluginLoadError> {
        let plugin = unsafe { LoadedPlugin::load(path)? };

        let mut plugins = self.plugins.write().expect("Plugin registry lock poisoned");
        plugins.insert(name.to_string(), plugin);
        Ok(())
    }

    /// Generate using a named plugin.
    pub fn generate(
        &self,
        name: &str,
        config: &PresetConfig,
    ) -> Result<Vec<NoteSequence>, PluginLoadError> {
        let plugins = self.plugins.read().expect("Plugin registry lock poisoned");
        let plugin = plugins
            .get(name)
            .ok_or_else(|| PluginLoadError::NotFound(PathBuf::from(name)))?;
        plugin.generate(config)
    }

    /// Check if a plugin is loaded.
    pub fn contains(&self, name: &str) -> bool {
        let plugins = self.plugins.read().expect("Plugin registry lock poisoned");
        plugins.contains_key(name)
    }

    /// List all loaded plugin names.
    pub fn list(&self) -> Vec<String> {
        let plugins = self.plugins.read().expect("Plugin registry lock poisoned");
        plugins.keys().cloned().collect()
    }

    /// Unload a plugin.
    pub fn unload(&self, name: &str) -> bool {
        let mut plugins = self.plugins.write().expect("Plugin registry lock poisoned");
        plugins.remove(name).is_some()
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Helper functions
// ============================================================================

/// Convert a C string pointer to a Rust String.
unsafe fn cstr_to_string(ptr: *const std::ffi::c_char) -> Option<String> {
    if ptr.is_null() {
        return None;
    }
    unsafe { CStr::from_ptr(ptr).to_str().ok().map(|s| s.to_string()) }
}

/// Resolve library path with platform-specific extension.
fn resolve_library_path(path: &Path) -> PathBuf {
    // If the path already has an extension, use it as-is
    if path.extension().is_some() {
        return path.to_path_buf();
    }

    // Add platform-specific extension
    #[cfg(target_os = "macos")]
    let ext = "dylib";
    #[cfg(target_os = "linux")]
    let ext = "so";
    #[cfg(target_os = "windows")]
    let ext = "dll";
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    let ext = "so"; // Fallback

    let mut resolved = path.to_path_buf();
    let filename = path
        .file_name()
        .and_then(|f| f.to_str())
        .unwrap_or("plugin");

    // Handle lib prefix on Unix-like systems
    #[cfg(not(target_os = "windows"))]
    {
        if !filename.starts_with("lib") {
            resolved.set_file_name(format!("lib{}.{}", filename, ext));
            return resolved;
        }
    }

    resolved.set_extension(ext);
    resolved
}

/// Convert PresetConfig to PluginConfig.
fn preset_config_to_plugin_config(config: &PresetConfig) -> PluginConfig {
    let (key_root, key_mode) = key_to_root_mode(&config.key);
    PluginConfig::new(
        config.duration_secs,
        key_root,
        key_mode,
        config.intensity,
        config.seed,
        config.tempo,
    )
}

/// Convert Key enum to (root MIDI note, mode).
fn key_to_root_mode(key: &Key) -> (u8, u8) {
    let root = key.root();
    let mode = match key {
        Key::C | Key::D | Key::Eb | Key::E | Key::F | Key::G | Key::A | Key::Bb | Key::B => 0, // Major
        Key::Cm | Key::Dm | Key::Ebm | Key::Em | Key::Fm | Key::Gm | Key::Am | Key::Bbm | Key::Bm => 1, // Minor
    };
    (root, mode)
}

/// Convert plugin sequences to Rust NoteSequence.
unsafe fn convert_plugin_sequences(
    result: &PluginResult,
    tempo: u16,
) -> Result<Vec<NoteSequence>, PluginLoadError> {
    if result.sequences.is_null() || result.sequence_count == 0 {
        return Ok(Vec::new());
    }

    let seq_slice = unsafe {
        std::slice::from_raw_parts(result.sequences, result.sequence_count as usize)
    };

    let mut sequences = Vec::with_capacity(seq_slice.len());

    for plugin_seq in seq_slice {
        if plugin_seq.notes.is_null() {
            continue;
        }

        let notes_slice = unsafe {
            std::slice::from_raw_parts(plugin_seq.notes, plugin_seq.note_count as usize)
        };

        let notes: Vec<Note> = notes_slice
            .iter()
            .map(|pn| Note::new(pn.pitch, pn.duration, pn.velocity, pn.offset))
            .collect();

        let mut seq = NoteSequence::new(
            notes,
            plugin_seq.instrument,
            if plugin_seq.tempo > 0 {
                plugin_seq.tempo
            } else {
                tempo
            },
        );
        seq.channel = plugin_seq.channel;
        sequences.push(seq);
    }

    Ok(sequences)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_library_path_with_extension() {
        let path = PathBuf::from("/usr/lib/libfoo.dylib");
        assert_eq!(resolve_library_path(&path), path);
    }

    #[test]
    fn test_resolve_library_path_without_extension() {
        let path = PathBuf::from("/usr/lib/foo");
        let resolved = resolve_library_path(&path);
        #[cfg(target_os = "macos")]
        assert_eq!(resolved, PathBuf::from("/usr/lib/libfoo.dylib"));
        #[cfg(target_os = "linux")]
        assert_eq!(resolved, PathBuf::from("/usr/lib/libfoo.so"));
    }

    #[test]
    fn test_key_to_root_mode() {
        assert_eq!(key_to_root_mode(&Key::C), (60, 0)); // C4 major
        assert_eq!(key_to_root_mode(&Key::Am), (69, 1)); // A4 minor
        assert_eq!(key_to_root_mode(&Key::G), (67, 0)); // G4 major
    }

    #[test]
    fn test_preset_config_to_plugin_config() {
        let preset = PresetConfig {
            duration_secs: 10.0,
            key: Key::Dm,
            intensity: 75,
            seed: 42,
            tempo: 140,
        };
        let plugin = preset_config_to_plugin_config(&preset);
        assert_eq!(plugin.duration_secs, 10.0);
        assert_eq!(plugin.key_root, 62); // D4
        assert_eq!(plugin.key_mode, 1); // minor
        assert_eq!(plugin.intensity, 75);
        assert_eq!(plugin.seed, 42);
        assert_eq!(plugin.tempo, 140);
    }

    #[test]
    fn test_plugin_registry_basic() {
        let registry = PluginRegistry::new();
        assert!(!registry.contains("test"));
        assert!(registry.list().is_empty());
    }
}
