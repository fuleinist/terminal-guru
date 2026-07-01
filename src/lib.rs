pub mod cli;
pub mod history;
pub mod analyzer;
pub mod suggest;
pub mod db;
pub mod daemon;
pub mod shell;

use clap::Parser;
use cli::Cli;
use db::Database;

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let db = Database::open()?;

    match cli.command {
        cli::Command::Suggest { json } => {
            let history = history::read_all()?;
            let analysis = analyzer::analyze(&history);
            let suggestions = suggest::generate(&analysis);

            // Persist suggestions to DB
            for s in &suggestions {
                db.record_suggestion(
                    &s.kind,
                    s.alias.as_deref(),
                    &s.command,
                    s.frequency,
                    &s.description,
                )?;
            }

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
        cli::Command::Daemon { action } => {
            match action {
                cli::DaemonAction::Start => daemon::start(&db)?,
                cli::DaemonAction::Stop => daemon::stop(&db)?,
                cli::DaemonAction::Status => daemon::status(&db)?,
            }
        }
        cli::Command::Apply { id } => {
            let suggestion = db
                .get_suggestion(id)?
                .ok_or_else(|| format!("No suggestion with id {}", id))?;

            if let Some(alias) = &suggestion.alias {
                match shell::write_alias(alias, &suggestion.command) {
                    Ok(path) => println!(
                        "✓ Added to {}: alias {}='{}'",
                        path.display(),
                        alias,
                        suggestion.command
                    ),
                    Err(e) if e.contains("already defined") => {
                        println!("ℹ️  {}", e);
                    }
                    Err(e) => return Err(e.into()),
                }
            } else {
                println!("Suggestion {} has no alias to write.", id);
            }

            db.apply_suggestion(id)?;
        }
        cli::Command::List { unapplied } => {
            let suggestions = db.list_suggestions(unapplied)?;
            if suggestions.is_empty() {
                println!("No suggestions found. Run `tguru suggest` first.");
            } else {
                for s in &suggestions {
                    let status = if s.applied != 0 { "✅" } else { "  " };
                    match &s.alias {
                        Some(alias) => println!("{} [{}] alias {}='{}'  ({}x, {})",
                            status, s.id, alias, s.command, s.frequency, s.description),
                        None => println!("{} [{}] {}  ({}x, {})",
                            status, s.id, s.command, s.frequency, s.description),
                    }
                }
            }
        }
    }

    Ok(())
}
