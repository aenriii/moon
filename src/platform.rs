use std::error::Error;
use std::process::{Command, ExitStatus, Stdio};

use crate::discord::{DiscordInstall, DiscordKind};

use log as l;

pub struct Platform;

impl Platform {
    #[inline(always)]
    pub fn conf_dir() -> String {
        #[cfg(target_os = "windows")]
        return format!("{}/{}", env("APPDATA", &env("USERPROFILE", "./")), "Moon");
        #[cfg(target_os = "macos")]
        return format!(
            "{}/{}",
            env("HOME", "~/"),
            "Library/Application Support/Moon"
        );
        #[cfg(target_os = "linux")]
        return format!("{}/{}", env("HOME", "~/"), ".config/moon");
    }
    #[inline(always)]
    pub fn cmd_is_ok(mut parts: Vec<String>, cwd: Option<&str>) -> bool {
        use std::process::Command;
        let mut cmd = Command::new(parts.remove(0));
        cmd.stdout(Stdio::null());
        cmd.stderr(Stdio::null());
        cmd.args(parts);
        if let Some(cwd) = cwd {
            cmd.current_dir(cwd);
        }
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
            PathBuf::from(format!("{}/{}", env!("LOCALAPPDATA", ""), kind.to_string())),
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
        use std::path::PathBuf;
        if let Some(install) = DiscordInstall::new(
            kind,
            PathBuf::from(format!(
                "/Applications/{}.app",
                match kind {
                    DiscordKind::Stable => "Discord",
                    DiscordKind::Canary => "Discord Canary",
                    DiscordKind::Ptb => "Discord PTB",
                    DiscordKind::Development => "Discord Development",
                }
            )),
        ) {
            l::info!("Found Discord install for {:?} at {:?}", kind, install.path);
            vec![install]
        } else {
            l::warn!("No Discord install found for {:?}", kind);
            vec![]
        }
    }

    #[cfg(target_os = "linux")]
    #[inline(always)]

    pub fn installs_by_kind(kind: DiscordKind) -> Vec<DiscordInstall> {}

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

    #[cfg(target_os = "windows")]
    #[inline(always)]
    pub fn disown_launch(args: Vec<String>) -> Result<ExitStatus, Box<dyn Error>> {
        let mut cmd = Command::new("cmd");
        cmd.args(vec!["/C", "start", "/b"]);
        cmd.args(args);
        cmd.stdout(Stdio::null());
        cmd.stderr(Stdio::null());
        Ok(cmd.spawn()?.wait()?)
    }

    #[cfg(target_os = "macos")]
    #[inline(always)]
    pub fn disown_launch(args: Vec<String>) -> Result<ExitStatus, Box<dyn Error>> {
        let mut cmd = Command::new("open");
        cmd.args(args);
        cmd.stdout(Stdio::null());
        cmd.stderr(Stdio::null());
        Ok(cmd.spawn()?.wait()?)
    }

    #[cfg(target_os = "linux")]
    #[inline(always)]
    pub fn disown_launch(args: Vec<String>) -> Result<ExitStatus, Box<dyn Error>> {
        let mut cmd = Command::new("sh");
        cmd.args(vec!["-c", &args.join(" ")]);
        cmd.stdout(Stdio::null());
        cmd.stderr(Stdio::null());
        Ok(cmd.spawn()?.wait()?)
    }

    #[cfg(target_os = "windows")]
    #[inline(always)]
    pub fn win_termbyname(taskname: String) -> bool {
        l::info!("Terminating process by name: {}", taskname);
        use windows::Win32::Foundation::CloseHandle;
        use windows::Win32::System::Diagnostics::ToolHelp::{
            CreateToolhelp32Snapshot, Process32First, Process32Next, PROCESSENTRY32,
            TH32CS_SNAPPROCESS,
        };
        use windows::Win32::System::Threading::{OpenProcess, TerminateProcess, PROCESS_TERMINATE};

        let process_snapshot = match unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) } {
            Ok(handle) => handle,
            _ => {
                l::error!("Failed to create process snapshot handle");
                return false;
            }
        };

        let mut proc_entry: PROCESSENTRY32 = Default::default();
        proc_entry.dwSize = std::mem::size_of::<PROCESSENTRY32>() as u32;

        if !unsafe { Process32First(process_snapshot, &mut proc_entry as *mut PROCESSENTRY32) }
            .is_ok()
        {
            unsafe {
                match CloseHandle(process_snapshot) {
                    Ok(_) => {}
                    Err(e) => {
                        l::error!("Failed to close process snapshot handle: {}", e);
                    }
                }
                return false;
            }
        }
        loop {
            let proc_name =
                unsafe { std::ffi::CStr::from_ptr(proc_entry.szExeFile.as_ptr() as *mut i8) }
                    .to_string_lossy()
                    .into_owned();

            if proc_name == taskname {
                unsafe {
                    let proc = match OpenProcess(PROCESS_TERMINATE, false, proc_entry.th32ProcessID)
                    {
                        Ok(handle) => handle,
                        Err(e) => {
                            l::error!("Failed to open process handle: {}", e);
                            match CloseHandle(process_snapshot) {
                                Ok(_) => {}
                                Err(e) => {
                                    l::error!("Failed to close process snapshot handle: {}", e);
                                }
                            }
                            return false;
                        }
                    };
                    match TerminateProcess(proc, 0) {
                        Ok(_) => return true,
                        Err(e) => {
                            l::error!("Failed to terminate process: {}", e);
                            match CloseHandle(process_snapshot) {
                                Ok(_) => {}
                                Err(e) => {
                                    l::error!("Failed to close process snapshot handle: {}", e);
                                }
                            }
                            return false;
                        }
                    }
                }
            }

            if !unsafe { Process32Next(process_snapshot, &mut proc_entry as *mut PROCESSENTRY32) }
                .is_ok()
            {
                break;
            }
        }
        false
    }
}

#[inline(always)]
pub fn env(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}
