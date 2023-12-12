#![feature(async_closure)]

mod platform;
mod cli;
mod discord;
mod moonlight;

use std::path::PathBuf;
use clap::Parser;
use log as l;

pub use platform::Platform; 
use crate::{discord::DiscordKind, moonlight::Channel, cli::{Subcommand, actions}};

#[tokio::main]
async fn main() {
    env_logger::init();
    let stopwatch = std::time::Instant::now();
    
    let args = cli::Args::parse();

    l::info!("Moon v{}", env!("CARGO_PKG_VERSION"));
    match args.subcommand {
        Subcommand::Up(up_args) => {
            actions::up(up_args).await;
        },
        Subcommand::Down(down_args) => {
            actions::down(down_args).await;
        },
        Subcommand::Dev(dev_args) => {
            actions::dev(dev_args).await;
        },
        Subcommand::Openasar(openasar_args) => {
            actions::openasar(openasar_args).await;
        },
    }

    l::info!(
        "Finished in {}ms",
        stopwatch.elapsed().as_millis()
    );

}
