---
description: Manage Claude Code sessions — list, show, delete, and more
argument-hint: <list|show|delete|projects|active> [options]
---

# Session Manager

Manage Claude Code sessions via a standalone binary at `${CLAUDE_PLUGIN_ROOT}/bin/claude-sessions`.

## Output Rules

- **Never show raw command output.** Always parse JSON output and present results as clean markdown.
- **Always use table format** — even for a single session. Every result gets a table.
- **Table columns (list):** `#` | Name | Session ID | Status | Lines | Size | Last Modified
  - Name: show `—` if null
  - Status: use 🟢 active / ⚫ inactive
  - Size: format with KB/MB
  - Last Modified: show date only (YYYY-MM-DD) from mtime

## Command Routing

Parse `$ARGUMENTS` to determine the operation (first word). If `$ARGUMENTS` is empty, default to `list` (current project only).

### list

Run with `--json`, parse the JSON lines, and present as a markdown table:

```
"${CLAUDE_PLUGIN_ROOT}/bin/claude-sessions" list --json
```

| User types | Behavior |
|-----------|----------|
| `/session` or `/session list` | List sessions for current project, format as table |
| `/session list --all` | `--json --all` → table (all projects) |
| `/session list --project <name>` | `--json --project <name>` → table |

**Always show a table** — if only one session, still use a table with one row.

### show

```
"${CLAUDE_PLUGIN_ROOT}/bin/claude-sessions" show <id>
```

Present as a key-value table.

### delete

```
"${CLAUDE_PLUGIN_ROOT}/bin/claude-sessions" show <id>    # first
"${CLAUDE_PLUGIN_ROOT}/bin/claude-sessions" delete <id>  # after confirming with user
```

Confirm with user before deleting. For force delete:

```
"${CLAUDE_PLUGIN_ROOT}/bin/claude-sessions" delete <id> --force
```

### projects

```
"${CLAUDE_PLUGIN_ROOT}/bin/claude-sessions" projects
```

Present as a table.

### active

```
"${CLAUDE_PLUGIN_ROOT}/bin/claude-sessions" active
```

Present as a table.

## Detailed Behavior

### list

**Defaults to current project only.** The binary auto-detects `process.cwd()` and filters accordingly. Use `--all` to show sessions from all projects, or `--project <name>` for a specific project.

Default limit is 20. If the user asks for "all sessions in this project", omit the limit but keep the project filter.

### show / delete / projects / active

Same as before. See routing table above.
