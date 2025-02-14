use serde_json::json;
use std::fs;
use crate::SyncSettings;

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
  };
  let sync_settings_json = json!(default_sync_settings);
  fs::write(sync_settings_path, sync_settings_json.to_string()).expect("Failed to write sync settings");
  default_sync_settings
}
