# CJK terminal alignment fix + project name fallback

## Summary
Two fixes:
1. Ambiguous-width Unicode symbols (●, ▾, ▸) render as 2 columns on CJK terminals but `width()` returns 1. Switched all width calculations to `width_cjk()`.
2. Sessions with empty `cwd` in JSONL metadata had no project name. Added fallback to decode parent directory name.

## Changes
- `asm/src/ui.rs`:
  - `width()` → `width_cjk()` in `pad_or_truncate`, `truncate_to_width`, `truncate_display`, proj_cols
  - `prefix_cols`: 4/6 → 5/7 (● is 2 cols in CJK mode)
  - `meta.len()` → `meta.width_cjk()` for consistency
  - Fixed `truncate_display` to subtract 3 for "..." (was adding beyond max)
- `asm-core/src/lib.rs`:
  - Added `decode_project_dir()`: decodes Claude project directory name using home dir as known prefix, validates path on disk, falls back to last hyphen-segment

## Verification
- `cargo build` — success
