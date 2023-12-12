use super::args;

#[derive(Debug, clap::Subcommand)]
pub enum Subcommand {
    Up(args::up::Args),
    Down(args::down::Args),
    Dev(args::dev::Args),
    Openasar(args::openasar::Args),
}