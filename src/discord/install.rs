use std::path::PathBuf;
use log as l;

use super::{DiscordKind, Injection};

#[cfg(target_os = "linux")]
use super::Flatpak;

#[derive(Debug)]
pub struct DiscordInstall {
    pub kind: DiscordKind,
    pub path: PathBuf,
    pub injection: bool,
    pub is_openasar: bool,
    #[cfg(target_os = "linux")]
    pub flatpak: Flatpak,
    #[cfg(target_os = "linux")]
    pub is_sys_electron: bool,
}

impl DiscordInstall {
    pub fn new(kind: DiscordKind, path: PathBuf) -> Option<Self> {
        l::info!("Checking if {:?} is a valid Discord install...", path);
        use std::fs;
        #[cfg(target_os = "windows")]
        let (is_valid, injection) = {
            let mut app_folders = fs::read_dir(path.clone()).ok()?
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.file_name().to_string_lossy().contains("app-"))
            .map(|entry| entry.path())
            .collect::<Vec<_>>();
            app_folders.sort_by(
                |a, b| {
                    semver::Version::parse(
                        a.file_name().unwrap().to_string_lossy().replace("app-", "").as_str()
                    ).unwrap().cmp(
                        &semver::Version::parse(
                            b.file_name().unwrap().to_string_lossy().replace("app-", "").as_str()
                        ).unwrap()
                    )
                }
            );
            app_folders.reverse();
            let app_folder = app_folders.first()?;
            (
                Some(app_folder.join("Discord.exe").exists()),
                if app_folder.join("resources").join("app").exists()
                    || app_folder.join("resources").join("app.asar").is_dir() {
                    Injection::Vencord
                } else {
                    Injection::None
                }
            )
        };

        #[cfg(target_os = "macos")]
        let (is_valid, injection) = {
            let resources_folder = path.join("Contents/Resources");
            (
                Some(path.exists() && resources_folder.exists()),
                if resources_folder.join("app").exists() || resources_folder.join("app.asar").is_dir() {
                    Injection::Vencord
                } else {
                    Injection::None
                }
            )
        };

        #[cfg(target_os = "linux")]
        let (is_valid, injection, is_sys_electron, flatpak) = {
            let sys_electron = path.join("app.asar").exists();
            // if path contains /flatpak/ then it's a flatpak install
            let flatpak = path.to_string_lossy().contains("/flatpak/");
            let sys_flatpak = flatpak && path.to_string_lossy().contains("/var"); // todo: change to starts_with
            let flatpak = {
                if sys_flatpak {
                    Flatpak::System
                } else if flatpak {
                    Flatpak::User
                } else {
                    Flatpak::Not
                }
            };

            let Injection = {
                if (sys_electron && path.join("_app.asar.unpacked").exists()) 
                || (path.join("app").exists() && path.join("resources").join("app.asar").is_dir()) {
                    Injection::Vencord
                } else {
                    Injection::None
                }
            };

            let is_valid = {
                if sys_electron {
                    path.join("app.asar").exists()
                } else {
                    path.join("resources").join("app.asar").exists()
                }
            };

            (Some(is_valid), injection, sys_electron, flatpak)

        };

        match is_valid {
            Some(true) => {
                l::info!("Found valid Discord install at {:?}", path);
                Some(Self {
                    kind,
                    path,
                    injection,
                    is_openasar: false, // TODO: openasar detection
                    #[cfg(target_os = "linux")]
                    flatpak,
                    #[cfg(target_os = "linux")]
                    is_sys_electron,
                })
            },
            Some(false) => {
                l::info!("Found invalid Discord install at {:?}", path);
                None
            },
            None => {
                l::info!("Found no Discord install at {:?}", path);
                None
            }
        }
    }
    pub async fn apply_moonlight(&mut self, moonlight_dir: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        todo!()
    }
    pub async fn apply_openasar(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        todo!()
    }
    pub async fn remove_moonlight(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        todo!()
    }
    pub async fn remove_openasar(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        todo!()
    }
}