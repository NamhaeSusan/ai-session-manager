# ai-session-manager

[![CI](https://github.com/NamhaeSusan/ai-session-manager/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/NamhaeSusan/ai-session-manager/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/NamhaeSusan/ai-session-manager/branch/main/graph/badge.svg)](https://codecov.io/gh/NamhaeSusan/ai-session-manager)

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
- Disk usage display (file size in tree view, preview panel, and stats)
- Bulk delete old sessions by age (press `D`)
- Session statistics popup with disk usage (press `i`)
- Configuration file support (`~/.config/asm/config.toml`)
- In-app settings editor (press `c`) — change settings without editing files
- Keyboard-driven navigation

## Project Structure

This project is a Cargo workspace with two members:

- **`asm-core`** — Shared library for session scanning, parsing, deletion, and conversation reading. Also used by [tre-file-manager](https://github.com/NamhaeSusan/tre-file-manager) as a git dependency.
- **`asm`** — TUI binary that depends on `asm-core`.

## Installation

### Homebrew (macOS / Linux)

```bash
brew install NamhaeSusan/tap/asm
```

### From source

```bash
cargo install --path asm
```

Requires Rust 1.70+.

## Development

The `CI` workflow runs formatting, clippy, tests, and uploads coverage to Codecov.

For public repositories, Codecov uploads from fork PRs can work without a token, but uploads to protected branches such as `main` may require a repository secret named `CODECOV_TOKEN`.

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
| `D`       | Bulk delete old sessions           |
| `/`       | Search / filter sessions        |
| `Space`   | Toggle folder expand/collapse   |
| `s`       | Cycle sort mode (date/project/messages) |
| `S`       | Toggle sort order (asc/desc)    |
| `c`       | Open settings                  |
| `i`       | Show session statistics         |
| `?`       | Show keybindings help           |
| `r`       | Refresh session list            |
| `Ctrl+d`  | Scroll preview down             |
| `Ctrl+u`  | Scroll preview up               |
| `Esc`     | Clear search / cancel           |
| `q`       | Quit                            |

## Configuration

Press `c` in the app to open the settings popup, or create `~/.config/asm/config.toml` (or `~/.asm.toml`) to customize behavior:

```toml
# Sort mode on startup: "date", "project", or "messages"
default_sort = "date"

# Expand all tree nodes by default
default_expanded = false

# Custom session directories
claude_projects_dir = "~/.claude/projects"
codex_sessions_dir = "~/.codex/sessions"

# Auto-add permission bypass flags on resume (default: true)
# Claude Code: --dangerously-skip-permissions
# Codex: --dangerously-bypass-approvals-and-sandbox
skip_permissions = true
```

All options are optional. Defaults are used for any missing values.

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `default_sort` | string | `"date"` | Startup sort mode. `"date"` (newest first), `"project"` (alphabetical), `"messages"` (most messages first) |
| `default_expanded` | bool | `true` | Whether tree nodes are expanded on startup |
| `claude_projects_dir` | string | `~/.claude/projects/` | Path to Claude Code projects directory |
| `codex_sessions_dir` | string | `~/.codex/sessions/` | Path to Codex sessions directory |
| `skip_permissions` | bool | `true` | Auto-add permission bypass flags on resume (`--dangerously-skip-permissions` for Claude Code, `--dangerously-bypass-approvals-and-sandbox` for Codex) |

## Session Sources

- **Claude Code**: `~/.claude/projects/`
- **Codex**: `~/.codex/`

## Dependencies

| Crate       | Package  | Purpose                   |
|-------------|----------|---------------------------|
| `asm-core`  | asm-core | Session scanning/parsing/deletion (shared library) |
| `ratatui`   | asm      | TUI rendering             |
| `crossterm` | asm      | Terminal input/output     |
| `serde`     | both     | Session data (de)serialize |
| `serde_json`| both     | JSON parsing              |
| `toml`      | asm      | Configuration file parsing |
| `unicode-width` | asm  | CJK/wide character display width |

## License

MIT
