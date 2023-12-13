use std::path::PathBuf;

use crate::{cli::args::up::Args, discord::DiscordKind, moonlight::{Channel, init_moonlight}, platform::{Platform, env}};


use log as l;
use tokio::{sync::mpsc, spawn};
#[inline(always)]
pub async fn up(args: Args) {
    let kind = DiscordKind::from(args.branch.clone());
    let plat_installs =crate::Platform::installs_by_kind(kind);
    let install = match plat_installs.first() {
        Some(install) => install,
        None => {
            l::error!("No Discord install found for branch {:?}", kind);
            return;
        }
    };
    let channel = Channel::from(args.channel.clone());
    let root = env("MOONLIGHT_ROOT", &Platform::conf_dir());

    let (tx, mut rx) = mpsc::channel::<bool>(1);
    l::info!("Install settings:");
    l::info!("  Using root directory {}", root);
    l::info!("  Using Discord install at {:?}", install.path);
    l::info!("  Using Moonlight channel {:?}", channel);
    l::info!("Making sure moonlight is up to date...");
    let root = PathBuf::from(root);
    let rt = root.clone();
    let ct = channel.clone();
    let t = spawn((async move || {
        let mut needs_revert = false;
        if let Err(e) = init_moonlight(rt, Some(ct), None).await {
            l::error!("Failed to update moonlight: {}", e);
            needs_revert = true;
        };
        let _ = tx.send(needs_revert).await;
        
    })());
    l::info!("Killing discord...");
    loop {
        match install.kill().await {
            Ok(_) => continue,
            Err(_) => {
                break;
            }
        }
    }

    l::info!("Injecting...");
    if let Err(e) = install.inject(&root).await {
        l::error!("Failed to inject: {}", e);
        return;
    };
    let _ = t.await;
    match rx.recv().await {
        Some(true) => {
            l::error!("Failed to update moonlight, uninjecting...");
            if let Err(e) = install.uninject().await {
                l::error!("Failed to uninject: {}", e);
                l::error!("Please manually uninject moonlight from Discord");
                return;
            };
            return;
        },
        Some(false) => {
            
        },
        _ => {
            l::warn!("no message in mpsc channel, assuming success")
        }
    }
    l::info!("Done!");
}