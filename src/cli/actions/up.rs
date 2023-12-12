use crate::{cli::args::up::Args, discord::DiscordKind};

use log as l;
#[inline(always)]
pub async fn up(args: Args) {
    let kind = DiscordKind::from(args.branch.clone());
    let install = match crate::Platform::installs_by_kind(kind).first() {
        Some(install) => install,
        None => {
            l::error!("No install found for branch {}", &args.branch);
            return;
        }
    };
    
}