# Mood Plugins

This directory contains sample TOML mood pack files that can be loaded into midi-cli-rs.

## Installation

Copy any `.toml` file to `~/.midi-cli-rs/moods/` to install it:

```bash
cp plugins/cinematic.toml ~/.midi-cli-rs/moods/
```

Or upload via the web UI (Plugins tab) when running `midi-cli-rs serve`.

## Creating Your Own

See `template.toml` for a complete annotated example you can customize.

## Current Plugin Architecture

The current plugin system uses TOML configuration that maps to built-in trait implementations:

### Structure

```toml
[meta]
name = "my-pack"
version = "1.0.0"

[[moods]]
name = "my-mood"
default_key = "Am"
default_tempo = 90

[moods.layers.foundation]
instruments = ["strings", "pad"]
# ... layer configuration
```

### Supported Layer Types

- `foundation` - Base layer (strings, pads, drones)
- `melody` - Lead melodic lines
- `accent` - Decorative elements (brass hits, bells)
- `rhythm` - Rhythmic elements
- `bass` - Bass lines

### Future Trait-Based Extension

The architecture is designed to support custom trait implementations:

```rust
// Future: Custom generator traits
pub trait MelodicGenerator {
    fn generate(&self, config: &LayerConfig, variation: &PresetVariation) -> Vec<Note>;
}

pub trait RhythmGenerator {
    fn generate(&self, config: &LayerConfig, variation: &PresetVariation) -> Vec<Note>;
}
```

TOML files will specify which trait implementation to use:

```toml
[moods.layers.melody]
generator = "contour-based"  # Maps to ContourMelodicGenerator
contour_style = "arch"
```

See `docs/mood-config-design.md` for the complete architecture vision.
