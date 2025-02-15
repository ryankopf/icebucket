use std::collections::HashMap;
use std::time::SystemTime;
use std::fs;
use std::sync::atomic::Ordering;
use crate::services::s3::{service_s3_check, service_s3_upload, service_s3_multipart_upload};
use crate::{Log, SyncSettings, VERBOSE};
use crate::settings::create_default_sync_settings;
use aws_sdk_s3::{Client, config::Region};
use aws_config::BehaviorVersion;
use aws_sdk_s3::config::Credentials;
use aws_config::meta::region::RegionProviderChain;

pub async fn sync_directory(dir: &str, file_map: &mut HashMap<String, SystemTime>, log: &mut Log) {
  let sync_settings_path = format!("{}/sync.json", dir);
  let sync_settings: SyncSettings = match fs::read_to_string(&sync_settings_path) {
      Ok(settings_data) => match serde_json::from_str(&settings_data) {
          Ok(settings) => settings,
          Err(_) => create_default_sync_settings(&sync_settings_path),
      },
      Err(_) => create_default_sync_settings(&sync_settings_path),
  };

  let mut files_to_sync = Vec::new();
  let mut deletions = Vec::new();

  match fs::read_dir(dir) {
      Ok(entries) => {
          for entry in entries {
              if let Ok(entry) = entry {
                  let path = entry.path();
                  if path.is_file() {
                      let metadata = fs::metadata(&path).unwrap();
                      let modified = metadata.modified().unwrap();
                      let path_str = path.to_str().unwrap().to_string();

                      if let Some(last_modified) = file_map.get(&path_str) {
                          if &modified > last_modified {
                              files_to_sync.push(path_str.clone());
                          }
                      } else {
                          files_to_sync.push(path_str.clone());
                      }

                      file_map.insert(path_str, modified);
                  }
              }
          }
      }
      Err(e) => eprintln!("Failed to read directory {}: {}", dir, e),
  }

  // Check for deletions
  let current_files: Vec<String> = file_map.keys().cloned().collect();
  for file in current_files {
      if !fs::metadata(&file).is_ok() {
          deletions.push(file.clone());
          file_map.remove(&file);
      }
  }

  // Sync files to S3
  if sync_settings.service == "s3" {
      let region_provider = RegionProviderChain::default_provider().or_else(Region::new(sync_settings.region));
      let config = aws_config::defaults(BehaviorVersion::latest())
          .region(region_provider)
          .credentials_provider(Credentials::new(
              sync_settings.access_key,
              sync_settings.secret_key,
              None,
              None,
              "default",
          ))
          .load()
          .await;
      let client = Client::new(&config);

      for file in &files_to_sync {
          let relative_path = file.strip_prefix(dir).unwrap().replace("\\", "/");
          let s3_path = relative_path.trim_start_matches('/');
          if !service_s3_check(&client, &sync_settings.bucket, s3_path).await {
              if VERBOSE.load(Ordering::Relaxed) {
                  println!("S3 << {}", s3_path);
              }
              if fs::metadata(file).unwrap().len() > 5 * 1024 * 1024 {
                  // Use multipart upload for files larger than 5MB
                  service_s3_multipart_upload(&client, &sync_settings.bucket, s3_path, file, log).await;
              } else {
                  service_s3_upload(&client, &sync_settings.bucket, s3_path, file, log).await;
              }
          }
      }
  }

  // Placeholder for syncing files
  if !files_to_sync.is_empty() {
      let recent_action = format!("S3 << {}", files_to_sync.last().unwrap());
      println!("Files to sync in {}: {:?}", dir, files_to_sync);
      fs::write("sync.log", recent_action).expect("Unable to write to sync.log");
  }
  if !deletions.is_empty() {
      println!("Files to delete in {}: {:?}", dir, deletions);
  }
}