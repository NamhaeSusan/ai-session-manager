# CJK terminal alignment fix + project name fallback

## Summary
Two fixes:
1. Alignment: use `width_cjk()` for all measurements and budget borders as 4 cols (CJK). Guarantees no clipping on CJK terminals. ~2 cols wasted on non-CJK.
2. Sessions with empty `cwd` in JSONL metadata had no project name. Added fallback to decode parent directory name.

## Changes
- `asm/src/ui.rs`:
  - `inner = area.width - 4` (CJK border width: │ = 2 cols each)
  - `prefix_cols = 5/7` (● = 2 cols in CJK)
  - `proj_cols` uses `width_cjk()`
  - `pad_or_truncate`, `truncate_to_width`, `truncate_display` all use `width_cjk()`
  - Removed two-pass compensation logic and `cjk_ambiguous_extra()` — no longer needed
  - Fixed `truncate_display` to subtract 3 for "..."
- `asm-core/src/lib.rs`:
  - Added `decode_project_dir()`: decodes Claude project directory name using home dir as known prefix, validates path on disk, falls back to last hyphen-segment

## Design notes
- Previous approaches (width() + fixed margin, width() + two-pass measurement) failed because ratatui uses width() for cursor positioning, creating unpredictable drift with ambiguous chars
- Using width_cjk() everywhere means we budget for the MAXIMUM possible terminal width of each character. Content can never overflow — it's an upper bound guarantee
- inner = area.width - 4 accounts for border chars being 2 cols on CJK terminals
- On non-CJK terminals, ~2-3 cols of extra margin appear (acceptable tradeoff)

## Verification
- `cargo build` — success
