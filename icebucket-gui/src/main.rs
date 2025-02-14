use iced::widget::{button, column, container, row, text, text_input, pick_list};
use iced::{Alignment, Application, Command, Element, Length, Settings, Theme};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

const SETTINGS_FILE: &str = "settings.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SettingsData {
    directories_to_scan: Vec<String>,
    seconds_between_scans: u64,
}

impl Default for SettingsData {
    fn default() -> Self {
        Self {
            directories_to_scan: vec![],
            seconds_between_scans: 60,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SyncSettings {
    access_key: String,
    secret_key: String,
    bucket: String,
    region: String,
    endpoint: String,
    service: String,
    sync_type: String,
    conflicts: String,
}

impl Default for SyncSettings {
    fn default() -> Self {
        Self {
            access_key: String::new(),
            secret_key: String::new(),
            bucket: String::new(),
            region: String::new(),
            endpoint: String::new(),
            service: "s3".to_string(),
            sync_type: "upload-only".to_string(),
            conflicts: "keep-local".to_string(),
        }
    }
}

#[derive(Default)]
struct IceBucketGui {
    settings: SettingsData,
    new_directory: String,
    adding_directory: bool,
    selected_directory: Option<String>,
    sync_settings: SyncSettings,
    editing_sync_settings: bool,
}

#[derive(Debug, Clone)]
enum Message {
    DirectoryClicked(String),
    RemoveDirectory(String),
    AddDirectory,
    UpdateNewDirectory(String),
    ConfirmAddDirectory,
    CancelAddDirectory,
    UpdateSyncSettings(SyncSettings),
    SaveSyncSettings,
    CancelSyncSettings,
}

impl IceBucketGui {
    fn view_add_directory(&self) -> Element<Message> {
        container(
            column![
                text("Enter Directory:"),
                text_input("Path", &self.new_directory)
                    .on_input(Message::UpdateNewDirectory)
                    .padding(10)
                    .width(Length::Fill),
                row![
                    button("Add").on_press(Message::ConfirmAddDirectory),
                    button("Cancel").on_press(Message::CancelAddDirectory),
                ]
                .spacing(10)
            ]
            .spacing(20)
            .align_items(Alignment::Center),
        )
        .padding(25)
        .center_x()
        .center_y()
        .into()
    }

    fn view_sync_settings(&self) -> Element<Message> {
        let services = vec!["s3".to_string(), "other".to_string()];
        let sync_types = vec!["upload-only".to_string(), "download-only".to_string(), "sync".to_string()];
        let conflicts = vec!["keep-local".to_string(), "keep-remote".to_string()];

        container(
            column![
                text("Sync Settings").size(30),
                text_input("Access Key", &self.sync_settings.access_key)
                    .on_input(|v| Message::UpdateSyncSettings(SyncSettings { access_key: v, ..self.sync_settings.clone() }))
                    .padding(10)
                    .width(Length::Fill),
                text_input("Secret Key", &self.sync_settings.secret_key)
                    .on_input(|v| Message::UpdateSyncSettings(SyncSettings { secret_key: v, ..self.sync_settings.clone() }))
                    .padding(10)
                    .width(Length::Fill),
                text_input("Bucket", &self.sync_settings.bucket)
                    .on_input(|v| Message::UpdateSyncSettings(SyncSettings { bucket: v, ..self.sync_settings.clone() }))
                    .padding(10)
                    .width(Length::Fill),
                text_input("Region", &self.sync_settings.region)
                    .on_input(|v| Message::UpdateSyncSettings(SyncSettings { region: v, ..self.sync_settings.clone() }))
                    .padding(10)
                    .width(Length::Fill),
                text_input("Endpoint", &self.sync_settings.endpoint)
                    .on_input(|v| Message::UpdateSyncSettings(SyncSettings { endpoint: v, ..self.sync_settings.clone() }))
                    .padding(10)
                    .width(Length::Fill),
                pick_list(services.clone(), Some(self.sync_settings.service.clone()), |v| Message::UpdateSyncSettings(SyncSettings { service: v, ..self.sync_settings.clone() })),
                pick_list(sync_types.clone(), Some(self.sync_settings.sync_type.clone()), |v| Message::UpdateSyncSettings(SyncSettings { sync_type: v, ..self.sync_settings.clone() })),
                pick_list(conflicts.clone(), Some(self.sync_settings.conflicts.clone()), |v| Message::UpdateSyncSettings(SyncSettings { conflicts: v, ..self.sync_settings.clone() })),
                row![
                    button("Save").on_press(Message::SaveSyncSettings),
                    button("Cancel").on_press(Message::CancelSyncSettings),
                ]
                .spacing(10)
            ]
            .spacing(20)
            .align_items(Alignment::Center),
        )
        .padding(25)
        .center_x()
        .center_y()
        .into()
    }

    fn view_directory_list(&self) -> Element<Message> {
        let directory_list: Vec<Element<Message>> = self
            .settings
            .directories_to_scan
            .iter()
            .map(|dir| {
                container(
                    row![
                        button(text(dir.clone()))
                            .on_press(Message::DirectoryClicked(dir.clone()))
                            .width(Length::Fill),
                        button("-")
                            .on_press(Message::RemoveDirectory(dir.clone()))
                            .style(iced::theme::Button::Destructive),
                    ]
                    .spacing(10)
                )
                .width(Length::Fill)
                .padding(10)
                .style(iced::theme::Container::default())
                .into()
            })
            .collect();

        container(
            column![
                column(directory_list).spacing(10),
                button("Add Directory").on_press(Message::AddDirectory),
            ]
            .spacing(20)
            .align_items(Alignment::Center),
        )
        .padding(25)
        .center_x()
        .center_y()
        .into()
    }

    fn view(&self) -> Element<Message> {
        if self.adding_directory {
            self.view_add_directory()
        } else if self.editing_sync_settings {
            self.view_sync_settings()
        } else {
            self.view_directory_list()
        }
    }
}

impl Application for IceBucketGui {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = SettingsData;

    fn new(flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (
            Self {
                settings: flags,
                new_directory: String::new(),
                adding_directory: false,
                selected_directory: None,
                sync_settings: SyncSettings::default(),
                editing_sync_settings: false,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        "IceBucket GUI".to_string()
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::DirectoryClicked(directory) => {
                self.selected_directory = Some(directory.clone());
                self.sync_settings = load_sync_settings(&directory);
                self.editing_sync_settings = true;
                Command::none()
            }
            Message::RemoveDirectory(directory) => {
                self.settings.directories_to_scan.retain(|d| d != &directory);
                save_settings_sync(&self.settings);
                Command::none()
            }
            Message::AddDirectory => {
                self.adding_directory = true;
                self.new_directory.clear();
                Command::none()
            }
            Message::UpdateNewDirectory(new_value) => {
                self.new_directory = new_value;
                Command::none()
            }
            Message::ConfirmAddDirectory => {
                if !self.new_directory.trim().is_empty() {
                    self.settings.directories_to_scan.push(self.new_directory.clone());
                    save_settings_sync(&self.settings);
                }
                self.adding_directory = false;
                Command::none()
            }
            Message::CancelAddDirectory => {
                self.adding_directory = false;
                Command::none()
            }
            Message::UpdateSyncSettings(new_settings) => {
                self.sync_settings = new_settings;
                Command::none()
            }
            Message::SaveSyncSettings => {
                if let Some(ref dir) = self.selected_directory {
                    save_sync_settings(dir, &self.sync_settings);
                }
                self.editing_sync_settings = false;
                Command::none()
            }
            Message::CancelSyncSettings => {
                self.editing_sync_settings = false;
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        if self.adding_directory {
            self.view_add_directory()
        } else if self.editing_sync_settings {
            self.view_sync_settings()
        } else {
            self.view_directory_list()
        }
    }
}

/// Load settings **before launching the GUI**
fn load_settings_sync() -> SettingsData {
    if Path::new(SETTINGS_FILE).exists() {
        if let Ok(data) = fs::read_to_string(SETTINGS_FILE) {
            if let Ok(settings) = serde_json::from_str(&data) {
                return settings;
            }
        }
    }
    SettingsData::default()
}

/// Save settings immediately when a change is made
fn save_settings_sync(settings: &SettingsData) {
    if let Ok(data) = serde_json::to_string_pretty(settings) {
        let _ = fs::write(SETTINGS_FILE, data);
    }
}

fn load_sync_settings(directory: &str) -> SyncSettings {
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

fn save_sync_settings(directory: &str, settings: &SyncSettings) {
    let path = Path::new(directory).join("sync.json");
    if let Ok(data) = serde_json::to_string_pretty(settings) {
        let _ = fs::write(path, data);
    }
}

fn main() -> iced::Result {
    let settings = load_settings_sync();
    IceBucketGui::run(Settings {
        flags: settings,
        ..Settings::default()
    })
}
