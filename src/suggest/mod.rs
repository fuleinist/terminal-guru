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
                    frequency: 0,
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
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    let base = parts.first()?;

    let alias = match *base {
        "docker" => {
            if parts.len() < 3 { return None; }
            let sub = parts[1];
            let action = parts[2];
            match (sub, action) {
                ("compose", "up") => Some("dcup"),
                ("compose", "down") => Some("dcdown"),
                ("compose", "logs") => Some("dclogs"),
                ("compose", "ps") => Some("dcps"),
                ("ps", _) => Some("dps"),
                ("images", _) => Some("dimg"),
                _ => None,
            }
        }
        "git" => {
            let sub = parts.get(1)?;
            match *sub {
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

    alias.map(|a| a.to_string())
}
