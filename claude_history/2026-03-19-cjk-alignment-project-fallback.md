# CJK terminal alignment fix + project name fallback

## Summary
Two fixes:
1. Alignment: use `width()` (matching ratatui) then measure actual CJK overflow per-line and shrink prompt to compensate exactly.
2. Sessions with empty `cwd` in JSONL metadata had no project name. Added fallback to decode parent directory name.

## Changes
- `asm/src/ui.rs`:
  - Use `width()` for all calculations (consistent with ratatui's rendering engine)
  - Two-pass approach: build prompt first, then measure `cjk_ambiguous_extra()` (= width_cjk - width) for ●, project name, and prompt text. If overflow > 0, re-truncate prompt by that amount.
  - Added `cjk_ambiguous_extra()` helper
  - Fixed `truncate_display` to subtract 3 for "..." (was adding beyond max)
- `asm-core/src/lib.rs`:
  - Added `decode_project_dir()`: decodes Claude project directory name using home dir as known prefix, validates path on disk, falls back to last hyphen-segment

## Design notes
- ratatui uses `width()` (non-CJK) for character positioning internally
- On CJK terminals, ambiguous-width chars (●, →, etc.) render as 2 cols but ratatui counts them as 1
- Fixed margin (+2, +3) fails because the number of ambiguous chars varies per line
- Two-pass approach measures the ACTUAL overflow for each line and compensates precisely
- The re-truncated prompt has ≤ original ambiguous chars, so the overflow is guaranteed to be resolved

## Verification
- `cargo build` — success
