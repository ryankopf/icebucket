use iced::widget::{button, column, container, row, text, text_input};
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

#[derive(Default)]
struct IceBucketGui {
    settings: SettingsData,
    new_directory: String,
    adding_directory: bool,
}

#[derive(Debug, Clone)]
enum Message {
    DirectoryClicked(String),
    RemoveDirectory(String),
    AddDirectory,
    UpdateNewDirectory(String),
    ConfirmAddDirectory,
    CancelAddDirectory,
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
                println!("Directory clicked: {}", directory);
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
        }
    }

    fn view(&self) -> Element<Message> {
        if self.adding_directory {
            return container(
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
            .center_x()
            .center_y()
            .into();
        }

        let directory_list: Vec<Element<Message>> = self
            .settings
            .directories_to_scan
            .iter()
            .map(|dir| {
                row![
                    button(
                        text(dir.clone())
                    ).on_press(Message::DirectoryClicked(dir.clone())),
                    button("[-]").on_press(Message::RemoveDirectory(dir.clone()))
                ]
                .spacing(10)
                .into()
            })
            .collect();

        container(
            column![
                column(directory_list).spacing(10),
                button("[Add Directory]").on_press(Message::AddDirectory),
            ]
            .spacing(20)
            .align_items(Alignment::Center),
        )
        .center_x()
        .center_y()
        .into()
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

fn main() -> iced::Result {
    let settings = load_settings_sync();
    IceBucketGui::run(Settings {
        flags: settings,
        ..Settings::default()
    })
}
