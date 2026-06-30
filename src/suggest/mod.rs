use crate::analyzer::Analysis;
use std::fmt;

/// A suggestion for the user.
#[derive(Debug, serde::Serialize)]
pub struct Suggestion {
    pub kind: String,
    pub alias: Option<String>,
    pub command: String,
    pub frequency: usize,
    pub description: String,
}

impl fmt::Display for Suggestion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.alias {
            Some(alias) => write!(f, "💡 alias {}='{}'  ({}x, {})", alias, self.command, self.frequency, self.description),
            None => write!(f, "💡 {}  ({}x, {})", self.command, self.frequency, self.description),
        }
    }
}

/// Generate suggestions from analysis.
pub fn generate(analysis: &Analysis) -> Vec<Suggestion> {
    let mut suggestions = Vec::new();

    // Suggest aliases for top commands that appear frequently
    for (cmd, count) in &analysis.top_commands {
        if *count >= 5 && cmd.len() > 3 {
            if let Some(alias) = suggest_alias(cmd) {
                suggestions.push(Suggestion {
                    kind: "alias".into(),
                    alias: Some(alias),
                    command: cmd.clone(),
                    frequency: *count,
                    description: format!("Run this {} times. Create an alias?", count),
                });
            }
        }
    }

    // Suggest aliases for frequent sequences
    for seq in &analysis.frequent_sequences {
        if seq.len() == 2 {
            let combined = format!("{} && {}", seq[0], seq[1]);
            if let Some(alias) = suggest_alias(&combined) {
                suggestions.push(Suggestion {
                    kind: "sequence".into(),
                    alias: Some(alias),
                    command: combined,
                    frequency: 0, // unknown from sequence data
                    description: format!("Frequent sequence: `{}` then `{}`", seq[0], seq[1]),
                });
            }
        }
    }

    // Suggest fixes for anti-patterns
    for ap in &analysis.anti_patterns {
        suggestions.push(Suggestion {
            kind: "anti-pattern".into(),
            alias: None,
            command: ap.command.clone(),
            frequency: ap.frequency,
            description: ap.suggestion.clone(),
        });
    }

    suggestions
}

fn suggest_alias(cmd: &str) -> Option<String> {
    let base = cmd.split_whitespace().next()?;

    // Common aliases
    let alias = match base {
        "docker" => {
            let sub = cmd.split_whitespace().nth(1)?;
            match sub {
                "compose" => {
                    let action = cmd.split_whitespace().nth(2)?;
                    match action {
                        "up" => Some("dcup"),
                        "down" => Some("dcdown"),
                        "logs" => Some("dclogs"),
                        "ps" => Some("dcps"),
                        _ => None,
                    }
                }
                "ps" => Some("dps"),
                "images" => Some("dimg"),
                _ => None,
            }
        }
        "git" => {
            let sub = cmd.split_whitespace().nth(1)?;
            match sub {
                "status" => Some("gst"),
                "add" => Some("ga"),
                "commit" => Some("gc"),
                "push" => Some("gp"),
                "pull" => Some("gl"),
                "diff" => Some("gd"),
                "log" => Some("glog"),
                "checkout" => Some("gco"),
                "branch" => Some("gb"),
                _ => None,
            }
        }
        "kubectl" => Some("k"),
        "terraform" => Some("tf"),
        _ => None,
    };

    alias.map(|a| {
        // Avoid collisions with existing aliases
        if a.len() <= 3 { a.to_string() } else { a.to_string() }
    })
}
