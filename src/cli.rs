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
    /// Apply a suggestion as an alias
    Apply {
        /// Suggestion ID to apply
        id: i64,
    },
    /// List all suggestions
    List {
        /// Show only unapplied suggestions
        #[arg(long)]
        unapplied: bool,
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
