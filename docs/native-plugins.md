# Native Plugin Development Guide

This guide explains how to create native plugins for midi-cli-rs using Rust (or any language with C ABI support).

## Overview

Native plugins are shared libraries (.dylib on macOS, .so on Linux, .dll on Windows) that implement custom mood generators. Unlike TOML-based plugins that configure existing moods, native plugins can implement entirely new generation algorithms.

## Plugin Architecture

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   TOML Config   │───▶│  Plugin Loader   │───▶│ Loaded Plugin   │
│  (library path) │    │  (libloading)    │    │ (function ptrs) │
└─────────────────┘    └──────────────────┘    └─────────────────┘
                                                        │
                              ┌─────────────────────────┘
                              ▼
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│  PresetConfig   │───▶│  PluginConfig    │───▶│ Plugin Generate │
│  (Rust types)   │    │  (C ABI / FFI)   │    │ (extern "C" fn) │
└─────────────────┘    └──────────────────┘    └─────────────────┘
                                                        │
                              ┌─────────────────────────┘
                              ▼
┌─────────────────┐    ┌──────────────────┐
│ Vec<NoteSeq>    │◀───│  PluginResult    │
│ (Rust types)    │    │  (C ABI / FFI)   │
└─────────────────┘    └──────────────────┘
```

## Required Exports

Every plugin must export these symbols:

```c
// Return plugin metadata (required)
PluginInfo plugin_info(void);

// Generate music sequences (required)
PluginResult* plugin_generate(const PluginConfig* config);

// Free result allocated by plugin_generate (required)
void plugin_free_result(PluginResult* result);

// Validate configuration (optional)
PluginError plugin_validate_config(const PluginConfig* config);
```

## FFI Types Reference

All types use `#[repr(C)]` for ABI stability.

### PluginError

```rust
#[repr(C)]
pub enum PluginError {
    Ok = 0,
    InvalidConfig = 1,
    AllocationFailed = 2,
    InternalError = 3,
    VersionMismatch = 4,
    MissingFunction = 5,
}
```

### PluginInfo

```rust
#[repr(C)]
pub struct PluginInfo {
    pub api_version: u32,          // Must equal PLUGIN_API_VERSION (currently 1)
    pub name: *const c_char,       // Plugin name (null-terminated UTF-8)
    pub version: *const c_char,    // Plugin version (null-terminated UTF-8)
    pub author: *const c_char,     // Author (may be null)
    pub description: *const c_char, // Description (may be null)
    pub _reserved: [*const c_void; 4],
}
```

### PluginConfig

```rust
#[repr(C)]
pub struct PluginConfig {
    pub duration_secs: f64,  // Duration in seconds
    pub key_root: u8,        // MIDI pitch of key root (60 = C4)
    pub key_mode: u8,        // 0 = major, 1 = minor
    pub intensity: u8,       // 0-100
    pub seed: u64,           // Random seed for reproducibility
    pub tempo: u16,          // BPM
    pub _pad: [u8; 4],
    pub _reserved: [u64; 4],
}
```

### PluginNote

```rust
#[repr(C)]
pub struct PluginNote {
    pub pitch: u8,       // MIDI pitch (0-127)
    pub velocity: u8,    // MIDI velocity (0-127)
    pub _pad: [u8; 6],
    pub duration: f64,   // Duration in beats
    pub offset: f64,     // Start time in beats
}
```

### PluginSequence

```rust
#[repr(C)]
pub struct PluginSequence {
    pub notes: *const PluginNote,  // Pointer to note array
    pub note_count: u32,           // Number of notes
    pub instrument: u8,            // GM instrument (0-127)
    pub channel: u8,               // MIDI channel (0-15)
    pub _pad: [u8; 2],
    pub tempo: u16,                // BPM (or 0 to use config tempo)
    pub _reserved: [u64; 2],
}
```

### PluginResult

```rust
#[repr(C)]
pub struct PluginResult {
    pub error: PluginError,
    pub _pad: [u8; 4],
    pub sequences: *const PluginSequence,  // Pointer to sequence array
    pub sequence_count: u32,               // Number of sequences
    pub error_message: *const c_char,      // Error message if error != Ok
    pub _reserved: [*const c_void; 2],
}
```

## Memory Management

**The memory ownership contract:**

1. **Plugin allocates**: `plugin_generate()` allocates all memory for the result, sequences, and notes
2. **Host uses**: The host reads the result without modifying it
3. **Host frees**: The host calls `plugin_free_result()` when done
4. **Plugin deallocates**: `plugin_free_result()` frees all memory

Example allocation pattern in Rust:

```rust
#[no_mangle]
pub extern "C" fn plugin_generate(config: *const PluginConfig) -> *mut PluginResult {
    // Generate notes (owned by the plugin)
    let notes: Vec<PluginNote> = generate_notes();

    // Create sequence pointing to notes
    let sequence = PluginSequence {
        notes: notes.as_ptr(),
        note_count: notes.len() as u32,
        // ...
    };

    // Leak the notes so they stay alive
    std::mem::forget(notes);

    // Create and return result
    let result = Box::new(PluginResult {
        sequences: Box::into_raw(Box::new(vec![sequence])) as *const _,
        // ...
    });

    Box::into_raw(result)
}

#[no_mangle]
pub extern "C" fn plugin_free_result(result: *mut PluginResult) {
    if result.is_null() { return; }

    unsafe {
        let result = Box::from_raw(result);

        // Free sequences and notes
        if !result.sequences.is_null() {
            let sequences = Vec::from_raw_parts(
                result.sequences as *mut PluginSequence,
                result.sequence_count as usize,
                result.sequence_count as usize,
            );

            for seq in sequences {
                Vec::from_raw_parts(
                    seq.notes as *mut PluginNote,
                    seq.note_count as usize,
                    seq.note_count as usize,
                );
            }
        }
    }
}
```

## Creating a Plugin in Rust

### 1. Create the Cargo project

```bash
cargo new --lib my-mood-plugin
cd my-mood-plugin
```

### 2. Configure Cargo.toml

```toml
[package]
name = "my-mood-plugin"
version = "1.0.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]
```

### 3. Implement the plugin

```rust
use std::ffi::{c_char, c_void};
use std::ptr;

// Copy the FFI type definitions from midi-cli-rs/src/plugin/ffi_types.rs
// or reference the types in the examples/euclidean-mood-plugin

const PLUGIN_API_VERSION: u32 = 1;

static PLUGIN_NAME: &[u8] = b"my-mood\0";
static PLUGIN_VERSION: &[u8] = b"1.0.0\0";

#[no_mangle]
pub extern "C" fn plugin_info() -> PluginInfo {
    PluginInfo {
        api_version: PLUGIN_API_VERSION,
        name: PLUGIN_NAME.as_ptr() as *const c_char,
        version: PLUGIN_VERSION.as_ptr() as *const c_char,
        author: ptr::null(),
        description: ptr::null(),
        _reserved: [ptr::null(); 4],
    }
}

#[no_mangle]
pub extern "C" fn plugin_generate(config: *const PluginConfig) -> *mut PluginResult {
    // Your generation logic here
}

#[no_mangle]
pub extern "C" fn plugin_free_result(result: *mut PluginResult) {
    // Free allocated memory
}
```

### 4. Build

```bash
cargo build --release
```

## TOML Configuration

Create a TOML file that references your plugin:

```toml
[pack]
name = "my-mood"
version = "1.0.0"
author = "Your Name"
description = "My custom mood generator"

[pack.native]
# Library name without platform extension
library = "libmy_mood_plugin"
# Set to true to reload on file change (future feature)
hot_reload = false

[[moods]]
name = "my-mood"
description = "My custom algorithmic mood"
default_key = "Am"
default_tempo = 120
tags = ["algorithmic", "custom"]
```

## Installation

1. Build your plugin: `cargo build --release`
2. Create the moods directory: `mkdir -p ~/.midi-cli-rs/moods`
3. Copy files:
   ```bash
   cp my-mood.toml ~/.midi-cli-rs/moods/
   cp target/release/libmy_mood_plugin.dylib ~/.midi-cli-rs/moods/
   ```

## Testing

```bash
# Verify plugin is detected
midi-cli-rs moods

# Generate audio
midi-cli-rs preset -m my-mood -d 10 --seed 42 -o output.wav
```

## Example: Euclidean Rhythm Plugin

See `examples/euclidean-mood-plugin/` for a complete working example that implements Bjorklund's algorithm for Euclidean rhythms.

## Troubleshooting

### "Library not found"

- Check the library path in your TOML matches the actual filename
- Ensure the library is in `~/.midi-cli-rs/moods/` alongside the TOML

### "Missing required symbol"

- Verify all required exports are present: `plugin_info`, `plugin_generate`, `plugin_free_result`
- Use `nm -gU libmyplugin.dylib` to list exported symbols

### "API version mismatch"

- Your plugin's `plugin_info()` must return `api_version = 1`
- Check you're using the latest FFI type definitions

### Segmentation fault

- Verify memory management: plugin allocates, host calls free
- Check all pointers before dereferencing
- Ensure notes/sequences remain valid until `plugin_free_result` is called

## Thread Safety

Plugins must be thread-safe. Multiple calls to `plugin_generate()` may happen concurrently. Avoid global mutable state, or protect it with synchronization primitives.

## Best Practices

1. **Use static strings** for plugin info to avoid allocation
2. **Validate inputs** in `plugin_validate_config()`
3. **Return descriptive errors** via `error_message`
4. **Test with various seeds** to ensure reproducibility
5. **Document your algorithm** in the TOML description
