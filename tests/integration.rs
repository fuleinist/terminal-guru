use terminal_guru::analyzer;
use terminal_guru::shell;
use terminal_guru::shell::Shell;
use terminal_guru::suggest;
use terminal_guru::db::Database;

#[test]
fn test_analyzer_empty() {
    let analysis = analyzer::analyze(&[]);
    assert_eq!(analysis.total_commands, 0);
    assert_eq!(analysis.unique_commands, 0);
    assert!(analysis.top_commands.is_empty());
}

#[test]
fn test_analyzer_counts() {
    use terminal_guru::history::HistoryEntry;
    let entries = vec![
        HistoryEntry { command: "ls".into(), shell: "bash".into(), timestamp: None },
        HistoryEntry { command: "ls".into(), shell: "bash".into(), timestamp: None },
        HistoryEntry { command: "cd".into(), shell: "bash".into(), timestamp: None },
        HistoryEntry { command: "git status".into(), shell: "bash".into(), timestamp: None },
        HistoryEntry { command: "git status".into(), shell: "bash".into(), timestamp: None },
        HistoryEntry { command: "git status".into(), shell: "bash".into(), timestamp: None },
    ];

    let analysis = analyzer::analyze(&entries);
    assert_eq!(analysis.total_commands, 6);
    assert_eq!(analysis.unique_commands, 3);
    assert_eq!(analysis.top_commands[0].0, "git status");
    assert_eq!(analysis.top_commands[0].1, 3);
}

#[test]
fn test_suggest_aliases() {
    use terminal_guru::history::HistoryEntry;
    let entries = vec![
        HistoryEntry { command: "git status".into(), shell: "bash".into(), timestamp: None },
        HistoryEntry { command: "git status".into(), shell: "bash".into(), timestamp: None },
        HistoryEntry { command: "git status".into(), shell: "bash".into(), timestamp: None },
        HistoryEntry { command: "git status".into(), shell: "bash".into(), timestamp: None },
        HistoryEntry { command: "git status".into(), shell: "bash".into(), timestamp: None },
        HistoryEntry { command: "docker compose up".into(), shell: "bash".into(), timestamp: None },
        HistoryEntry { command: "docker compose up".into(), shell: "bash".into(), timestamp: None },
        HistoryEntry { command: "docker compose up".into(), shell: "bash".into(), timestamp: None },
        HistoryEntry { command: "docker compose up".into(), shell: "bash".into(), timestamp: None },
        HistoryEntry { command: "docker compose up".into(), shell: "bash".into(), timestamp: None },
    ];

    let analysis = analyzer::analyze(&entries);
    let suggestions = suggest::generate(&analysis);

    let alias_suggestions: Vec<_> = suggestions.iter().filter(|s| s.kind == "alias").collect();
    assert!(!alias_suggestions.is_empty(), "Should suggest aliases for frequent commands");

    let gst = alias_suggestions.iter().find(|s| s.alias.as_deref() == Some("gst"));
    assert!(gst.is_some(), "Should suggest gst for git status");

    let dcup = alias_suggestions.iter().find(|s| s.alias.as_deref() == Some("dcup"));
    assert!(dcup.is_some(), "Should suggest dcup for docker compose up");
}

#[test]
fn test_anti_pattern_detection() {
    use terminal_guru::history::HistoryEntry;
    let mut entries = Vec::new();
    for _ in 0..10 {
        entries.push(HistoryEntry {
            command: "grep something file.txt".into(),
            shell: "bash".into(),
            timestamp: None,
        });
    }

    let analysis = analyzer::analyze(&entries);
    let has_grep_antipattern = analysis.anti_patterns.iter()
        .any(|ap| ap.pattern.contains("grep"));
    assert!(has_grep_antipattern, "Should detect grep anti-pattern");
}

#[test]
fn test_suggest_empty() {
    let analysis = analyzer::analyze(&[]);
    let suggestions = suggest::generate(&analysis);
    assert!(suggestions.is_empty());
}

#[test]
fn test_db_open() {
    let db = Database::open_in_memory();
    assert!(db.is_ok(), "Database should open without error");
}

#[test]
fn test_db_record_and_list() {
    let db = Database::open_in_memory().unwrap();

    db.record_suggestion("alias", Some("gst"), "git status", 10, "Test suggestion").unwrap();
    let suggestions = db.list_suggestions(false).unwrap();
    assert!(!suggestions.is_empty());
    assert_eq!(suggestions[0].kind, "alias");
    assert_eq!(suggestions[0].alias.as_deref(), Some("gst"));
}

#[test]
fn test_db_apply() {
    let db = Database::open_in_memory().unwrap();

    db.record_suggestion("alias", Some("gst"), "git status", 10, "Test").unwrap();
    let suggestions = db.list_suggestions(false).unwrap();
    let id = suggestions[0].id;

    db.apply_suggestion(id).unwrap();
    let updated = db.list_suggestions(false).unwrap();
    assert_eq!(updated[0].applied, 1);
}

#[test]
fn test_daemon_state() {
    let db = Database::open_in_memory().unwrap();

    db.set_daemon_state("test_key", "test_value").unwrap();
    let val = db.get_daemon_state("test_key").unwrap();
    assert_eq!(val, Some("test_value".into()));

    let missing = db.get_daemon_state("nonexistent").unwrap();
    assert_eq!(missing, None);
}

#[test]
fn test_get_suggestion() {
    let db = Database::open_in_memory().unwrap();
    db.record_suggestion("alias", Some("gst"), "git status", 10, "Test").unwrap();

    let fetched = db.get_suggestion(1).unwrap().expect("should find suggestion");
    assert_eq!(fetched.id, 1);
    assert_eq!(fetched.alias.as_deref(), Some("gst"));
    assert_eq!(fetched.command, "git status");

    let missing = db.get_suggestion(999).unwrap();
    assert!(missing.is_none());
}

#[test]
fn test_shell_module_renders_per_shell() {
    assert!(Shell::Bash.render_alias("g", "git").contains("alias g='git'"));
    assert!(Shell::Zsh.render_alias("g", "git").contains("alias g='git'"));
    assert!(Shell::Fish.render_alias("g", "git").contains("alias g 'git'"));
    assert!(Shell::PowerShell.render_alias("g", "git").contains("Set-Alias"));
}

#[test]
fn test_shell_detect_from_env() {
    let original = std::env::var("SHELL").ok();
    std::env::set_var("SHELL", "/bin/zsh");
    assert_eq!(shell::detect_shell(), Shell::Zsh);
    std::env::set_var("SHELL", "/usr/bin/bash");
    assert_eq!(shell::detect_shell(), Shell::Bash);
    if let Some(v) = original {
        std::env::set_var("SHELL", v);
    } else {
        std::env::remove_var("SHELL");
    }
}

// -------- Zsh history reader tests ----------------------------------------
//
// Regression coverage for the bug where plain-format zsh entries (the
// default — `EXTENDED_HISTORY` is unset in most installs) were silently
// dropped because the parser unconditionally required a ';' separator.

#[test]
fn test_read_zsh_extended_format() {
    use terminal_guru::history;
    use std::io::Write;

    let dir = std::env::temp_dir().join(format!("tguru_zsh_test_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(".zsh_history");
    let mut f = std::fs::File::create(&path).unwrap();
    // Extended format: `: <timestamp>:<elapsed>;<command>`
    writeln!(f, ": 1687999999:0;ls -la").unwrap();
    writeln!(f, ": 1688000000:0;git status").unwrap();
    drop(f);

    let entries = history::read_zsh(&path).unwrap();
    let cmds: Vec<&str> = entries.iter().map(|e| e.command.as_str()).collect();
    assert_eq!(cmds, vec!["ls -la", "git status"]);

    std::fs::remove_file(&path).ok();
}

#[test]
fn test_read_zsh_plain_format() {
    use terminal_guru::history;
    use std::io::Write;

    let dir = std::env::temp_dir().join(format!("tguru_zsh_plain_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(".zsh_history");
    let mut f = std::fs::File::create(&path).unwrap();
    // Plain format (no `: TS:ELAPSED;` prefix) — the zsh default.
    writeln!(f, "ls -la").unwrap();
    writeln!(f, "git status").unwrap();
    writeln!(f, "echo hello").unwrap();
    drop(f);

    let entries = history::read_zsh(&path).unwrap();
    let cmds: Vec<&str> = entries.iter().map(|e| e.command.as_str()).collect();
    assert_eq!(
        cmds,
        vec!["ls -la", "git status", "echo hello"],
        "Plain zsh history entries must be parsed (the previous parser dropped them)"
    );

    std::fs::remove_file(&path).ok();
}

#[test]
fn test_read_zsh_mixed_format() {
    use terminal_guru::history;
    use std::io::Write;

    let dir = std::env::temp_dir().join(format!("tguru_zsh_mixed_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(".zsh_history");
    let mut f = std::fs::File::create(&path).unwrap();
    writeln!(f, ": 1687999999:0;ls -la").unwrap();
    writeln!(f, "git status").unwrap();
    writeln!(f, ": 1688000001:0;echo hello").unwrap();
    writeln!(f, "cargo test").unwrap();
    drop(f);

    let entries = history::read_zsh(&path).unwrap();
    let cmds: Vec<&str> = entries.iter().map(|e| e.command.as_str()).collect();
    assert_eq!(
        cmds,
        vec!["ls -la", "git status", "echo hello", "cargo test"],
        "Both extended and plain entries must parse in the same file"
    );

    std::fs::remove_file(&path).ok();
}

#[test]
fn test_read_zsh_skips_blank_lines() {
    use terminal_guru::history;
    use std::io::Write;

    let dir = std::env::temp_dir().join(format!("tguru_zsh_blank_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(".zsh_history");
    let mut f = std::fs::File::create(&path).unwrap();
    writeln!(f, "").unwrap();
    writeln!(f, "   ").unwrap();
    writeln!(f, "ls").unwrap();
    writeln!(f, "").unwrap();
    writeln!(f, ": 1687999999:0;").unwrap();
    writeln!(f, ": 1688000000:0;git status").unwrap();
    drop(f);

    let entries = history::read_zsh(&path).unwrap();
    let cmds: Vec<&str> = entries.iter().map(|e| e.command.as_str()).collect();
    assert_eq!(cmds, vec!["ls", "git status"]);

    std::fs::remove_file(&path).ok();
}
