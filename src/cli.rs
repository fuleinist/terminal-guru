use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "tguru", about = "AI-powered terminal assistant")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Analyze shell history and suggest optimizations
    Suggest {
        /// Output in JSON format
        #[arg(long)]
        json: bool,
    },
    /// Show terminal usage statistics
    Stats {
        /// Output in JSON format
        #[arg(long)]
        json: bool,
    },
    /// Manage the background daemon
    Daemon {
        #[command(subcommand)]
        action: DaemonAction,
    },
}

#[derive(Subcommand)]
pub enum DaemonAction {
    /// Start the daemon
    Start,
    /// Stop the daemon
    Stop,
    /// Show daemon status
    Status,
}
