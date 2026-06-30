use std::collections::HashMap;
use crate::history::HistoryEntry;

/// Analysis results from shell history.
#[derive(Debug, serde::Serialize)]
pub struct Analysis {
    pub total_commands: usize,
    pub unique_commands: usize,
    pub top_commands: Vec<(String, usize)>,
    pub frequent_sequences: Vec<Vec<String>>,
    pub anti_patterns: Vec<AntiPattern>,
    pub shell_breakdown: HashMap<String, usize>,
}

/// Detected anti-pattern.
#[derive(Debug, serde::Serialize)]
pub struct AntiPattern {
    pub pattern: String,
    pub command: String,
    pub frequency: usize,
    pub suggestion: String,
}

impl std::fmt::Display for Analysis {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "📊 Terminal Usage Stats")?;
        writeln!(f, "  Commands: {} total, {} unique", self.total_commands, self.unique_commands)?;
        writeln!(f, "")?;
        writeln!(f, "  Top Commands:")?;
        for (cmd, count) in &self.top_commands[..std::cmp::min(10, self.top_commands.len())] {
            writeln!(f, "    {:<30} {}x", cmd, count)?;
        }
        if !self.anti_patterns.is_empty() {
            writeln!(f, "")?;
            writeln!(f, "  ⚠️  Anti-patterns detected:")?;
            for ap in &self.anti_patterns {
                writeln!(f, "    • {}: {}", ap.pattern, ap.suggestion)?;
            }
        }
        Ok(())
    }
}

/// Analyze shell history entries.
pub fn analyze(entries: &[HistoryEntry]) -> Analysis {
    let total_commands = entries.len();

    // Count command frequency
    let mut freq: HashMap<String, usize> = HashMap::new();
    for entry in entries {
        let base = entry.command.split_whitespace().next().unwrap_or(&entry.command).to_string();
        *freq.entry(base).or_insert(0) += 1;
    }

    let unique_commands = freq.len();

    let mut top: Vec<_> = freq.into_iter().collect();
    top.sort_by(|a, b| b.1.cmp(&a.1));

    // Detect frequent sequences (2-command patterns)
    let mut seq_freq: HashMap<Vec<String>, usize> = HashMap::new();
    for window in entries.windows(2) {
        let seq = vec![window[0].command.clone(), window[1].command.clone()];
        *seq_freq.entry(seq).or_insert(0) += 1;
    }

    let mut sequences: Vec<_> = seq_freq.into_iter()
        .filter(|(_, count)| *count >= 3)
        .collect();
    sequences.sort_by(|a, b| b.1.cmp(&a.1));
    let frequent_sequences: Vec<Vec<String>> = sequences.into_iter()
        .take(5)
        .map(|(seq, _)| seq)
        .collect();

    // Detect anti-patterns
    let anti_patterns = detect_anti_patterns(entries);

    // Shell breakdown
    let mut shell_breakdown: HashMap<String, usize> = HashMap::new();
    for entry in entries {
        *shell_breakdown.entry(entry.shell.clone()).or_insert(0) += 1;
    }

    Analysis {
        total_commands,
        unique_commands,
        top_commands: top,
        frequent_sequences,
        anti_patterns,
        shell_breakdown,
    }
}

fn detect_anti_patterns(entries: &[HistoryEntry]) -> Vec<AntiPattern> {
    let mut patterns = Vec::new();

    // Count grep usage
    let grep_count = entries.iter()
        .filter(|e| e.command.contains("grep") && !e.command.contains("rg "))
        .count();
    if grep_count > 5 {
        patterns.push(AntiPattern {
            pattern: "Using grep instead of ripgrep".into(),
            command: "grep".into(),
            frequency: grep_count,
            suggestion: "Consider using `rg` (ripgrep) — it's 5-10x faster and has better defaults".into(),
        });
    }

    // Count find usage
    let find_count = entries.iter()
        .filter(|e| e.command.starts_with("find "))
        .count();
    if find_count > 3 {
        patterns.push(AntiPattern {
            pattern: "Using find instead of fd".into(),
            command: "find".into(),
            frequency: find_count,
            suggestion: "Consider using `fd` — it's faster and has saner defaults".into(),
        });
    }

    // Count cat on single file
    let cat_count = entries.iter()
        .filter(|e| {
            let parts: Vec<&str> = e.command.split_whitespace().collect();
            parts.len() == 2 && parts[0] == "cat"
        })
        .count();
    if cat_count > 5 {
        patterns.push(AntiPattern {
            pattern: "Using cat on single files".into(),
            command: "cat <file>".into(),
            frequency: cat_count,
            suggestion: "Consider using `bat` or just `< file` for syntax highlighting".into(),
        });
    }

    patterns
}
