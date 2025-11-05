use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author, version, about = "Saros DLMM Interface CLI")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Start the interface with optional web dashboard
    Start {
        /// Enable web interface
        #[arg(long)]
        web: bool,
    },
}