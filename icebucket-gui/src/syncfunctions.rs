use std::fs;
use std::path::Path;
use crate::SETTINGS_FILE;
use crate::SettingsData;
use crate::SyncSettings;

/// Save settings immediately when a change is made
pub fn save_settings(settings: &SettingsData) {
  if let Ok(data) = serde_json::to_string_pretty(settings) {
      let _ = fs::write(SETTINGS_FILE, data);
  }
}

pub fn load_sync_settings(directory: &str) -> SyncSettings {
  let path = Path::new(directory).join("sync.json");
  if path.exists() {
      if let Ok(data) = fs::read_to_string(path) {
          if let Ok(settings) = serde_json::from_str(&data) {
              return settings;
          }
      }
  }
  SyncSettings::default()
}

pub fn save_sync_settings(directory: &str, settings: &SyncSettings) {
  let path = Path::new(directory).join("sync.json");
  if let Ok(data) = serde_json::to_string_pretty(settings) {
      let _ = fs::write(path, data);
  }
}