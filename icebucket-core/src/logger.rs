use serde::{Serialize, Deserialize};
use std::time::{SystemTime, UNIX_EPOCH};
use std::fs;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LogEntry {
    pub file_path: String,
    pub destination_service: String,
    pub destination_bucket: String,
    pub completed_bytes: u64,
    pub total_bytes: u64,
    pub completed: bool,
    pub updated_at: u64,
}

impl LogEntry {
    pub fn new(file_path: String, destination_service: String, destination_bucket: String, total_bytes: u64) -> Self {
        LogEntry {
            file_path,
            destination_service,
            destination_bucket,
            completed_bytes: 0,
            total_bytes,
            completed: false,
            updated_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        }
    }
}

pub struct Log {
    pub entries: Vec<LogEntry>,
}

impl Log {
    pub fn new() -> Self {
        Log { entries: Vec::new() }
    }

    pub fn add_entry(&mut self, entry: LogEntry) {
        self.entries.push(entry);
        self.write_to_file();
    }

    pub fn update_entry(&mut self, file_path: &str, completed_bytes: u64) {
        if let Some(entry) = self.entries.iter_mut().find(|e| e.file_path == file_path) {
            entry.completed_bytes = completed_bytes;
            entry.completed = true;
            entry.updated_at = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        }
        self.write_to_file();
    }

    fn write_to_file(&self) {
        let mut sorted_entries = self.entries.clone();
        sorted_entries.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        let recent_entries: Vec<LogEntry> = sorted_entries.into_iter().take(4).collect();
        let log_content = serde_json::to_string(&recent_entries).expect("Failed to serialize log entries");
        fs::write("sync.log", log_content).expect("Unable to write to sync.log");
    }
}
