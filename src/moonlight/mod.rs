mod channel;
mod download;

use std::path::PathBuf;

pub use channel::Channel;
pub use download::{download};

pub async fn init_moonlight(at: PathBuf, channel: Option<Channel>, repo_location: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    let channel = channel.unwrap_or({
        if at.join("branch.txt").exists() {
            match std::fs::read_to_string(at.join("branch.txt"))?.as_str() {
                "stable" => Channel::Stable,
                "nightly" => Channel::Nightly,
                "git" => Channel::Git,
                _ => Channel::Stable
            }
        } else if at.join("dist").join("branch.txt").exists() {
            match std::fs::read_to_string(at.join("dist").join("branch.txt"))?.as_str() {
                "stable" => Channel::Stable,
                "nightly" => Channel::Nightly,
                "git" => Channel::Git,
                _ => Channel::Stable
            }
        } else {
            Channel::Stable
        }
    });

    download(channel, at, repo_location, true).await?;

    Ok(())
}