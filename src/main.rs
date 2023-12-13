#![feature(async_closure)]

mod cli;
mod discord;
mod moonlight;
mod platform;

use clap::Parser;
use env_logger::Env;
use log as l;

use crate::cli::{actions, Subcommand};
pub use platform::Platform;

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let stopwatch = std::time::Instant::now();

    let args = cli::Args::parse();

    l::info!("Moon v{}", env!("CARGO_PKG_VERSION"));
    match args.subcommand {
        Subcommand::Up(up_args) => {
            actions::up(up_args).await;
        }
        Subcommand::Down(down_args) => {
            actions::down(down_args).await;
        }
        Subcommand::Dev(dev_args) => {
            actions::dev(dev_args).await;
        }
        Subcommand::Openasar(openasar_args) => {
            actions::openasar(openasar_args).await;
        }
    }

    l::info!("Finished in {}ms", stopwatch.elapsed().as_millis());
}
