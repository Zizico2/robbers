# Working on Robbers

- Prefix shell commands with `rtk` (use `rtk proxy` for commands without a filter).
- This prototype implements a local pass-and-play investigation: board movement, clue notebooks,
  route sketches, exact guesses, authoring validation and two-action pass-and-play turns.
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
- Cases live in `robbers-case/cases`; authoring files contain spoilers. Normal
  snapshots must never expose solutions, undiscovered evidence or other notebooks.
- Certify cases with `rtk cargo run --no-default-features -- --validate-case [PATH]`.
  A search-budget exhaustion must never count as a uniqueness proof.
- Test the full mystery with `rtk proxy python3 scripts/case_playtest.py`.
- Mouse and keyboard adapters must honor move/sketch mode using the same commands.
- Android (`platforms/android`) and web use `robbers-app` for shared graphical startup.
  Keep viewport rules in `robbers-view/src/layout.rs`. Short landscape windows
  use the compact UI and touch-scrollable Notes overlay.
- Export with `scripts/build_android.py` / `scripts/build_web.py`; these select LLVM
  for cross builds. Android assets must use the packaged reader, web assets relative URLs.

- Turn rules: move/investigate/submit cost one of two actions; sketching is free.
  Only `end_turn` followed by `begin_turn` advances the active player. Direct
  `select` cannot change actors. Handoff snapshots and UI must hide notebooks.
- A correct route ends the game; only reset remains available. Invalid actions
  must not spend points. The normal protocol uses schema version 2.
