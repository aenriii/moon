mod kind;
mod install;
mod injected;

#[cfg(target_os = "linux")]
mod flatpak;

pub use kind::DiscordKind;
pub use install::DiscordInstall;

#[cfg(target_os = "linux")]
pub use flatpak::Flatpak;
