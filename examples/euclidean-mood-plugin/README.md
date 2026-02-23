# Euclidean Mood Plugin

A native plugin for midi-cli-rs that generates polyrhythmic patterns using Bjorklund's algorithm.

## What are Euclidean Rhythms?

Euclidean rhythms distribute k pulses across n steps as evenly as possible. Many traditional world music rhythms are Euclidean:

- **E(5,8)** = [x . x . x . x x] - Cuban tresillo
- **E(3,8)** = [x . . x . . x .] - Cuban cinquillo
- **E(7,16)** - Brazilian samba pattern

This plugin creates layered Euclidean patterns that interlock to form complex polyrhythmic textures.

## Building

```bash
cd examples/euclidean-mood-plugin
cargo build --release
```

This produces:
- macOS: `target/release/libeuclidean_mood_plugin.dylib`
- Linux: `target/release/libeuclidean_mood_plugin.so`
- Windows: `target/release/euclidean_mood_plugin.dll`

## Installation

1. Build the plugin:
   ```bash
   cargo build --release
   ```

2. Create the moods directory if it doesn't exist:
   ```bash
   mkdir -p ~/.midi-cli-rs/moods
   ```

3. Copy the TOML config and library:
   ```bash
   cp euclidean.toml ~/.midi-cli-rs/moods/
   cp target/release/libeuclidean_mood_plugin.dylib ~/.midi-cli-rs/moods/
   ```

## Usage

```bash
# List moods - should show "euclidean" under Native Plugins
midi-cli-rs moods

# Generate Euclidean rhythm
midi-cli-rs preset -m euclidean -d 10 --seed 42 -o euclidean.wav

# Higher intensity = more layers
midi-cli-rs preset -m euclidean -d 10 --intensity 80 --seed 42 -o complex.wav

# Different seeds produce different pattern combinations
midi-cli-rs preset -m euclidean -d 10 --seed 1 -o pattern1.wav
midi-cli-rs preset -m euclidean -d 10 --seed 2 -o pattern2.wav
```

## How It Works

The plugin generates 1-3 layers based on intensity:

1. **Bass layer** (intensity >= 0): Sparse Euclidean pattern (3-5 pulses over 8-16 steps) on electric bass
2. **Melody layer** (intensity >= 30): Moderate density (5-8 pulses over 8-12 steps) on piano
3. **High layer** (intensity >= 70): Dense pattern (7-11 pulses over 12-16 steps) on vibraphone

Each layer uses independently seeded random variations for:
- Euclidean pattern parameters (pulses, steps)
- Note selection from the scale
- Velocity variation

## Plugin API

This plugin demonstrates the midi-cli-rs native plugin API. Key exports:

```c
// Return plugin metadata
PluginInfo plugin_info(void);

// Generate music sequences
PluginResult* plugin_generate(const PluginConfig* config);

// Free result allocated by plugin_generate
void plugin_free_result(PluginResult* result);

// Optional: validate configuration
PluginError plugin_validate_config(const PluginConfig* config);
```

See `src/lib.rs` for the complete implementation.

## License

MIT License - same as midi-cli-rs
