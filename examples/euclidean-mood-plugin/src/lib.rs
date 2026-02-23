//! Euclidean Rhythm Plugin for midi-cli-rs
//!
//! Implements Bjorklund's algorithm to create polyrhythmic patterns.
//! Euclidean rhythms distribute k pulses across n steps as evenly as possible.
//!
//! Examples:
//! - E(5,8) = [x . x . x . x x] - Cuban tresillo
//! - E(3,8) = [x . . x . . x .] - Cuban cinquillo
//! - E(7,16) = [x . x . x . x . x . x . x . x .] - Brazilian samba

use std::ffi::{c_char, c_void, CString};
use std::ptr;

// ============================================================================
// FFI Types (must match midi-cli-rs/src/plugin/ffi_types.rs)
// ============================================================================

const PLUGIN_API_VERSION: u32 = 1;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginError {
    Ok = 0,
    InvalidConfig = 1,
    AllocationFailed = 2,
    InternalError = 3,
    VersionMismatch = 4,
    MissingFunction = 5,
}

#[repr(C)]
pub struct PluginInfo {
    pub api_version: u32,
    pub name: *const c_char,
    pub version: *const c_char,
    pub author: *const c_char,
    pub description: *const c_char,
    pub _reserved: [*const c_void; 4],
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct PluginConfig {
    pub duration_secs: f64,
    pub key_root: u8,
    pub key_mode: u8,
    pub intensity: u8,
    pub seed: u64,
    pub tempo: u16,
    pub _pad: [u8; 4],
    pub _reserved: [u64; 4],
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct PluginNote {
    pub pitch: u8,
    pub velocity: u8,
    pub _pad: [u8; 6],
    pub duration: f64,
    pub offset: f64,
}

#[repr(C)]
pub struct PluginSequence {
    pub notes: *const PluginNote,
    pub note_count: u32,
    pub instrument: u8,
    pub channel: u8,
    pub _pad: [u8; 2],
    pub tempo: u16,
    pub _reserved: [u64; 2],
}

#[repr(C)]
pub struct PluginResult {
    pub error: PluginError,
    pub _pad: [u8; 4],
    pub sequences: *const PluginSequence,
    pub sequence_count: u32,
    pub error_message: *const c_char,
    pub _reserved: [*const c_void; 2],
}

// ============================================================================
// Static strings for plugin info
// ============================================================================

static PLUGIN_NAME: &[u8] = b"euclidean\0";
static PLUGIN_VERSION: &[u8] = b"1.0.0\0";
static PLUGIN_AUTHOR: &[u8] = b"midi-cli-rs contributors\0";
static PLUGIN_DESC: &[u8] = b"Euclidean rhythm generator using Bjorklund's algorithm\0";

// ============================================================================
// Plugin exports
// ============================================================================

#[no_mangle]
pub extern "C" fn plugin_info() -> PluginInfo {
    PluginInfo {
        api_version: PLUGIN_API_VERSION,
        name: PLUGIN_NAME.as_ptr() as *const c_char,
        version: PLUGIN_VERSION.as_ptr() as *const c_char,
        author: PLUGIN_AUTHOR.as_ptr() as *const c_char,
        description: PLUGIN_DESC.as_ptr() as *const c_char,
        _reserved: [ptr::null(); 4],
    }
}

#[no_mangle]
pub extern "C" fn plugin_generate(config: *const PluginConfig) -> *mut PluginResult {
    if config.is_null() {
        return create_error_result("Null config pointer");
    }

    let config = unsafe { &*config };

    match generate_euclidean(config) {
        Ok((sequences, notes_storage)) => {
            // Box the sequences and notes so they live on the heap
            let sequences_box = Box::new(sequences);
            let notes_box = Box::new(notes_storage);

            let result = Box::new(PluginResult {
                error: PluginError::Ok,
                _pad: [0; 4],
                sequences: sequences_box.as_ptr(),
                sequence_count: sequences_box.len() as u32,
                error_message: ptr::null(),
                _reserved: [ptr::null(); 2],
            });

            // Leak the boxes - they will be freed in plugin_free_result
            Box::leak(sequences_box);
            Box::leak(notes_box);

            Box::into_raw(result)
        }
        Err(msg) => create_error_result(&msg),
    }
}

#[no_mangle]
pub extern "C" fn plugin_free_result(result: *mut PluginResult) {
    if result.is_null() {
        return;
    }

    unsafe {
        let result = Box::from_raw(result);

        // Free error message if present
        if !result.error_message.is_null() {
            drop(CString::from_raw(result.error_message as *mut c_char));
        }

        // Free sequences and notes
        if !result.sequences.is_null() && result.sequence_count > 0 {
            let sequences = std::slice::from_raw_parts(
                result.sequences,
                result.sequence_count as usize,
            );

            for seq in sequences {
                if !seq.notes.is_null() && seq.note_count > 0 {
                    // Reconstruct the Vec<PluginNote> and drop it
                    drop(Vec::from_raw_parts(
                        seq.notes as *mut PluginNote,
                        seq.note_count as usize,
                        seq.note_count as usize,
                    ));
                }
            }

            // Reconstruct and drop the sequences Vec
            drop(Vec::from_raw_parts(
                result.sequences as *mut PluginSequence,
                result.sequence_count as usize,
                result.sequence_count as usize,
            ));
        }
    }
}

#[no_mangle]
pub extern "C" fn plugin_validate_config(config: *const PluginConfig) -> PluginError {
    if config.is_null() {
        return PluginError::InvalidConfig;
    }

    let config = unsafe { &*config };

    if config.duration_secs <= 0.0 {
        return PluginError::InvalidConfig;
    }
    if config.tempo == 0 || config.tempo > 300 {
        return PluginError::InvalidConfig;
    }

    PluginError::Ok
}

// ============================================================================
// Euclidean rhythm generation
// ============================================================================

/// Bjorklund's algorithm: distribute k pulses across n steps as evenly as possible.
fn bjorklund(pulses: usize, steps: usize) -> Vec<bool> {
    if pulses >= steps {
        return vec![true; steps];
    }
    if pulses == 0 {
        return vec![false; steps];
    }

    // Initialize groups
    let mut groups: Vec<Vec<bool>> = Vec::new();
    for _ in 0..pulses {
        groups.push(vec![true]);
    }
    let mut remainder: Vec<Vec<bool>> = Vec::new();
    for _ in pulses..steps {
        remainder.push(vec![false]);
    }

    // Iteratively distribute
    while remainder.len() > 1 {
        let n = groups.len().min(remainder.len());
        for i in 0..n {
            if let Some(rem) = remainder.pop() {
                groups[i].extend(rem);
            }
        }
        if remainder.is_empty() && groups.len() > 1 {
            let split_at = groups.len() / 2;
            remainder = groups.split_off(split_at);
        }
    }

    // Flatten
    let mut result: Vec<bool> = groups.into_iter().flatten().collect();
    result.extend(remainder.into_iter().flatten());
    result
}

/// Simple pseudo-random number generator (xorshift64)
struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 { 1 } else { seed },
        }
    }

    fn next(&mut self) -> u64 {
        self.state ^= self.state << 13;
        self.state ^= self.state >> 7;
        self.state ^= self.state << 17;
        self.state
    }

    fn next_range(&mut self, min: u64, max: u64) -> u64 {
        min + (self.next() % (max - min + 1))
    }
}

/// Generate Euclidean rhythm sequences.
fn generate_euclidean(
    config: &PluginConfig,
) -> Result<(Vec<PluginSequence>, Vec<Vec<PluginNote>>), String> {
    let mut rng = SimpleRng::new(config.seed);

    // Calculate total beats
    let total_beats = (config.duration_secs * config.tempo as f64 / 60.0).ceil() as usize;
    if total_beats == 0 {
        return Err("Duration too short".to_string());
    }

    // Determine number of layers based on intensity
    let num_layers = if config.intensity < 30 {
        1
    } else if config.intensity < 70 {
        2
    } else {
        3
    };

    // Get scale notes based on key
    let scale = get_scale(config.key_root, config.key_mode == 1);

    // Generate layers
    let mut sequences = Vec::with_capacity(num_layers);
    let mut all_notes: Vec<Vec<PluginNote>> = Vec::with_capacity(num_layers);

    // Layer configurations: (pulses, steps, instrument, octave_offset, velocity_base)
    let layer_configs: [(usize, usize, u8, i8, u8); 3] = [
        // Bass layer: sparse, low octave
        (rng.next_range(3, 5) as usize, rng.next_range(8, 16) as usize, 33, -12, 90),
        // Melody layer: moderate density, mid octave
        (rng.next_range(5, 8) as usize, rng.next_range(8, 12) as usize, 0, 0, 80),
        // High layer: denser, high octave (vibraphone)
        (rng.next_range(7, 11) as usize, rng.next_range(12, 16) as usize, 11, 12, 70),
    ];

    for (layer_idx, (pulses, steps, instrument, octave_offset, velocity_base)) in
        layer_configs.iter().enumerate().take(num_layers)
    {
        // Generate Euclidean pattern
        let pattern = bjorklund(*pulses, *steps);

        // Convert pattern to notes
        let mut notes = Vec::new();
        let step_duration = 0.5; // Each step is an eighth note

        // Calculate how many pattern repetitions fit in the duration
        let pattern_duration = *steps as f64 * step_duration;
        let repetitions = ((total_beats as f64 / pattern_duration).ceil() as usize).max(1);

        for rep in 0..repetitions {
            for (step_idx, &is_pulse) in pattern.iter().enumerate() {
                if is_pulse {
                    let offset = (rep * *steps + step_idx) as f64 * step_duration;

                    // Stop if we've exceeded the duration
                    if offset >= total_beats as f64 {
                        break;
                    }

                    // Pick a note from the scale with some variation
                    let scale_idx = (rng.next() % scale.len() as u64) as usize;
                    let base_pitch = scale[scale_idx] as i16 + *octave_offset as i16;
                    let pitch = base_pitch.clamp(21, 108) as u8;

                    // Add velocity variation
                    let velocity_variation = (rng.next() % 20) as i16 - 10;
                    let velocity = ((*velocity_base as i16) + velocity_variation).clamp(40, 127) as u8;

                    notes.push(PluginNote {
                        pitch,
                        velocity,
                        _pad: [0; 6],
                        duration: step_duration,
                        offset,
                    });
                }
            }
        }

        if notes.is_empty() {
            continue;
        }

        // Store notes and create sequence
        let notes_ptr = notes.as_ptr();
        let note_count = notes.len() as u32;

        sequences.push(PluginSequence {
            notes: notes_ptr,
            note_count,
            instrument: *instrument,
            channel: layer_idx as u8,
            _pad: [0; 2],
            tempo: config.tempo,
            _reserved: [0; 2],
        });

        all_notes.push(notes);
    }

    if sequences.is_empty() {
        return Err("No notes generated".to_string());
    }

    // Update sequence pointers to point to the actual stored notes
    for (i, seq) in sequences.iter_mut().enumerate() {
        seq.notes = all_notes[i].as_ptr();
    }

    Ok((sequences, all_notes))
}

/// Get scale notes for a given root and mode.
fn get_scale(root: u8, is_minor: bool) -> Vec<u8> {
    let intervals = if is_minor {
        // Natural minor scale intervals
        [0, 2, 3, 5, 7, 8, 10]
    } else {
        // Major scale intervals
        [0, 2, 4, 5, 7, 9, 11]
    };

    intervals.iter().map(|i| root + i).collect()
}

/// Create an error result.
fn create_error_result(msg: &str) -> *mut PluginResult {
    let error_msg = CString::new(msg).unwrap_or_else(|_| CString::new("Unknown error").unwrap());

    let result = Box::new(PluginResult {
        error: PluginError::InternalError,
        _pad: [0; 4],
        sequences: ptr::null(),
        sequence_count: 0,
        error_message: error_msg.into_raw(),
        _reserved: [ptr::null(); 2],
    });

    Box::into_raw(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bjorklund_tresillo() {
        // E(5,8) - Cuban tresillo
        let pattern = bjorklund(5, 8);
        assert_eq!(pattern.len(), 8);
        assert_eq!(pattern.iter().filter(|&&x| x).count(), 5);
    }

    #[test]
    fn test_bjorklund_cinquillo() {
        // E(3,8)
        let pattern = bjorklund(3, 8);
        assert_eq!(pattern.len(), 8);
        assert_eq!(pattern.iter().filter(|&&x| x).count(), 3);
    }

    #[test]
    fn test_bjorklund_edge_cases() {
        // All pulses
        let pattern = bjorklund(8, 8);
        assert!(pattern.iter().all(|&x| x));

        // No pulses
        let pattern = bjorklund(0, 8);
        assert!(pattern.iter().all(|&x| !x));
    }

    #[test]
    fn test_get_scale() {
        let c_major = get_scale(60, false);
        assert_eq!(c_major, vec![60, 62, 64, 65, 67, 69, 71]);

        let a_minor = get_scale(57, true);
        assert_eq!(a_minor, vec![57, 59, 60, 62, 64, 65, 67]);
    }

    #[test]
    fn test_simple_rng() {
        let mut rng1 = SimpleRng::new(42);
        let mut rng2 = SimpleRng::new(42);

        // Same seed should produce same sequence
        for _ in 0..10 {
            assert_eq!(rng1.next(), rng2.next());
        }
    }
}
