# Working on Robbers

- Prefix shell commands with `rtk` (use `rtk proxy` for commands without a filter).
- Keep scope small: this prototype currently implements the board and movement.
- `robbers-command::dispatch` is the single gameplay write path. Human input,
  scripts and future remote adapters must use the same commands and validation.
- Keep the game/headless crates independent of graphics, assets, a display and time.
- Agent protocol: run `target/debug/robbers --headless`; send JSON-lines `schema`,
  `state`, `command`, `quit`. See README for the wire format and coordinates.
- Player IDs 0-3 and axial coordinates are the external identity. Never expose
  Bevy entity IDs as stable protocol identifiers.
- Failed commands must not mutate gameplay or revision. Reset increments revision.
- Use current Bevy 0.20 APIs. BSN uses `@scene()` and `--`; pointer events are flat.
- Draw new tile art in `tools/generate_assets.py`, then regenerate PNGs. Do not
  use AI image generation. Keep generation deterministic and Pillow-only.
- Verify rules with `rtk cargo test --workspace --no-default-features --exclude robbers-view`.
- Verify the actual transport with `rtk cargo build --no-default-features` then
  `rtk proxy python3 scripts/playtest.py`. Driver is importable by other scripts.
- For visual changes, inspect an actual screenshot with `--screenshot PATH`.
