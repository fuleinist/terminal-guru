use std::path::PathBuf;
use std::fs;

/// A single shell history entry.
#[derive(Debug, Clone, serde::Serialize)]
pub struct HistoryEntry {
    pub command: String,
    pub shell: String,
    pub timestamp: Option<i64>,
}

/// Read history from all supported shells.
pub fn read_all() -> Result<Vec<HistoryEntry>, Box<dyn std::error::Error>> {
    let mut entries = Vec::new();

    if let Ok(home) = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")) {
        let home_path = PathBuf::from(&home);

        // Zsh
        let zsh_history = home_path.join(".zsh_history");
        if zsh_history.exists() {
            if let Ok(e) = read_zsh(&zsh_history) {
                entries.extend(e);
            }
        }

        // Bash
        let bash_history = home_path.join(".bash_history");
        if bash_history.exists() {
            if let Ok(e) = read_bash(&bash_history) {
                entries.extend(e);
            }
        }

        // PowerShell
        if let Ok(ps_path) = std::env::var("APPDATA") {
            let ps_history = PathBuf::from(&ps_path)
                .join("Microsoft")
                .join("Windows")
                .join("PowerShell")
                .join("PSReadLine")
                .join("ConsoleHost_history.txt");
            if ps_history.exists() {
                if let Ok(e) = read_powershell(&ps_history) {
                    entries.extend(e);
                }
            }
        }

        // Fish
        let fish_dir = home_path.join(".local").join("share").join("fish");
        if fish_dir.exists() {
            if let Ok(entries2) = read_fish_dir(&fish_dir) {
                entries.extend(entries2);
            }
        }
    }

    // Deduplicate consecutive identical commands
    entries.dedup_by(|a, b| a.command == b.command);

    Ok(entries)
}

/// Read a zsh history file.
///
/// Supports both extended (`: <timestamp>:<elapsed>;<command>` — `EXTENDED_HISTORY`)
/// and plain (`<command>` — the default) formats. Plain entries were previously
/// dropped because the parser unconditionally looked for the `;` separator.
pub fn read_zsh(path: &PathBuf) -> Result<Vec<HistoryEntry>, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let mut entries = Vec::new();

    for line in content.lines() {
        // Zsh extended format: `: <timestamp>:<elapsed>;<command>`
        // Zsh plain format (default; `EXTENDED_HISTORY` unset): just `<command>`.
        // Extended entries start with ':'; everything after the first ';' is the command.
        let cmd = if let Some(rest) = line.strip_prefix(':') {
            match rest.find(';') {
                Some(idx) => rest[idx + 1..].trim(),
                None => continue,
            }
        } else {
            line.trim()
        };
        if !cmd.is_empty() {
            entries.push(HistoryEntry {
                command: cmd.to_string(),
                shell: "zsh".into(),
                timestamp: None,
            });
        }
    }

    Ok(entries)
}

fn read_bash(path: &PathBuf) -> Result<Vec<HistoryEntry>, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let entries = content
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| HistoryEntry {
            command: l.trim().to_string(),
            shell: "bash".into(),
            timestamp: None,
        })
        .collect();
    Ok(entries)
}

fn read_powershell(path: &PathBuf) -> Result<Vec<HistoryEntry>, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let entries = content
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| HistoryEntry {
            command: l.trim().to_string(),
            shell: "powershell".into(),
            timestamp: None,
        })
        .collect();
    Ok(entries)
}

fn read_fish_dir(dir: &PathBuf) -> Result<Vec<HistoryEntry>, Box<dyn std::error::Error>> {
    let mut entries = Vec::new();
    if let Ok(read_dir) = fs::read_dir(dir) {
        for entry in read_dir.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("fish") {
                if let Ok(content) = fs::read_to_string(&path) {
                    for line in content.lines() {
                        let cmd = line.trim().trim_start_matches("- cmd: ");
                        if !cmd.is_empty() {
                            entries.push(HistoryEntry {
                                command: cmd.to_string(),
                                shell: "fish".into(),
                                timestamp: None,
                            });
                        }
                    }
                }
            }
        }
    }
    Ok(entries)
}
