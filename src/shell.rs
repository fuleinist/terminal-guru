use std::fs;
use std::path::{Path, PathBuf};

/// A detected user shell.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Shell {
    Zsh,
    Bash,
    Fish,
    PowerShell,
    Unknown,
}

impl Shell {
    /// Path to the user's rc file for this shell.
    pub fn rc_path(&self) -> Result<PathBuf, String> {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .map_err(|_| "Could not determine home directory".to_string())?;
        let home = PathBuf::from(home);
        Ok(match self {
            Shell::Zsh => home.join(".zshrc"),
            Shell::Bash => home.join(".bashrc"),
            Shell::Fish => home.join(".config").join("fish").join("config.fish"),
            Shell::PowerShell => PathBuf::from(
                std::env::var("PROFILE")
                    .unwrap_or_else(|_| {
                        home.join("Documents")
                            .join("PowerShell")
                            .join("Microsoft.PowerShell_profile.ps1")
                            .to_string_lossy()
                            .into_owned()
                    }),
            ),
            Shell::Unknown => return Err("Unknown shell — cannot determine rc file".into()),
        })
    }

    /// Render an alias line in this shell's syntax.
    pub fn render_alias(&self, alias: &str, command: &str) -> String {
        match self {
            Shell::Fish => format!("alias {} '{}'", alias, command.replace('\'', "\\'")),
            Shell::PowerShell => format!("Set-Alias -Name {} -Value '{}'", alias, command),
            _ => format!("alias {}='{}'", alias, command),
        }
    }
}

/// Detect the user's shell from $SHELL (POSIX) or PSVersionTable / $PROFILE (Windows).
pub fn detect_shell() -> Shell {
    if let Ok(shell_path) = std::env::var("SHELL") {
        let lower = shell_path.to_lowercase();
        if lower.contains("zsh") {
            return Shell::Zsh;
        }
        if lower.contains("fish") {
            return Shell::Fish;
        }
        if lower.contains("bash") {
            return Shell::Bash;
        }
    }
    if cfg!(windows) {
        return Shell::PowerShell;
    }
    Shell::Unknown
}

/// Write an alias to the shell rc file. Returns the path written to.
/// If the alias already exists in the file (by name), the file is left unchanged.
pub fn write_alias(alias: &str, command: &str) -> Result<PathBuf, String> {
    let shell = detect_shell();
    let path = shell.rc_path()?;
    append_alias(&path, &shell, alias, command)?;
    Ok(path)
}

fn append_alias(path: &Path, shell: &Shell, alias: &str, command: &str) -> Result<(), String> {
    let line = shell.render_alias(alias, command);

    let existing = if path.exists() {
        fs::read_to_string(path).map_err(|e| format!("read {}: {}", path.display(), e))?
    } else {
        String::new()
    };

    // Don't add duplicates — check for an alias with this name on its own word boundary.
    let alias_prefix = match shell {
        Shell::Fish => format!("alias {} ", alias),
        Shell::PowerShell => "Set-Alias ".to_string(),
        _ => format!("alias {}=", alias),
    };
    let already_defined = existing.lines().any(|l| {
        let trimmed = l.trim_start();
        match shell {
            Shell::PowerShell => {
                let after = trimmed.strip_prefix("Set-Alias").unwrap_or("");
                after.split_whitespace().nth(1).map(|n| n == alias).unwrap_or(false)
                    || after.split_whitespace().next().map(|n| n == alias).unwrap_or(false)
            }
            _ => trimmed.starts_with(&alias_prefix),
        }
    });

    if already_defined {
        return Err(format!("alias `{}` already defined in {}", alias, path.display()));
    }

    use std::io::Write;
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| format!("open {}: {}", path.display(), e))?;
    if !existing.is_empty() && !existing.ends_with('\n') {
        writeln!(file).map_err(|e| e.to_string())?;
    }
    writeln!(file, "{}", line).map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_shell_from_env() {
        // Save and restore
        let original = std::env::var("SHELL").ok();
        std::env::set_var("SHELL", "/bin/zsh");
        assert_eq!(detect_shell(), Shell::Zsh);
        std::env::set_var("SHELL", "/usr/local/bin/fish");
        assert_eq!(detect_shell(), Shell::Fish);
        std::env::set_var("SHELL", "/bin/bash");
        assert_eq!(detect_shell(), Shell::Bash);
        if let Some(v) = original {
            std::env::set_var("SHELL", v);
        } else {
            std::env::remove_var("SHELL");
        }
    }

    #[test]
    fn test_render_alias() {
        assert_eq!(Shell::Bash.render_alias("gst", "git status"), "alias gst='git status'");
        assert_eq!(Shell::Zsh.render_alias("gst", "git status"), "alias gst='git status'");
        assert_eq!(Shell::Fish.render_alias("gst", "git status"), "alias gst 'git status'");
        assert_eq!(
            Shell::PowerShell.render_alias("gst", "git status"),
            "Set-Alias -Name gst -Value 'git status'"
        );
    }

    #[test]
    fn test_render_alias_escapes_single_quote() {
        assert_eq!(
            Shell::Fish.render_alias("g", "echo 'hi'"),
            "alias g 'echo \\'hi\\''"
        );
    }

    #[test]
    fn test_append_alias_to_new_file() {
        let dir = std::env::temp_dir().join(format!("tg-test-new-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join(".bashrc");

        append_alias(&path, &Shell::Bash, "gst", "git status").unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("alias gst='git status'"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_append_alias_to_existing_file_preserves_content() {
        let dir = std::env::temp_dir().join(format!("tg-test-existing-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join(".bashrc");
        std::fs::write(&path, "# existing content\nexport PATH=$PATH:/opt/bin\n").unwrap();

        append_alias(&path, &Shell::Bash, "gst", "git status").unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("# existing content"));
        assert!(content.contains("alias gst='git status'"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_append_alias_skips_duplicates() {
        let dir = std::env::temp_dir().join(format!("tg-test-dup-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join(".bashrc");
        std::fs::write(&path, "alias gst='git status'\n").unwrap();

        let result = append_alias(&path, &Shell::Bash, "gst", "git status");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("already defined"));

        let _ = std::fs::remove_dir_all(&dir);
    }
}