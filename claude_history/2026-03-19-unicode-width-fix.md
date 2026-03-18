# Unicode width fix for tree view metadata alignment

## Summary
CJK characters (Korean, Chinese, Japanese) occupy 2 terminal columns but `pad_or_truncate` was using `chars().count()` (1 char = 1 col). This caused metadata misalignment and truncation when session prompts contained CJK text.

## Changes
- `asm/Cargo.toml`: Added `unicode-width = "0.2"` dependency
- `asm/src/ui.rs`:
  - Added `use unicode_width::UnicodeWidthStr`
  - Rewrote `pad_or_truncate` to use display width (`.width()`) instead of char count
  - Added `truncate_to_width` helper that iterates chars accumulating display width
  - Changed `entry.project_name.len()` → `.width()` for project name column calculation
  - Fixed `truncate_display` to use `.width()` for consistent behavior
  - Removed `.max(8)` on `prompt_max` that forced minimum prompt width, causing meta overflow
  - Added post-truncation padding in `pad_or_truncate` to fill CJK boundary gaps
- `CLAUDE.md`: Added unicode-width to tech stack table
- `README.md`: Added unicode-width to dependencies table

## Verification
- `cargo build` — success, no warnings
