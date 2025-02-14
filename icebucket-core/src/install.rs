use std::env;
use std::fs;
use std::path::PathBuf;
use winreg::enums::*;
use winreg::RegKey;

const PROGRAM_NAME: &str = "IceBucket";
const EXECUTABLES: &[&str] = &["icebucket-core.exe", "icebucket-gui.exe"];

pub fn add_to_startup() -> Result<(), ()> {
    let appdata = match env::var("APPDATA") {
        Ok(path) => path,
        Err(_) => {
            println!("Failed to get APPDATA environment variable");
            return Err(());
        }
    };
    let target_dir = PathBuf::from(appdata).join(PROGRAM_NAME);
    if fs::create_dir_all(&target_dir).is_err() {
        println!("Failed to create directory: {:?}", target_dir);
        return Err(());
    }
    for &exe in EXECUTABLES {
        let target_path = target_dir.join(exe);
        let current_exe = match env::current_exe() {
            Ok(path) => path.with_file_name(exe),
            Err(_) => {
                println!("Failed to get current executable path for {}", exe);
                return Err(());
            }
        };
        if fs::copy(&current_exe, &target_path).is_err() {
            println!("Failed to copy executable {} to: {:?}", exe, target_path);
            return Err(());
        }
    }
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (key, _) = match hkcu.create_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Run") {
        Ok(pair) => pair,
        Err(_) => {
            println!("Failed to create registry key");
            return Err(());
        }
    };
    if key.set_value(PROGRAM_NAME, &target_dir.to_string_lossy().to_string()).is_err() {
        println!("Failed to set registry value");
        return Err(());
    }
    Ok(())
}

pub fn remove_from_startup() -> Result<(), ()> {
    let appdata = match env::var("APPDATA") {
        Ok(path) => path,
        Err(_) => {
            println!("Failed to get APPDATA environment variable");
            return Err(());
        }
    };
    let target_dir = PathBuf::from(appdata).join(PROGRAM_NAME);

    for &exe in EXECUTABLES {
        let target_path = target_dir.join(exe);
        if fs::remove_file(&target_path).is_err() {
            println!("Failed to remove executable {} from: {:?}", exe, target_path);
        }
    }

    if fs::remove_dir_all(&target_dir).is_err() {
        println!("Failed to remove directory: {:?}", target_dir);
    }

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let key = match hkcu.open_subkey_with_flags("Software\\Microsoft\\Windows\\CurrentVersion\\Run", KEY_SET_VALUE) {
        Ok(key) => key,
        Err(_) => {
            println!("Failed to open registry key");
            return Err(());
        }
    };

    if key.delete_value(PROGRAM_NAME).is_err() {
        println!("Failed to delete registry value");
        return Err(());
    }

    Ok(())
}
