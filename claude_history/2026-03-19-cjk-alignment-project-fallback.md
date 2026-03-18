# CJK terminal alignment fix + project name fallback

## Summary
Two fixes:
1. Alignment: use `width()` (matching ratatui's internal rendering) + 2-column safety margin for ambiguous-width characters (●, → etc.) that render wider on CJK terminals.
2. Sessions with empty `cwd` in JSONL metadata had no project name. Added fallback to decode parent directory name.

## Changes
- `asm/src/ui.rs`:
  - Use `width()` for all calculations (consistent with ratatui's rendering engine)
  - Added +2 safety margin in `prompt_max` to compensate for ambiguous-width chars
  - Fixed `truncate_display` to subtract 3 for "..." (was adding beyond max)
- `asm-core/src/lib.rs`:
  - Added `decode_project_dir()`: decodes Claude project directory name using home dir as known prefix, validates path on disk, falls back to last hyphen-segment

## Design notes
- ratatui uses `width()` (non-CJK) for character positioning internally
- Using `width_cjk()` in our code creates a mismatch: ratatui positions chars at width() offsets, but terminal renders ambiguous chars wider, causing shift that grows with more ambiguous chars in the line
- The +2 margin accounts for ● (always present, +1) and occasional ambiguous chars in prompt text (+1)

## Verification
- `cargo build` — success
