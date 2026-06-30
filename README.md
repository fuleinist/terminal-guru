# terminal-guru 🧙‍♂️

AI-powered terminal assistant that watches your shell history and proactively suggests optimizations, aliases, and automations.

## Quick Start

```bash
# Analyze your shell history and get suggestions
tguru suggest

# See your terminal usage stats
tguru stats

# Run the background daemon
tguru daemon start
```

## Features

- **Smart alias suggestions** — "You run `docker compose up -d && docker compose logs -f` 12x/day. Create alias: `dcup`?"
- **Anti-pattern detection** — "You're piping to `grep` when `rg` would be 5x faster"
- **Usage statistics** — Top commands, daily activity, peak hours
- **Privacy-first** — All analysis runs locally, no data leaves your machine
- **Cross-platform** — Linux, macOS, Windows

## Supported Shells

- Zsh
- Bash
- Fish
- PowerShell

## Installation

```bash
cargo install terminal-guru
```

Or download a binary from the [releases page](https://github.com/fuleinist/terminal-guru/releases).

## License

MIT
