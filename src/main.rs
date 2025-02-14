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
mod install;

#[derive(Clone, Eq, PartialEq, Debug)]
enum UserEvents {
    Exit,
    RightClick,
    Help,
}

#[derive(Serialize, Deserialize)]
struct Settings {
    directories_to_scan: Vec<String>,
    seconds_between_scans: u64,
}

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
    thread::spawn(move || loop {
        for dir in &settings.directories_to_scan {
            match fs::read_dir(dir) {
                Ok(entries) => {
                    for entry in entries {
                        if let Ok(entry) = entry {
                            println!("Found file: {:?}", entry.path());
                        }
                    }
                }
                Err(e) => eprintln!("Failed to read directory {}: {}", dir, e),
            }
        }
        thread::sleep(Duration::from_secs(settings.seconds_between_scans));
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
        }
    }
}
