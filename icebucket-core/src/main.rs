use std::env;
use std::{fs, thread, time::Duration, process::Command};
use trayicon::{Icon, MenuBuilder, MenuItem, TrayIcon, TrayIconBuilder};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::WindowId,
};
use windows::Win32::System::Console::FreeConsole;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::SystemTime;
use aws_sdk_s3::{Client, config::Region};
use aws_config::BehaviorVersion;
use aws_sdk_s3::config::Credentials;
use aws_config::meta::region::RegionProviderChain;
use tokio::runtime::Runtime;
use sysinfo::System;//, SystemExt, ProcessExt};
mod install;
mod services;
use services::s3::{service_s3_check, service_s3_upload, service_s3_multipart_upload};
mod sync;
use sync::create_default_sync_settings;

// This program is a simple file sync tool that runs in the system tray.
// It scans specified directories for files and syncs the changes to
// a service like AWS S3, using a sync.json file for configuration in the folder.

#[derive(Clone, Eq, PartialEq, Debug)]
enum UserEvents {
    Exit,
    RightClick,
    Help,
    LeftClick,  // Add new event for left click
}

#[derive(Serialize, Deserialize)]
struct Settings {
    directories_to_scan: Vec<String>,
    seconds_between_scans: u64,
}

#[derive(Serialize, Deserialize)]
struct SyncSettings {
    service: String,
    access_key: String,
    secret_key: String,
    region: String,
    bucket: String,
    endpoint: String,
    sync_type: String,
    conflicts: String,
}

static VERBOSE: AtomicBool = AtomicBool::new(false);

fn main() {
    unsafe {
        let _ = FreeConsole(); // Hides console window
    }
    let args: Vec<String> = env::args().collect();
    if args.contains(&"--install".to_string()) {
        match install::add_to_startup() {
            Ok(_) => println!("Added to startup successfully."),
            Err(_) => eprintln!("Failed to add to startup."),
        }
        return;
    }
    if args.contains(&"--uninstall".to_string()) {
        match install::remove_from_startup() {
            Ok(_) => println!("Removed from startup successfully."),
            Err(_) => eprintln!("Failed to remove from startup."),
        }
        return;
    }
    if args.contains(&"--verbose".to_string()) || env::var("CARGO").is_ok() {
        VERBOSE.store(true, Ordering::Relaxed);
    }

    let settings = load_or_create_settings();

    let event_loop = EventLoop::<UserEvents>::with_user_event().build().unwrap();
    let proxy = event_loop.create_proxy();

    let icon_data = include_bytes!("../src/icon1.ico");
    let icon = Icon::from_buffer(icon_data, None, None).unwrap();

    let tray_icon = TrayIconBuilder::new()
        .sender(move |e: &UserEvents| {
            let _ = proxy.send_event(e.clone());
        })
        .icon(icon)
        .tooltip("Folder Sync")
        .on_right_click(UserEvents::RightClick)
        .on_click(UserEvents::LeftClick)  // Handle left click
        .menu(
            MenuBuilder::new()
                .with(MenuItem::Item { 
                    name: "Kopf Robotics IceBucket".into(), 
                    disabled: true,  // This makes it non-clickable
                    id: UserEvents::RightClick,
                    icon: None,
                })
                .separator()
                .item("Help", UserEvents::Help)
                .separator()
                .item("Exit", UserEvents::Exit),
        )
        .build()
        .unwrap();

    // Background sync loop
    thread::spawn(move || {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let mut file_maps: HashMap<String, HashMap<String, SystemTime>> = HashMap::new();
            for dir in &settings.directories_to_scan {
                file_maps.insert(dir.clone(), HashMap::new());
            }
            loop {
                for dir in &settings.directories_to_scan {
                    if let Some(file_map) = file_maps.get_mut(dir) {
                        if VERBOSE.load(Ordering::Relaxed) {
                            println!("Syncing directory: {}", dir);
                        }
                        sync_directory(dir, file_map).await;
                    }
                }
                thread::sleep(Duration::from_secs(settings.seconds_between_scans));
            }
        });
    });

    let mut app = TrayApp { tray_icon };
    event_loop.run_app(&mut app).unwrap();
}

fn load_or_create_settings() -> Settings {
    let settings_path = "settings.json";
    if let Ok(settings_data) = fs::read_to_string(settings_path) {
        serde_json::from_str(&settings_data).unwrap_or_else(|_| create_default_settings(settings_path))
    } else {
        create_default_settings(settings_path)
    }
}

fn create_default_settings(settings_path: &str) -> Settings {
    let default_settings = Settings {
        directories_to_scan: vec!["./".to_string()],
        seconds_between_scans: 60,
    };
    let settings_json = json!(default_settings);
    fs::write(settings_path, settings_json.to_string()).expect("Failed to write default settings");
    default_settings
}

async fn sync_directory(dir: &str, file_map: &mut HashMap<String, SystemTime>) {
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
                    service_s3_multipart_upload(&client, &sync_settings.bucket, s3_path, file).await;
                } else {
                    service_s3_upload(&client, &sync_settings.bucket, s3_path, file).await;
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

struct TrayApp {
    tray_icon: TrayIcon<UserEvents>,
}

impl ApplicationHandler<UserEvents> for TrayApp {
    fn resumed(&mut self, _event_loop: &ActiveEventLoop) {
        // No-op (not using a window)
    }

    fn window_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        _event: WindowEvent,
    ) {
        // No-op (not using a window)
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: UserEvents) {
        match event {
            UserEvents::Exit => event_loop.exit(),
            UserEvents::RightClick => {
                self.tray_icon.show_menu().unwrap();
            }
            UserEvents::Help => {
                let _ = Command::new("cmd")
                    .args(["/C", "start https://kopfrobotics.com/icebucket"])
                    .spawn();
            }
            UserEvents::LeftClick => {
                if !is_process_running("icebucket-gui.exe") {
                    let _ = Command::new("icebucket-gui.exe").spawn();
                }
            }
        }
    }
}

fn is_process_running(process_name: &str) -> bool {
    let system = System::new_all();
    for process in system.processes().values() {
        if process.name().eq_ignore_ascii_case(process_name) {
            return true;
        }
    }
    false
}