use std::env;
use std::{thread, time::Duration, process::Command};
use trayicon::{Icon, MenuBuilder, MenuItem, TrayIcon, TrayIconBuilder};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::WindowId,
};
use windows::Win32::System::Console::FreeConsole;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::SystemTime;
use tokio::runtime::Runtime;
use sysinfo::System;
mod install;
mod services;
mod settings;
mod sync;
mod logger;
use logger::Log;
use settings::load_or_create_settings;
use sync::sync_directory;

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
    public: bool,
}

static VERBOSE: AtomicBool = AtomicBool::new(false);

fn main() {
    // unsafe {
    //     let _ = FreeConsole(); // Hides console window
    // }
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
            let mut log = Log::new(); // Initialize the log
            for dir in &settings.directories_to_scan {
                file_maps.insert(dir.clone(), HashMap::new());
            }
            loop {
                for dir in &settings.directories_to_scan {
                    if let Some(file_map) = file_maps.get_mut(dir) {
                        if VERBOSE.load(Ordering::Relaxed) {
                            println!("Syncing directory: {}", dir);
                        }
                        sync_directory(dir, file_map, &mut log).await; // Pass log to sync_directory
                    }
                }
                thread::sleep(Duration::from_secs(settings.seconds_between_scans));
            }
        });
    });

    let mut app = TrayApp { tray_icon };
    event_loop.run_app(&mut app).unwrap();
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