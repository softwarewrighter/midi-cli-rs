//! Plugin discovery - extends TOML parsing to detect native libraries.
//!
//! Looks for `[pack.native]` sections in mood pack TOML files to find
//! native plugins alongside the standard TOML-only configuration.

use std::path::{Path, PathBuf};

/// Information about a discovered native plugin.
#[derive(Debug, Clone)]
pub struct NativePluginInfo {
    /// Name of the mood pack
    pub pack_name: String,
    /// Path to the TOML configuration file
    pub config_path: PathBuf,
    /// Path to the native library
    pub library_path: PathBuf,
    /// Version from pack metadata
    pub version: String,
    /// Author from pack metadata
    pub author: Option<String>,
    /// Description from pack metadata
    pub description: Option<String>,
    /// Mood names defined in this plugin
    pub mood_names: Vec<String>,
    /// Whether hot reload is enabled
    pub hot_reload: bool,
}

/// Parse a TOML file to check for native plugin configuration.
///
/// Returns `Some(NativePluginInfo)` if the TOML has a `[pack.native]` section
/// with a `library` field.
pub fn parse_native_plugin(toml_path: &Path) -> Option<NativePluginInfo> {
    let content = std::fs::read_to_string(toml_path).ok()?;
    let pack: toml::Value = toml::from_str(&content).ok()?;

    // Get pack metadata
    let pack_meta = pack.get("pack")?;
    let pack_name = pack_meta.get("name")?.as_str()?.to_string();
    let version = pack_meta
        .get("version")
        .and_then(|v| v.as_str())
        .unwrap_or("1.0.0")
        .to_string();
    let author = pack_meta
        .get("author")
        .and_then(|a| a.as_str())
        .map(|s| s.to_string());
    let description = pack_meta
        .get("description")
        .and_then(|d| d.as_str())
        .map(|s| s.to_string());

    // Check for native section
    let native = pack_meta.get("native")?;
    let library_name = native.get("library")?.as_str()?;
    let hot_reload = native
        .get("hot_reload")
        .and_then(|h| h.as_bool())
        .unwrap_or(false);

    // Resolve library path relative to TOML file
    let library_path = resolve_library_relative_to(library_name, toml_path);

    // Get mood names
    let mood_names = pack
        .get("moods")
        .and_then(|m| m.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|m| m.get("name").and_then(|n| n.as_str()))
                .map(|s| s.to_string())
                .collect()
        })
        .unwrap_or_default();

    Some(NativePluginInfo {
        pack_name,
        config_path: toml_path.to_path_buf(),
        library_path,
        version,
        author,
        description,
        mood_names,
        hot_reload,
    })
}

/// Discover all native plugins in a directory.
pub fn discover_native_plugins(moods_dir: &Path) -> Vec<NativePluginInfo> {
    let mut plugins = Vec::new();

    if !moods_dir.exists() {
        return plugins;
    }

    if let Ok(entries) = std::fs::read_dir(moods_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "toml").unwrap_or(false) {
                if let Some(info) = parse_native_plugin(&path) {
                    plugins.push(info);
                }
            }
        }
    }

    plugins
}

/// Check if a mood name belongs to a native plugin.
pub fn is_native_mood(mood_name: &str, moods_dir: &Path) -> Option<NativePluginInfo> {
    let mood_lower = mood_name.to_lowercase();

    for plugin in discover_native_plugins(moods_dir) {
        for name in &plugin.mood_names {
            if name.to_lowercase() == mood_lower {
                return Some(plugin);
            }
        }
    }

    None
}

/// Resolve library path relative to TOML file location.
fn resolve_library_relative_to(library_name: &str, toml_path: &Path) -> PathBuf {
    // If it's an absolute path, use it directly
    let path = Path::new(library_name);
    if path.is_absolute() {
        return add_platform_extension(path);
    }

    // Resolve relative to the TOML file's directory
    let parent = toml_path.parent().unwrap_or(Path::new("."));
    add_platform_extension(&parent.join(library_name))
}

/// Add platform-specific library extension if not present.
fn add_platform_extension(path: &Path) -> PathBuf {
    // If already has an extension, use as-is
    if path.extension().is_some() {
        return path.to_path_buf();
    }

    #[cfg(target_os = "macos")]
    let ext = "dylib";
    #[cfg(target_os = "linux")]
    let ext = "so";
    #[cfg(target_os = "windows")]
    let ext = "dll";
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    let ext = "so";

    let filename = path
        .file_name()
        .and_then(|f| f.to_str())
        .unwrap_or("plugin");

    let mut result = path.to_path_buf();

    // Add lib prefix on Unix-like systems
    #[cfg(not(target_os = "windows"))]
    {
        if !filename.starts_with("lib") {
            result.set_file_name(format!("lib{}.{}", filename, ext));
            return result;
        }
    }

    result.set_extension(ext);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_platform_extension() {
        let path = PathBuf::from("/usr/lib/myplugin");
        let resolved = add_platform_extension(&path);
        #[cfg(target_os = "macos")]
        assert!(resolved.to_str().unwrap().ends_with("libmyplugin.dylib"));
        #[cfg(target_os = "linux")]
        assert!(resolved.to_str().unwrap().ends_with("libmyplugin.so"));
    }

    #[test]
    fn test_add_platform_extension_with_lib_prefix() {
        let path = PathBuf::from("/usr/lib/libmyplugin");
        let resolved = add_platform_extension(&path);
        #[cfg(target_os = "macos")]
        assert!(resolved.to_str().unwrap().ends_with("libmyplugin.dylib"));
    }

    #[test]
    fn test_add_platform_extension_already_has_extension() {
        let path = PathBuf::from("/usr/lib/myplugin.dylib");
        let resolved = add_platform_extension(&path);
        assert_eq!(resolved, path);
    }

    #[test]
    fn test_discover_empty_dir() {
        let temp_dir = std::env::temp_dir().join("test_discover_empty");
        let _ = std::fs::create_dir_all(&temp_dir);
        let plugins = discover_native_plugins(&temp_dir);
        assert!(plugins.is_empty());
        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
