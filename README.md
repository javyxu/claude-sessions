# claude-sessions

Manage Claude Code sessions via CLI — list, inspect, and delete sessions across projects.

## Install

Download the binary for your platform from [Releases](https://github.com/javyxu/claude-sessions/releases), or build from source:

```bash
cargo build --release
# Binary at target/release/claude-sessions
```

## Commands

| Command | Aliases | Description |
|---------|---------|-------------|
| `list` | `ls` | List sessions for current project (default 20) |
| `show <id>` | `info`, `inspect` | Show detailed session info |
| `delete <id>` | `rm`, `remove` | Delete a session and all associated files |
| `projects` | `prj` | List projects with session counts |
| `active` | `running` | Show currently active sessions |

## Options

| Option | Applies to | Description |
|--------|------------|-------------|
| `--project <name>` | `list` | Filter by project name (fuzzy match) |
| `--limit N` | `list` | Limit results to N most recent |
| `--json` | `list` | Output as JSON Lines |
| `--all` | `list` | Show sessions from all projects |
| `--force` | `delete` | Force delete an active session |

## Usage

```bash
# List current project sessions
claude-sessions list

# List sessions from all projects
claude-sessions list --all

# JSON output
claude-sessions list --json

# Search by project name
claude-sessions list --project my-app

# Show session details
claude-sessions show 24fc85db-xxxx-xxxx-xxxx-xxxxxxxxxxxx

# Delete a session
claude-sessions delete 24fc85db-xxxx-xxxx-xxxx-xxxxxxxxxxxx

# Force delete an active session
claude-sessions delete 24fc85db-xxxx-xxxx-xxxx-xxxxxxxxxxxx --force

# View project summary
claude-sessions projects

# View active sessions
claude-sessions active
```

## Development

```bash
# Build release binary
cargo build --release

# The binary will be at target/release/claude-sessions
```

## Tech Stack

- Rust
- serde / serde_json

## License

MIT
