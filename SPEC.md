# terminal-guru — SPEC v1

## Mission
AI-powered terminal assistant that watches shell history and proactively suggests optimizations, aliases, and automations.

## Why
Developers repeat the same terminal commands daily. Shell history is underutilized — it's a goldmine of patterns that could be automated but nobody mines it.

## Usage
```bash
# Suggest optimizations based on shell history
tguru suggest

# Interactive shell plugin (zsh/bash/fish)
tguru daemon start
tguru daemon status

# Show stats about your terminal usage
tguru stats

# Apply a suggestion as an alias
tguru apply <suggestion-id>
```

## Acceptance Criteria

### CLI (MVP)
- [x] `tguru suggest` — analyze shell history and suggest aliases/optimizations
- [x] `tguru stats` — show terminal usage statistics
- [x] `tguru daemon start|status|stop` — background daemon
- [x] `tguru apply <id>` — mark a suggestion as applied
- [x] `tguru list [--unapplied]` — list persisted suggestions
- [x] SQLite persistence for suggestions and daemon state
- [x] Reads zsh/bash/powershell/fish history files
- [x] Identifies frequent command sequences
- [x] Suggests aliases for repeated commands
- [x] Detects anti-patterns (slow commands, unnecessary piping)
- [x] JSON output mode (`--json`)

### Output Quality
- [x] Suggestions include: alias name, command, frequency, estimated time saved
- [x] Stats show: top commands, daily usage, peak hours
- [x] Anti-pattern detection with replacement suggestions

### Developer Experience
- [x] Cross-platform (Linux, macOS, Windows)
- [x] Clear error messages
- [x] `--help` with examples
- [x] Verbose mode (`-v`)

## Tech Stack
- **Language:** Rust (performance + cross-platform)
- **Storage:** SQLite for pattern storage
- **LLM:** Optional Ollama integration for semantic pattern recognition
- **Shell plugins:** zsh/bash/fish integration scripts

## Architecture
```
terminal-guru/
├── src/
│   ├── main.rs          # CLI entry point
│   ├── cli.rs           # Command parsing
│   ├── history/         # Shell history readers
│   │   ├── mod.rs
│   │   ├── zsh.rs
│   │   ├── bash.rs
│   │   ├── fish.rs
│   │   └── powershell.rs
│   ├── analyzer/        # Pattern analysis
│   │   ├── mod.rs
│   │   ├── frequency.rs
│   │   └── anti_patterns.rs
│   ├── suggest/         # Suggestion engine
│   │   ├── mod.rs
│   │   ├── aliases.rs
│   │   └── optimizations.rs
│   ├── daemon/          # Background daemon
│   │   ├── mod.rs
│   │   └── watcher.rs
│   └── db/              # SQLite storage
│       ├── mod.rs
│       └── models.rs
├── SPEC.md
├── README.md
├── Cargo.toml
└── shell-plugins/       # Shell integration scripts
    ├── zsh.sh
    ├── bash.sh
    └── fish.fish
```

## Out of Scope (v1)
- Web UI / dashboard
- Remote sync across machines
- Machine learning model training
- Team/org analytics
