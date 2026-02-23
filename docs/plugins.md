# Plugin System Documentation

This document provides comprehensive documentation for the midi-cli-rs plugin system, which allows users to create custom mood presets without writing Rust code.

## Table of Contents

- [Overview](#overview)
- [Architecture](#architecture)
- [Plugin File Format](#plugin-file-format)
- [Creating a Plugin](#creating-a-plugin)
- [Plugin Resolution](#plugin-resolution)
- [API Reference](#api-reference)
- [Web UI Integration](#web-ui-integration)
- [Best Practices](#best-practices)
- [Troubleshooting](#troubleshooting)

## Overview

The plugin system enables custom mood presets through TOML configuration files. Each plugin defines a "mood pack" containing one or more moods that delegate audio generation to built-in mood generators.

**Key Concepts:**
- **Mood Pack**: A TOML file containing one or more custom moods
- **Base Mood**: The built-in mood that handles actual audio generation
- **Overrides**: Custom defaults for tempo, key, and intensity

```
┌─────────────────────────────────────────────────────────┐
│                    Plugin System                         │
├─────────────────────────────────────────────────────────┤
│                                                          │
│  ~/.midi-cli-rs/moods/                                  │
│  ├── electronic.toml  ──┬── synthwave (base: upbeat)    │
│  │                      ├── techno (base: upbeat)       │
│  │                      ├── chillout (base: ambient)    │
│  │                      └── 8bit (base: chiptune)       │
│  │                                                       │
│  └── my-pack.toml     ──┬── custom1 (base: jazz)        │
│                         └── custom2 (base: suspense)    │
│                                                          │
└─────────────────────────────────────────────────────────┘
```

## Architecture

### System Flow

```
User Request                Plugin Resolution              Generation
─────────────              ─────────────────              ──────────

midi-cli-rs
preset -m synthwave   ──►  1. Check built-in moods
                           2. Search plugin files     ──►  3. Get base_mood
                           4. Load plugin overrides   ──►  5. Generate with
                                                              built-in generator
```

### Component Diagram

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│   CLI / API     │────►│  Plugin Loader  │────►│  Mood Generator │
└─────────────────┘     └─────────────────┘     └─────────────────┘
                               │                        │
                               ▼                        ▼
                        ┌─────────────────┐     ┌─────────────────┐
                        │  TOML Parser    │     │  NoteSequence   │
                        └─────────────────┘     └─────────────────┘
                               │                        │
                               ▼                        ▼
                        ┌─────────────────┐     ┌─────────────────┐
                        │ PluginMoodInfo  │     │    MIDI File    │
                        └─────────────────┘     └─────────────────┘
```

### Data Structures

```rust
/// Information about a mood pack (plugin file)
pub struct MoodPackInfo {
    pub name: String,              // Pack identifier
    pub version: String,           // Semantic version
    pub author: Option<String>,    // Pack author
    pub description: Option<String>,
    pub mood_count: usize,         // Number of moods
    pub moods: Vec<PluginMoodInfo>,
    pub file_path: Option<String>, // Source file path
}

/// Information about a single mood in a plugin
pub struct PluginMoodInfo {
    pub name: String,                    // Mood name users type
    pub base_mood: Option<String>,       // Built-in mood to delegate to
    pub default_key: String,             // Override key
    pub default_tempo: u16,              // Override tempo
    pub default_intensity: Option<u8>,   // Override intensity
    pub description: Option<String>,     // Human-readable description
}
```

## Plugin File Format

Plugins are TOML files with two sections: pack metadata and mood definitions.

### Complete Reference

```toml
# ============================================================
# PACK METADATA (required)
# ============================================================
[pack]
name = "electronic"              # Required: unique identifier
version = "1.0.0"                # Optional: semantic version
author = "Your Name"             # Optional: author name
description = "Electronic moods" # Optional: pack description

# ============================================================
# MOOD DEFINITIONS (at least one required)
# ============================================================
[[moods]]
name = "synthwave"               # Required: mood name (what users type)
base_mood = "upbeat"             # Required: built-in mood to use
default_key = "Am"               # Optional: default key (default: from base)
default_tempo = 118              # Optional: default tempo in BPM
default_intensity = 65           # Optional: default intensity (0-100)
description = "80s synth vibes"  # Optional: description
tags = ["electronic", "retro"]   # Optional: tags for discovery

[[moods]]
name = "techno"
base_mood = "upbeat"
default_key = "Dm"
default_tempo = 130
default_intensity = 85
description = "Driving techno beats"
tags = ["electronic", "dance"]
```

### Field Reference

#### Pack Section

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `name` | String | Yes | - | Unique pack identifier (used for filename) |
| `version` | String | No | "1.0" | Semantic version number |
| `author` | String | No | None | Pack author name |
| `description` | String | No | None | Human-readable description |

#### Mood Section

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `name` | String | Yes | - | Mood name (case-insensitive) |
| `base_mood` | String | Yes | - | Built-in mood for generation |
| `default_key` | String | No | Base mood's key | Musical key (e.g., "Am", "C") |
| `default_tempo` | Integer | No | Base mood's tempo | Tempo in BPM (20-300) |
| `default_intensity` | Integer | No | 50 | Intensity level (0-100) |
| `description` | String | No | None | Human-readable description |
| `tags` | Array | No | [] | Tags for categorization |

### Available Base Moods

| Base Mood | Character | Default Key | Best For |
|-----------|-----------|-------------|----------|
| `suspense` | Tense, dramatic | Am | Thriller, tension |
| `eerie` | Spooky, unsettling | Dm | Horror, mystery |
| `upbeat` | Energetic, happy | C | Pop, dance, electronic |
| `calm` | Peaceful, gentle | G | Relaxation, meditation |
| `ambient` | Atmospheric | Em | Background, atmosphere |
| `jazz` | Swinging, sophisticated | F | Lounge, cocktail |
| `chiptune` | 8-bit, retro | C | Retro, game music |
| `orchestral` | Cinematic, grand | C | Film scores, trailers |
| `show` | Broadway, theatrical | Bb | Musical theater |

### Supported Keys

```
Major Keys: C, D, Eb, E, F, G, A, Bb, B
Minor Keys: Cm, Dm, Ebm, Em, Fm, Gm, Am, Bbm, Bm
```

## Creating a Plugin

### Step 1: Create the Moods Directory

```bash
mkdir -p ~/.midi-cli-rs/moods
```

### Step 2: Create Your Plugin File

```bash
cat > ~/.midi-cli-rs/moods/my-moods.toml << 'EOF'
[pack]
name = "my-moods"
version = "1.0.0"
author = "Your Name"
description = "My custom mood collection"

[[moods]]
name = "workout"
base_mood = "upbeat"
default_tempo = 145
default_intensity = 90
description = "High-energy workout music"

[[moods]]
name = "focus"
base_mood = "ambient"
default_key = "C"
default_tempo = 70
default_intensity = 35
description = "Background music for concentration"

[[moods]]
name = "dramatic"
base_mood = "orchestral"
default_key = "Dm"
default_tempo = 80
default_intensity = 85
description = "Epic cinematic moments"
EOF
```

### Step 3: Verify Installation

```bash
# List all moods - your new ones should appear
midi-cli-rs moods

# Generate with your custom mood
midi-cli-rs preset -m workout -d 5 -o workout.wav
midi-cli-rs preset -m focus -d 10 -o focus.wav
midi-cli-rs preset -m dramatic -d 5 -o dramatic.wav
```

### Example: Complete Electronic Pack

```toml
# ~/.midi-cli-rs/moods/electronic.toml
# Electronic music mood pack

[pack]
name = "electronic"
version = "1.1.0"
author = "midi-cli-rs"
description = "Electronic and synth-based moods"

[[moods]]
name = "synthwave"
base_mood = "upbeat"
default_key = "Am"
default_tempo = 118
default_intensity = 65
description = "80s inspired synthwave with arpeggios and bass"
tags = ["electronic", "synthwave", "retro", "80s"]

[[moods]]
name = "chillout"
base_mood = "ambient"
default_key = "Em"
default_tempo = 85
default_intensity = 40
description = "Relaxed downtempo electronic"
tags = ["electronic", "chill", "downtempo"]

[[moods]]
name = "techno"
base_mood = "upbeat"
default_key = "Dm"
default_tempo = 130
default_intensity = 85
description = "Driving techno with pumping bass"
tags = ["electronic", "techno", "dance", "club"]

[[moods]]
name = "8bit"
base_mood = "chiptune"
default_key = "C"
default_tempo = 140
default_intensity = 70
description = "Retro 8-bit video game style"
tags = ["electronic", "8bit", "chiptune", "retro", "game"]
```

## Plugin Resolution

When a user requests a mood, the system follows this resolution order:

```
1. Check built-in moods (suspense, jazz, etc.)
   └── If found: Use directly

2. Search plugin files in ~/.midi-cli-rs/moods/
   └── For each .toml file:
       └── Parse and search [[moods]] array
           └── If mood.name matches:
               └── Return PluginMoodInfo

3. If not found: Return error "Unknown mood"
```

### Resolution Code Path

```rust
// Simplified resolution logic
fn resolve_mood(name: &str) -> Result<(Mood, Option<PluginMoodInfo>), Error> {
    // 1. Try built-in moods first
    if let Some(mood) = Mood::parse(name) {
        return Ok((mood, None));
    }

    // 2. Search plugin moods
    if let Some(plugin_mood) = lookup_plugin_mood(name) {
        if let Some(base) = &plugin_mood.base_mood {
            if let Some(base_enum) = Mood::parse(base) {
                return Ok((base_enum, Some(plugin_mood)));
            }
        }
    }

    Err(Error::UnknownMood(name.to_string()))
}
```

### Override Application

When a plugin mood is resolved, its overrides are applied to the generation config:

```rust
let mut config = PresetConfig {
    duration_secs: args.duration,
    key: plugin_mood.default_key.parse().unwrap_or(base_mood.default_key()),
    tempo: plugin_mood.default_tempo,
    intensity: plugin_mood.default_intensity.unwrap_or(50),
    seed: args.seed,
};

// Generate using the base mood's generator
let sequences = generate_mood(base_mood, &config);
```

## API Reference

### Public Functions

```rust
/// Get the moods plugin directory path
pub fn get_moods_dir() -> PathBuf;
// Returns: ~/.midi-cli-rs/moods/

/// Look up a plugin mood by name
pub fn lookup_plugin_mood(mood_name: &str) -> Option<PluginMoodInfo>;

/// Parse a mood pack TOML file
pub fn parse_mood_pack_info(path: &Path) -> Option<MoodPackInfo>;
```

### REST API Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/plugins` | GET | List all installed plugins |
| `/api/plugins` | POST | Upload a new plugin |
| `/api/plugins/:name` | DELETE | Remove a plugin |
| `/api/moods` | GET | List all moods (built-in + plugins) |

### Example API Usage

```bash
# List plugins
curl http://localhost:3105/api/plugins

# Upload a plugin
curl -X POST http://localhost:3105/api/plugins \
  -H "Content-Type: application/json" \
  -d '{"content": "[pack]\nname = \"test\"\n\n[[moods]]\nname = \"test-mood\"\nbase_mood = \"jazz\""}'

# Delete a plugin
curl -X DELETE http://localhost:3105/api/plugins/test
```

## Web UI Integration

The Web UI provides a Plugins tab for managing mood packs:

1. **View Plugins**: See all installed packs and their moods
2. **Plugin Details**: Click to see moods within each pack
3. **Upload Plugin**: Paste TOML content to install new packs
4. **Delete Plugin**: Remove unwanted packs
5. **Use Plugin Moods**: Plugin moods appear in the Presets dropdown

## Best Practices

### Naming Conventions

- **Pack names**: lowercase, hyphenated (e.g., `my-moods`, `electronic`)
- **Mood names**: lowercase, descriptive (e.g., `synthwave`, `workout`)
- **Avoid conflicts**: Don't name moods the same as built-ins

### Version Management

```toml
[pack]
name = "my-pack"
version = "1.0.0"  # Major.Minor.Patch
```

- **Major**: Breaking changes to mood behavior
- **Minor**: New moods added
- **Patch**: Bug fixes, description changes

### Documentation

Always include descriptions:

```toml
[[moods]]
name = "focus"
base_mood = "ambient"
description = "Low-key background for deep work. Based on ambient with slower tempo."
```

### Tempo Guidelines

| Use Case | Recommended BPM |
|----------|-----------------|
| Meditation | 40-60 |
| Background/Focus | 60-80 |
| Chill/Lounge | 80-100 |
| Pop/Dance | 100-130 |
| High Energy | 130-160 |
| Extreme | 160-200 |

### Intensity Guidelines

| Level | Description |
|-------|-------------|
| 0-30 | Minimal, sparse, quiet |
| 30-50 | Moderate, balanced |
| 50-70 | Active, full |
| 70-90 | Intense, powerful |
| 90-100 | Maximum, overwhelming |

## Troubleshooting

### Plugin Not Appearing

1. **Check file location**: Must be in `~/.midi-cli-rs/moods/`
2. **Check file extension**: Must be `.toml`
3. **Validate TOML syntax**: Use `toml-cli` or online validator
4. **Check required fields**: `[pack].name` and `[[moods]].name` + `base_mood`

```bash
# Verify file exists
ls -la ~/.midi-cli-rs/moods/

# Validate TOML (if toml-cli installed)
toml fmt ~/.midi-cli-rs/moods/my-pack.toml
```

### "Unknown mood" Error

- Verify `base_mood` is a valid built-in mood name
- Check spelling (names are case-insensitive)
- Ensure the mood is in a `[[moods]]` array, not a `[mood]` table

### Mood Sounds Wrong

- **Too fast/slow**: Adjust `default_tempo`
- **Wrong feel**: Try a different `base_mood`
- **Too loud/quiet**: Adjust `default_intensity`
- **Wrong tonality**: Change `default_key`

### TOML Syntax Errors

Common mistakes:
```toml
# Wrong: missing quotes
name = my-pack

# Correct: strings need quotes
name = "my-pack"

# Wrong: wrong array syntax
[moods]
name = "test"

# Correct: double brackets for array items
[[moods]]
name = "test"
```

## Native Plugins

For advanced users who need custom generation algorithms (not just parameter overrides), midi-cli-rs supports **native plugins** - shared libraries that implement custom mood generators.

Native plugins can:
- Implement entirely new algorithmic approaches (e.g., Euclidean rhythms, L-systems, Markov chains)
- Use external data or APIs
- Achieve higher performance for complex algorithms

See [Native Plugin Development Guide](native-plugins.md) for details.

### Quick Example

```toml
# ~/.midi-cli-rs/moods/euclidean.toml
[pack]
name = "euclidean"
version = "1.0.0"

[pack.native]
library = "libeuclidean_mood_plugin"

[[moods]]
name = "euclidean"
description = "Polyrhythmic patterns using Bjorklund's algorithm"
default_key = "Am"
default_tempo = 100
```

## See Also

- [Native Plugin Development Guide](native-plugins.md)
- [Architecture Documentation](architecture.md)
- [Design Documentation](design.md)
- [Usage Guide](usage.md)
- [Wiki: Plugins and Extensibility](https://github.com/softwarewrighter/midi-cli-rs/wiki/Plugins-and-Extensibility)
