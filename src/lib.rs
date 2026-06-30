pub mod cli;
pub mod history;
pub mod analyzer;
pub mod suggest;
pub mod db;

use clap::Parser;
use cli::Cli;

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        cli::Command::Suggest { json } => {
            let history = history::read_all()?;
            let analysis = analyzer::analyze(&history);
            let suggestions = suggest::generate(&analysis);
            if json {
                println!("{}", serde_json::to_string_pretty(&suggestions)?);
            } else {
                for s in &suggestions {
                    println!("{}", s);
                }
                if suggestions.is_empty() {
                    println!("No suggestions found. Keep using the terminal and check back!");
                }
            }
        }
        cli::Command::Stats { json } => {
            let history = history::read_all()?;
            let analysis = analyzer::analyze(&history);
            if json {
                println!("{}", serde_json::to_string_pretty(&analysis)?);
            } else {
                println!("{}", analysis);
            }
        }
        cli::Command::Daemon { action: _ } => {
            eprintln!("Daemon mode not yet implemented. Use `tguru suggest` for now.");
        }
    }

    Ok(())
}
