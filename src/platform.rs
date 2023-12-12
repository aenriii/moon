use std::process::ExitStatus;

use crate::discord::{DiscordKind, DiscordInstall};

use log as l;

pub struct Platform;

impl Platform {

    #[inline(always)]
    pub fn conf_dir() -> String {
        #[cfg(target_os = "windows")]
        return format!("{}/{}", env("APPDATA", &env("USERPROFILE", "./")), "Moon");
        #[cfg(target_os = "macos")]
        return format!("{}/{}", env("HOME", "~/"), "Library/Application Support/Moon");
        #[cfg(target_os = "linux")]
        return format!("{}/{}", env("HOME", "~/"), ".config/moon");
    }
    #[inline(always)]
    pub fn cmd_is_ok(mut parts: Vec<String>) -> bool {
        use std::process::Command;
        let mut cmd = Command::new(parts.remove(0));
        cmd.args(parts);
        match cmd.status() {
            Ok(s) => s.success(),
            _ => false,
        }
    }

    #[cfg(target_os = "windows")]
    #[inline(always)]
    pub fn installs_by_kind(kind: DiscordKind) -> Vec<DiscordInstall> {
        use std::path::PathBuf;

        if let Some(install) = DiscordInstall::new(
            kind,
            PathBuf::from(
                format!(
                    "{}/{}",
                    env!("LOCALAPPDATA", ""),
                    kind.to_string()
                )
            )
        ) {
            l::info!("Found Discord install for {:?} at {:?}", kind, install.path);
            vec![install]
        } else {
            l::warn!("No Discord install found for {:?}", kind);
            vec![]
        }
    }

    #[cfg(target_os = "macos")]
    #[inline(always)]
    pub fn installs_by_kind(kind: DiscordKind) -> Vec<DiscordInstall> {

    }

    #[cfg(target_os = "linux")]
    #[inline(always)]

    pub fn installs_by_kind(kind: DiscordKind) -> Vec<DiscordInstall> {

    }

    #[cfg(target_os = "windows")]
    #[inline(always)]
    pub fn pretransaction_checks() -> Result<(), String> {
        Ok(())
    }

    #[cfg(target_os = "macos")]
    #[inline(always)]
    pub fn pretransaction_checks() -> Result<(), String> {
        Ok(())
    }

    #[cfg(target_os = "linux")]
    #[inline(always)]
    pub fn pretransaction_checks() -> Result<(), String> {
        let sudoer = env("SUDO_USER", &env("DOAS_USER", ""));
        if &sudoer == "root" {
            Err("You cannot run moon as the root user!".to_string())
        } else {
            Ok(())
        }
    }
}

#[inline(always)]
pub fn env(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}