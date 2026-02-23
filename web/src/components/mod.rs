//! UI components for the MIDI CLI web interface.

mod audio_player;
mod melody_editor;
mod melody_list;
mod plugin_manager;
mod preset_editor;
mod preset_list;

pub use audio_player::AudioPlayer;
pub use melody_editor::MelodyEditor;
pub use melody_list::MelodyList;
pub use plugin_manager::PluginManager;
pub use preset_editor::PresetEditor;
pub use preset_list::PresetList;
