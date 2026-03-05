# ai-session-manager

A terminal UI for browsing, resuming, and deleting Claude Code and Codex sessions.

## Layout

```
+------------------+----------------------------------------+
| SESSIONS         | PREVIEW                                |
|                  |                                        |
| > claude         | Project: ~/my-project                  |
|   ~/my-project   | Session: abc123                        |
|     * abc123     | Started: 2026-03-01 14:22              |
|     * def456     |                                        |
|   ~/other        | --- Conversation ---                   |
|                  | User: fix the login bug                |
| > codex          | Assistant: I'll look at the auth...    |
|   ~/my-project   |                                        |
|     * xyz789     |                                        |
|                  |                                        |
+------------------+----------------------------------------+
| [Enter] resume  [d] delete  [/] search  [q] quit          |
+-----------------------------------------------------------+
```

## Features

- 3-level tree view: Tool (claude/codex) -> Project -> Session
- Session preview panel with conversation history
- Resume sessions directly (`exec` into `claude --resume` or `codex --resume`)
- Search/filter sessions with `/`
- Delete sessions with confirmation prompt
- Sort sessions by date, project name, or message count (press `s`)
- Session statistics popup (press `i`)
- Configuration file support (`~/.config/asm/config.toml`)
- Keyboard-driven navigation

## Installation

```bash
cargo install --path .
```

Requires Rust 1.70+.

## Usage

```
asm
```

### Keybindings

| Key       | Action                          |
|-----------|---------------------------------|
| `j` / `k` | Move down / up                  |
| `Enter`   | Resume session or toggle folder |
| `d`       | Delete session (with confirmation) |
| `/`       | Search / filter sessions        |
| `Space`   | Toggle folder expand/collapse   |
| `s`       | Cycle sort mode (date/project/messages) |
| `S`       | Toggle sort order (asc/desc)    |
| `i`       | Show session statistics         |
| `?`       | Show keybindings help           |
| `r`       | Refresh session list            |
| `Ctrl+d`  | Scroll preview down             |
| `Ctrl+u`  | Scroll preview up               |
| `Esc`     | Clear search / cancel           |
| `q`       | Quit                            |

## Configuration

Create `~/.config/asm/config.toml` (or `~/.asm.toml`) to customize behavior:

```toml
# Sort mode on startup: "date", "project", or "messages"
default_sort = "date"

# Expand all tree nodes by default
default_expanded = false

# Custom session directories
claude_projects_dir = "~/.claude/projects"
codex_sessions_dir = "~/.codex/sessions"
```

All options are optional. Defaults are used for any missing values.

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `default_sort` | string | `"date"` | Startup sort mode. `"date"` (newest first), `"project"` (alphabetical), `"messages"` (most messages first) |
| `default_expanded` | bool | `true` | Whether tree nodes are expanded on startup |
| `claude_projects_dir` | string | `~/.claude/projects/` | Path to Claude Code projects directory |
| `codex_sessions_dir` | string | `~/.codex/sessions/` | Path to Codex sessions directory |

## Session Sources

- **Claude Code**: `~/.claude/projects/`
- **Codex**: `~/.codex/`

## Dependencies

| Crate       | Purpose                   |
|-------------|---------------------------|
| `ratatui`   | TUI rendering             |
| `crossterm` | Terminal input/output     |
| `serde`     | Session data (de)serialize |
| `serde_json`| JSON parsing              |
| `toml`      | Configuration file parsing |

## License

MIT
