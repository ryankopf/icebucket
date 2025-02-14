use std::env;
use std::fs;
use std::path::PathBuf;
use winreg::enums::*;
use winreg::RegKey;

const PROGRAM_NAME: &str = "IceBucket";
const EXECUTABLE_NAME: &str = "icebucket.exe";

pub fn add_to_startup() -> Result<(), ()> {
    let appdata = match env::var("APPDATA") {
        Ok(path) => path,
        Err(_) => {
            println!("Failed to get APPDATA environment variable");
            return Err(());
        }
    };
    let target_dir = PathBuf::from(appdata).join(PROGRAM_NAME);
    let target_path = target_dir.join(EXECUTABLE_NAME);
    if fs::create_dir_all(&target_dir).is_err() {
        println!("Failed to create directory: {:?}", target_dir);
        return Err(());
    }
    let current_exe = match env::current_exe() {
        Ok(path) => path,
        Err(_) => {
            println!("Failed to get current executable path");
            return Err(());
        }
    };
    if fs::copy(current_exe, &target_path).is_err() {
        println!("Failed to copy executable to: {:?}", target_path);
        return Err(());
    }
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (key, _) = match hkcu.create_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Run") {
        Ok(pair) => pair,
        Err(_) => {
            println!("Failed to create registry key");
            return Err(());
        }
    };
    if key.set_value(PROGRAM_NAME, &target_path.to_string_lossy().to_string()).is_err() {
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
    let target_path = target_dir.join(EXECUTABLE_NAME);

    if fs::remove_file(&target_path).is_err() {
        println!("Failed to remove executable from: {:?}", target_path);
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
