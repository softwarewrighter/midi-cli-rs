//! FFI type definitions for native plugin interface.
//!
//! These types use `#[repr(C)]` for ABI stability across the plugin boundary.
//! Memory ownership follows the contract: plugin allocates, host calls free function.

use std::ffi::{c_char, c_void};

/// Plugin API version for compatibility checking.
/// Plugins must return this version from `plugin_info()`.
pub const PLUGIN_API_VERSION: u32 = 1;

/// Error codes returned by plugin functions.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginError {
    /// No error, operation succeeded
    Ok = 0,
    /// Invalid configuration parameter
    InvalidConfig = 1,
    /// Memory allocation failed
    AllocationFailed = 2,
    /// Internal plugin error
    InternalError = 3,
    /// API version mismatch
    VersionMismatch = 4,
    /// Required function not found
    MissingFunction = 5,
}

/// Plugin metadata returned by `plugin_info()`.
#[repr(C)]
pub struct PluginInfo {
    /// API version (must equal PLUGIN_API_VERSION)
    pub api_version: u32,
    /// Plugin name (null-terminated UTF-8)
    pub name: *const c_char,
    /// Plugin version string (null-terminated UTF-8)
    pub version: *const c_char,
    /// Plugin author (null-terminated UTF-8, may be null)
    pub author: *const c_char,
    /// Plugin description (null-terminated UTF-8, may be null)
    pub description: *const c_char,
    /// Reserved for future expansion
    pub _reserved: [*const c_void; 4],
}

/// Configuration passed to `plugin_generate()`.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct PluginConfig {
    /// Duration in seconds
    pub duration_secs: f64,
    /// Musical key root note (MIDI pitch, 0-127)
    pub key_root: u8,
    /// Key mode: 0 = major, 1 = minor
    pub key_mode: u8,
    /// Intensity level (0-100)
    pub intensity: u8,
    /// Random seed for reproducibility
    pub seed: u64,
    /// Tempo in BPM
    pub tempo: u16,
    /// Padding for alignment
    pub _pad: [u8; 4],
    /// Reserved for future expansion
    pub _reserved: [u64; 4],
}

/// A single note in C-compatible format.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct PluginNote {
    /// MIDI pitch (0-127)
    pub pitch: u8,
    /// Velocity (0-127)
    pub velocity: u8,
    /// Padding for alignment
    pub _pad: [u8; 6],
    /// Duration in beats
    pub duration: f64,
    /// Offset from sequence start in beats
    pub offset: f64,
}

/// A sequence of notes (one instrument track).
#[repr(C)]
pub struct PluginSequence {
    /// Pointer to array of notes
    pub notes: *const PluginNote,
    /// Number of notes in the array
    pub note_count: u32,
    /// GM instrument program number (0-127)
    pub instrument: u8,
    /// MIDI channel (0-15)
    pub channel: u8,
    /// Padding for alignment
    pub _pad: [u8; 2],
    /// Tempo in BPM (same as config, for convenience)
    pub tempo: u16,
    /// Reserved for future expansion
    pub _reserved: [u64; 2],
}

/// Result from `plugin_generate()`.
#[repr(C)]
pub struct PluginResult {
    /// Error code (0 = success)
    pub error: PluginError,
    /// Padding for alignment
    pub _pad: [u8; 4],
    /// Pointer to array of sequences
    pub sequences: *const PluginSequence,
    /// Number of sequences in the array
    pub sequence_count: u32,
    /// Error message if error != Ok (null-terminated UTF-8, may be null)
    pub error_message: *const c_char,
    /// Reserved for future expansion
    pub _reserved: [*const c_void; 2],
}

// Implement Default for PluginConfig to help with initialization
impl Default for PluginConfig {
    fn default() -> Self {
        Self {
            duration_secs: 5.0,
            key_root: 60, // C4
            key_mode: 0,  // Major
            intensity: 50,
            seed: 1,
            tempo: 120,
            _pad: [0; 4],
            _reserved: [0; 4],
        }
    }
}

impl PluginConfig {
    /// Create a new PluginConfig from preset configuration values.
    pub fn new(
        duration_secs: f64,
        key_root: u8,
        key_mode: u8,
        intensity: u8,
        seed: u64,
        tempo: u16,
    ) -> Self {
        Self {
            duration_secs,
            key_root,
            key_mode,
            intensity,
            seed,
            tempo,
            _pad: [0; 4],
            _reserved: [0; 4],
        }
    }
}

impl Default for PluginResult {
    fn default() -> Self {
        Self {
            error: PluginError::Ok,
            _pad: [0; 4],
            sequences: std::ptr::null(),
            sequence_count: 0,
            error_message: std::ptr::null(),
            _reserved: [std::ptr::null(); 2],
        }
    }
}

impl Default for PluginInfo {
    fn default() -> Self {
        Self {
            api_version: 0,
            name: std::ptr::null(),
            version: std::ptr::null(),
            author: std::ptr::null(),
            description: std::ptr::null(),
            _reserved: [std::ptr::null(); 4],
        }
    }
}

/// Safety trait marker for plugin function pointers.
///
/// # Safety
/// Implementing types must ensure that the function pointers are valid
/// and point to functions with the correct signatures.
pub unsafe trait PluginFunctions {
    /// Get plugin information.
    fn info(&self) -> PluginInfo;

    /// Generate music sequences.
    ///
    /// # Safety
    /// The returned PluginResult must be freed with `free_result`.
    unsafe fn generate(&self, config: *const PluginConfig) -> *mut PluginResult;

    /// Free a result returned by generate.
    ///
    /// # Safety
    /// The result pointer must have been returned by `generate` and not already freed.
    unsafe fn free_result(&self, result: *mut PluginResult);

    /// Optional: Validate configuration before generation.
    fn validate_config(&self, config: *const PluginConfig) -> PluginError;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::{align_of, size_of};

    #[test]
    fn test_plugin_note_layout() {
        // Verify expected sizes and alignment for FFI compatibility
        assert_eq!(size_of::<PluginNote>(), 24);
        assert_eq!(align_of::<PluginNote>(), 8);
    }

    #[test]
    fn test_plugin_config_default() {
        let config = PluginConfig::default();
        assert_eq!(config.duration_secs, 5.0);
        assert_eq!(config.key_root, 60);
        assert_eq!(config.tempo, 120);
    }

    #[test]
    fn test_plugin_error_repr() {
        assert_eq!(PluginError::Ok as i32, 0);
        assert_eq!(PluginError::InvalidConfig as i32, 1);
    }
}
