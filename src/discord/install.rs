use crate::Platform;
use concat_string::concat_string; // you looove microoptimizations
use log as l;
#[cfg(target_os = "windows")]
use path_slash::PathBufExt as _;
use std::{error::Error, path::PathBuf};

use super::DiscordKind;
#[cfg(target_os = "linux")]
use super::Flatpak;

const PACKAGE_JSON: &str = r#"{
    "name": "discord",
    "main": "./injector.js",
    "private": true
}"#;

const INJECTOR_1: &str = r#"require(""#;
const INJECTOR_2: &str = r#"").inject(require("path").resolve(__dirname, "../_app.asar"));"#;

#[derive(Debug)]
pub struct DiscordInstall {
    pub kind: DiscordKind,
    pub path: PathBuf,
    pub injected: bool,
    pub is_openasar: bool,
    #[cfg(target_os = "linux")]
    pub flatpak: Flatpak,
    #[cfg(target_os = "linux")]
    pub is_sys_electron: bool,
}

impl DiscordInstall {
    pub fn new(kind: DiscordKind, mut path: PathBuf) -> Option<Self> {
        l::info!("Checking if {:?} is a valid Discord install...", path);
        use std::fs;
        #[cfg(target_os = "windows")]
        let (is_valid, injected) = {
            let mut app_folders = fs::read_dir(path.clone())
                .ok()?
                .filter_map(|entry| entry.ok())
                .filter(|entry| entry.file_name().to_string_lossy().contains("app-"))
                .map(|entry| entry.path())
                .collect::<Vec<_>>();
            app_folders.sort_by(|a, b| {
                semver::Version::parse(
                    a.file_name()
                        .unwrap()
                        .to_string_lossy()
                        .replace("app-", "")
                        .as_str(),
                )
                .unwrap()
                .cmp(
                    &semver::Version::parse(
                        b.file_name()
                            .unwrap()
                            .to_string_lossy()
                            .replace("app-", "")
                            .as_str(),
                    )
                    .unwrap(),
                )
            });
            app_folders.reverse();
            let app_folder = app_folders.first()?;
            path = app_folder.clone();
            (
                Some(
                    app_folder
                        .join(concat_string!(kind.to_string(), ".exe"))
                        .exists(),
                ),
                if app_folder.join("resources").join("app").exists()
                    || app_folder.join("resources").join("app.asar").is_dir()
                {
                    true
                } else {
                    false
                },
            )
        };

        #[cfg(target_os = "macos")]
        let (is_valid, injected) = {
            let resources_folder = path.join("Contents/Resources");
            (
                Some(path.exists() && resources_folder.exists()),
                if resources_folder.join("app").exists()
                    || resources_folder.join("app.asar").is_dir()
                {
                    true
                } else {
                    false
                },
            )
        };

        #[cfg(target_os = "linux")]
        let (is_valid, injected, is_sys_electron, flatpak) = {
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

            let injected = {
                if (sys_electron && path.join("_app.asar.unpacked").exists())
                    || (path.join("app").exists()
                        && path.join("resources").join("app.asar").is_dir())
                {
                    true
                } else {
                    false
                }
            };

            let is_valid = {
                if sys_electron {
                    path.join("app.asar").exists()
                } else {
                    path.join("resources").join("app.asar").exists()
                }
            };

            (Some(is_valid), injected, sys_electron, flatpak)
        };

        match is_valid {
            Some(true) => {
                l::info!("Found valid Discord install at {:?}", path);
                Some(Self {
                    kind,
                    path,
                    injected,
                    is_openasar: false, // TODO: openasar detection
                    #[cfg(target_os = "linux")]
                    flatpak,
                    #[cfg(target_os = "linux")]
                    is_sys_electron,
                })
            }
            Some(false) => {
                l::info!("Found invalid Discord install at {:?}", path);
                None
            }
            None => {
                l::info!("Found no Discord install at {:?}", path);
                None
            }
        }
    }

    pub async fn inject(&self, moonlight_root: &PathBuf) -> Result<(), Box<dyn Error>> {
        if self.injected {
            l::warn!(
                "Discord install at {:?} is already injected, uninjecting first",
                self.path
            );
            self.uninject().await?;
            l::info!("Reinjecting Discord {:?}", self.kind)
        }

        self.move_discord_items().await?;
        l::info!("Writing injection files");
        self.write_injection_files(moonlight_root).await?;
        Ok(())
    }

    pub async fn uninject(&self) -> Result<(), Box<dyn Error>> {
        if !self.injected {
            l::warn!("Discord install at {:?} is not injected", self.path);
            return Ok(());
        }
        l::info!("Resetting Discord {:?}", self.kind);
        self.unmove_discord_items().await?;
        self.rm_injection_files().await?;
        Ok(())
    }

    #[inline(always)]
    pub async fn modify_moonlight_root(
        &self,
        moonlight_root: &PathBuf,
    ) -> Result<(), Box<dyn Error>> {
        if !self.injected {
            return self.inject(moonlight_root).await;
        }
        l::info!(
            "Modifying Moonlight root for Discord install at {:?}",
            self.path
        );

        self.write_injection_files(moonlight_root).await?;

        Ok(())
    }
    #[inline(always)]
    async fn move_discord_items(&self) -> Result<(), Box<dyn Error>> {
        use std::fs;
        let mut root_path = self.path.clone();
        #[cfg(target_os = "linux")]
        {
            if !self.is_sys_electron {
                root_path = root_path.join("resources");
            }
        }
        #[cfg(target_os = "windows")]
        {
            root_path = root_path.join("resources");
        }
        #[cfg(target_os = "macos")]
        {
            root_path = root_path.join("Contents/Resources");
        }

        l::debug!("Moving Discord items from {:?}", root_path);
        let app_asar = root_path.join("app.asar");
        let _app_asar = root_path.join("_app.asar");
        if app_asar.exists() {
            fs::rename(app_asar, _app_asar)?;
        }
        Ok(())
    }
    #[inline(always)]
    async fn unmove_discord_items(&self) -> Result<(), Box<dyn Error>> {
        use std::fs;
        let mut root_path = self.path.clone();
        #[cfg(target_os = "linux")]
        {
            if !self.is_sys_electron {
                root_path = root_path.join("resources");
            }
        }

        #[cfg(target_os = "windows")]
        {
            root_path = root_path.join("resources");
        }
        #[cfg(target_os = "macos")]
        {
            root_path = root_path.join("Contents/Resources");
        }
        let app_asar = root_path.join("app.asar");
        let _app_asar = root_path.join("_app.asar");
        if app_asar.exists() {
            if app_asar.is_dir() {
                fs::remove_dir_all(app_asar.clone())?;
            } else if _app_asar.exists() {
                fs::remove_file(app_asar.clone())?;
            }
        }
        fs::rename(_app_asar, app_asar)?;
        Ok(())
    }

    #[inline(always)]
    async fn write_injection_files(&self, moonlight_root: &PathBuf) -> Result<(), Box<dyn Error>> {
        use std::fs;
        let mut root_path = self.path.clone();
        #[cfg(target_os = "linux")]
        {
            if !self.is_sys_electron {
                root_path = root_path.join("resources");
            }
        }
        #[cfg(target_os = "windows")]
        {
            root_path = root_path.join("resources");
        }
        #[cfg(target_os = "macos")]
        {
            root_path = root_path.join("Contents/Resources");
        }
        root_path = root_path.join("app");
        if !root_path.exists() {
            fs::create_dir_all(&root_path)?;
        }
        l::debug!("Writing injection files to {:?}", root_path);
        let package_json = root_path.join("package.json");
        fs::write(package_json, PACKAGE_JSON)?;
        let injector_js = root_path.join("injector.js");
        #[cfg(target_os = "windows")]
        fs::write(
            injector_js,
            concat_string!(
                INJECTOR_1,
                moonlight_root
                    .join("dist")
                    .join("injector.js")
                    .to_slash()
                    .unwrap(),
                INJECTOR_2
            ),
        )?;
        #[cfg(not(target_os = "windows"))]
        fs::write(
            injector_js,
            concat_string!(
                INJECTOR_1,
                moonlight_root
                    .join("dist")
                    .join("injector.js")
                    .to_string_lossy(),
                INJECTOR_2
            ),
        )?;
        Ok(())
    }
    #[inline(always)]
    async fn rm_injection_files(&self) -> Result<(), Box<dyn Error>> {
        use std::fs;
        let mut root_path = self.path.clone();
        #[cfg(target_os = "linux")]
        {
            if !self.is_sys_electron {
                root_path = root_path.join("resources");
            }
        }
        #[cfg(target_os = "windows")]
        {
            root_path = root_path.join("resources");
        }
        #[cfg(target_os = "macos")]
        {
            root_path = root_path.join("Contents/Resources");
        }
        root_path = root_path.join("app");
        if root_path.exists() {
            fs::remove_dir_all(&root_path)?;
        }
        let old_asar_path = &root_path.join("_app.asar");
        let asar_path = &root_path.join("app.asar");
        if old_asar_path.exists() {
            fs::rename(old_asar_path, asar_path)?;
        }
        Ok(())
    }

    pub async fn kill(&self) -> Result<(), Box<dyn Error>> {
        let path = self.path.clone();
        #[cfg(target_os = "windows")]
        {
            if !Platform::win_termbyname(concat_string!(self.kind.to_string(), ".exe")) {
                return Err("Failed to kill Discord".into());
            }
        }
        #[cfg(target_os = "macos")]
        {
            if !Platform::cmd_is_ok(
                vec![
                    "killall".to_owned(),
                    // The executable name is the same as the folder name
                    path.file_name()
                        .unwrap()
                        .to_string_lossy()
                        .to_string()
                        .replace(".app", ""),
                ],
                None,
            ) {
                return Err("Failed to kill Discord".into());
            }
        }
        #[cfg(target_os = "linux")]
        {
            match self.flatpak {
                Flatpak::System => {
                    if !Platform::cmd_is_ok(
                        vec![
                            "flatpak".to_owned(),
                            "kill".to_owned(),
                            "com.discordapp.Discord".to_owned(),
                        ],
                        None,
                    ) {
                        return Err("Failed to kill Discord".into());
                    }
                }
                Flatpak::User => {
                    if !Platform::cmd_is_ok(
                        vec![
                            "flatpak".to_owned(),
                            "kill".to_owned(),
                            "--user".to_owned(),
                            "com.discordapp.Discord".to_owned(),
                        ],
                        None,
                    ) {
                        return Err("Failed to kill Discord".into());
                    }
                }
                Flatpak::Not => {
                    if !Platform::cmd_is_ok(vec!["killall".to_owned(), self.kind.to_string()], None)
                    {
                        return Err("Failed to kill Discord".into());
                    };
                }
            }
        }
        Ok(())
    }
    pub async fn start(&self) -> Result<(), Box<dyn Error>> {
        #[cfg(target_os = "windows")]
        {
            match Platform::disown_launch(vec![self
                .path
                .join(concat_string!(self.kind.to_string(), ".exe"))
                .to_string_lossy()
                .to_string()])
            {
                Ok(_) => {}
                Err(e) => {
                    return Err(format!("Failed to start Discord: {}", e).into());
                }
            }
        }
        #[cfg(target_os = "macos")]
        {
            match Platform::disown_launch(vec![self
                .path
                .join("Contents/MacOS/Discord")
                .to_string_lossy()
                .to_string()])
            {
                Ok(_) => {}
                Err(e) => {
                    return Err(format!("Failed to start Discord: {}", e).into());
                }
            }
        }
        #[cfg(target_os = "linux")]
        {
            match self.flatpak {
                Flatpak::System => {
                    if !Platform::cmd_is_ok(
                        vec![
                            "flatpak".to_owned(),
                            "run".to_owned(),
                            "com.discordapp.Discord".to_owned(),
                        ],
                        None,
                    ) {
                        return Err("Failed to start Discord".into());
                    }
                }
                Flatpak::User => {
                    if !Platform::cmd_is_ok(
                        vec![
                            "flatpak".to_owned(),
                            "run".to_owned(),
                            "--user".to_owned(),
                            "com.discordapp.Discord".to_owned(),
                        ],
                        None,
                    ) {
                        return Err("Failed to start Discord".into());
                    }
                }
                Flatpak::Not => {
                    return Err("Not implemented: start Discord without flatpak".into());
                }
            }
        }
        Ok(())
    }
}
