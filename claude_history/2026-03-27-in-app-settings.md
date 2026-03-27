# In-app Settings Editor

## Summary
Added settings popup (press `c`) so users can change config without editing files manually.

## Changes
- `asm/src/config.rs` — added `loaded_from` field, `save_path()`, `save()` methods for persisting config
- `asm/src/app.rs` — added `Settings`/`SettingsEdit` modes, key handlers, config mutation logic
- `asm/src/tree.rs` — added `SortMode::prev()`, `TreeState::set_sort()`
- `asm/src/ui.rs` — added `draw_settings_popup()`, updated status bar and help popup
- `README.md` — added settings keybinding and feature description
- `CLAUDE.md` — added in-app settings editor section

## Verification
- `cargo build` passes
