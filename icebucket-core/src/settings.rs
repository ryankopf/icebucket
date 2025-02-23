use serde_json::json;
use std::fs;
use crate::Settings;
use crate::SyncSettings;

pub fn load_or_create_settings() -> Settings {
  let settings_path = "settings.json";
  if let Ok(settings_data) = fs::read_to_string(settings_path) {
      serde_json::from_str(&settings_data).unwrap_or_else(|_| create_default_settings(settings_path))
  } else {
      create_default_settings(settings_path)
  }
}

pub fn create_default_settings(settings_path: &str) -> Settings {
  let default_settings = Settings {
      directories_to_scan: vec!["./".to_string()],
      seconds_between_scans: 60,
  };
  let settings_json = json!(default_settings);
  fs::write(settings_path, settings_json.to_string()).expect("Failed to write default settings");
  default_settings
}

pub fn create_default_sync_settings(sync_settings_path: &str) -> SyncSettings {
  let default_sync_settings = SyncSettings {
      service: "s3".to_string(),
      access_key: "YOUR_ACCESS_KEY".to_string(),
      secret_key: "YOUR_SECRET_KEY".to_string(),
      region: "us-east-1".to_string(),
      bucket: "your-bucket-name".to_string(),
      endpoint: "".to_string(),
      sync_type: "upload-only".to_string(),
      conflicts: "keep-local".to_string(),
      public: false,
  };
  let sync_settings_json = json!(default_sync_settings);
  fs::write(sync_settings_path, sync_settings_json.to_string()).expect("Failed to write sync settings");
  default_sync_settings
}