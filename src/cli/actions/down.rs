use crate::{cli::args::down::Args, discord::DiscordKind};

use log as l;

#[inline(always)]
pub async fn down(args: Args) {
    let kind = DiscordKind::from(args.branch.clone());
    let plat_installs =crate::Platform::installs_by_kind(kind);
    let install = match plat_installs.first() {
        Some(install) => install,
        None => {
            l::error!("No Discord install found for branch {:?}", kind);
            return;
        }
    };
    l::info!("Killing discord...");
    loop {
        match install.kill().await {
            Ok(_) => continue,
            Err(_) => {
                break;
            }
        }
    }
    l::info!("Uninjecting...");
    if let Err(e) = install.uninject().await {
        l::error!("Failed to uninject: {}", e);
        return;
    };
    l::info!("Done!");
}