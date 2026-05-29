# claude-sessions

Claude Code session management plugin — list, inspect, and delete sessions across projects.

## Install

```bash
# Add the marketplace
claude plugins install git@github.com:javyxu/claude-sessions.git

# Install the plugin from the marketplace
claude plugins install claude-sessions
```

Or install from local path:

```bash
git clone git@github.com:javyxu/claude-sessions.git
cd claude-sessions && bun install && bun run build
claude plugins install ./claude-sessions
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
# Install dependencies
bun install

# Build binary
bun run build

# Type check
bun run typecheck
```

## Tech Stack

- TypeScript
- Bun
- Node.js File System API

## License

MIT
