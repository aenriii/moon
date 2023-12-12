use std::path::PathBuf;
use std::fs;
use log as l;
use octocrab::models::repos::Object;
use octocrab::params::repos::Reference;

use super::Channel;



pub async fn download(channel: Channel, path: PathBuf, repo_location: Option<String>, check_ver: bool) -> Result<bool, Box<dyn std::error::Error>> { // returns <needs_build, err>

    let stopwatch = std::time::Instant::now();

    if !path.join("dist").exists() {
        fs::create_dir_all(&path.join("dist"))?;
    }
    match channel {
        Channel::Stable => {
            let release = octocrab::instance().repos("moonlight-mod", "moonlight").releases().get_latest().await?;

            if check_ver {
                if path.join("dist/version.txt").exists() {
                    if fs::read_to_string(path.join("dist/version.txt"))?.contains(release.tag_name.as_str()) {
                        l::info!("Moonlight is up to date");
                        return Ok(false);
                    }
                }
            }

            let tarball = reqwest::get(release.assets.iter().find(|a| a.name == "dist.tar.gz").unwrap().browser_download_url.to_string()).await?.bytes().await?;
            let stable_ref = release.tag_name;
            tar::Archive::new(flate2::read::GzDecoder::new(std::io::Cursor::new(tarball))).unpack(&path.join("dist"))?; 

            fs::write(path.join("dist/version.txt"), stable_ref)?;
            fs::write(path.join("dist/branch.txt"), "stable")?;
            
            l::info!(
                "Downloaded moonlight (channel: {:?}) in {}ms",
                channel, stopwatch.elapsed().as_millis()
            );
            Ok(false)
        },
        Channel::Nightly => {
            let nightly_ref = reqwest::get("https://moonlight-mod.github.io/moonlight/ref").await?.text().await?.split("\n").next().unwrap().to_owned();
            if check_ver {
                if path.join("dist/version.txt").exists() {
                    if fs::read_to_string(path.join("dist/version.txt"))? == nightly_ref {
                        l::info!("Moonlight is up to date");
                        return Ok(false);
                    }
                }
            }
            let tarball = reqwest::get("https://moonlight-mod.github.io/moonlight/dist.tar.gz").await?.bytes().await?;
            tar::Archive::new(flate2::read::GzDecoder::new(std::io::Cursor::new(tarball))).unpack(&path.join("dist"))?;

            fs::write(path.join("dist/version.txt"), nightly_ref)?;
            fs::write(path.join("dist/branch.txt"), "nightly")?;

            l::info!(
                "Downloaded moonlight (channel: {:?}) in {}ms",
                channel, stopwatch.elapsed().as_millis()
            );
            Ok(false)
        },
        Channel::Git => {
            {
                if check_ver && path.join(".git").exists() {
                    let reference = git2::Repository::open(path.join(".git"))?.head()?.target().unwrap();
                    if path.join("dist/version.txt").exists() {
                        if fs::read_to_string(path.join("dist/version.txt"))? == reference.to_string() {
                            l::info!("Moonlight is up to date");
                            return Ok(false);
                        }
                    }
                }
                if path.exists() {
                    fs::remove_dir_all(&path)?;
                }
            };
            git2::Repository::clone(&repo_location.unwrap_or("https://github.com/moonlight-mod/moonlight".to_string()), &path)?;


            fs::write(path.join("dist/branch.txt"), "git")?;
            fs::write(path.join("dist/version.txt"), get_ref(Channel::Git).await)?; // TODO: make this not use await (it's not async, but it's in an async function
            l::info!(
                "Downloaded moonlight (channel: {:?}) in {}ms",
                channel, stopwatch.elapsed().as_millis()
            );
            Ok(true)
        }
    }


}

pub async fn get_ref(channel: Channel) -> String {
    match channel {
        Channel::Stable => {
            octocrab::instance()
                .repos("moonlight-mod", "moonlight")
                .releases().get_latest().await.unwrap().tag_name
        },
        Channel::Nightly => {
            reqwest::get("https://moonlight-mod.github.io/moonlight/ref").await
            .unwrap().text().await
            .unwrap().split("\n").next()
            .unwrap().to_owned()
        },
        Channel::Git => {
            match octocrab::instance()
                .repos("moonlight-mod", "moonlight")
                .get_ref(&Reference::Branch("main".to_string())).await
                .unwrap().object {
                    Object::Commit { sha, .. } |
                    Object::Tag { sha, .. } => sha,
                    _ => panic!("Invalid object type")
                }
        }
    }
}