# Robbers

A Bevy **0.20** prototype for a hex-city deduction board game. Solve **The Morning
Escape**, a complete authored case with seven clues and one verified solution.
The city has 37 hexes, eight illustrated tile kinds, and four investigators taking turns. Every adjacent hex is walkable in all six directions.
Investigators can share tiles. The graphical interface and headless API share the same investigation,
private clue notebooks, route sketches and exact solution checking. See
[Investigation API](docs/investigation-api.md) and [Case authoring](docs/case-authoring.md).
Turns use two actions with explicit handoff; there are no dice.

## Play

```sh
rtk cargo run
```

Scarlet (player 1) starts. The banner names the active investigator, round, and
remaining actions. Each turn gives **2 actions**. Moving one hex, investigating,
or submitting a complete route costs **1 action**. Reading clues and editing
route sketches are free. Failed actions spend nothing.

Click a neighboring tile to move; gold outlines show legal destinations while
you have actions left. You cannot select another investigator, including through
keyboard shortcuts, pawn clicks or the CLI. The numbered strip shows turn order.

Press **E / End turn** when ready, even if you have unused actions. The game
passes to the next investigator and hides their notebook. **Enter / Begin turn**
reveals their notes and grants two actions. After player 4, a new round starts.
Using your last action does not automatically pass the turn, so you can finish
reading or sketching first.
The six keyboard directions are:

```text
         Q (NW)    W (NE)
     A (W)            D (E)
         Z (SW)    X (SE)
```

Visit a **? witness** and press **I** or **Investigate** to record its evidence.
Read the selected investigator's notebook; scroll over the inspector when needed.
Recorded witnesses become green **+** markers. Each investigator retains a
private notebook and route sketch between turns.

Press **P** or **Sketch** to plan the robbers' route. The sketch starts at the
named bank. Click adjacent tiles or use the same six direction keys to extend it;
the investigator stays in place. Gold-connected numbered badges show your sketch.
**Backspace / Undo** removes its last tile, **C / Clear** empties it, and **Esc / P**
returns to moving pawns. After clearing, click the bank to restart.

Press **Enter / Submit** when your sketch contains exactly eight tiles ending at
the named airport. A wrong route lets you retry; the exact ordered route solves
the case and ends the game. **R** resets every investigator's progress. Hover tiles for names and
coordinates. Illustrated paths do not constrain movement.

Occupied tiles have a translucent investigator-colored wash and a strong colored
rim, rendered with cached procedural Bevy meshes and translucent materials. Shared tiles split into equal pie sections in player-ID order; numbered
pawns sit in their matching sections. Empty tiles have no wash. The overlays
ignore pointer picking so movement and route picking continue to work normally.

The board fits the available window area beside the inspector, including 16:9,
ultrawide, and tall tiled windows. Fitting is isolated in
`crates/robbers-view/src/layout.rs`: one parent transform scales the board, and
sprite picking follows it automatically. Controls retain their proportions and
scale down together in shorter or narrower windows. Layout tests cover 16:9,
ultrawide, and the tall window sizes produced by a tiling desktop manager.

Default builds use Bevy dynamic linking for development. A standalone graphical
release is `rtk cargo build --release --no-default-features --features graphics`.
The existing pinned nightly/Cranelift and clang/mold configuration is retained.

## Headless agent interface

Build and run **without a renderer, window server, GPU, or tile assets**:

```sh
rtk cargo build --no-default-features
rtk proxy target/debug/robbers --headless
```

Send one JSON object per line on stdin. Receive one flushed JSON response per
line on stdout. There is no unsolicited output or frame stepping. Every accepted
command is fully applied before its response. EOF or `quit` exits cleanly.

```jsonl
{"id":1,"method":"schema"}
{"id":2,"method":"state"}
{"id":3,"method":"command","command":{"type":"select","player":0}}
{"id":4,"method":"command","command":{"type":"move","direction":"e"}}
{"id":5,"method":"command","command":{"type":"step","q":2,"r":0}}
{"id":6,"method":"command","command":{"type":"reset"}}
{"id":7,"method":"quit"}
```

`step` accepts **only an adjacent tile**, just like a mouse click. Example 5 is
intentionally invalid after example 4: it returns `not_adjacent` without moving.
Direction strings are `e`, `ne`, `nw`, `w`, `sw`, and `se`.

Success: `{"id":4,"result":{...snapshot...}}`.
Failure: `{"id":5,"error":{"code":"not_adjacent","message":"..."}}`.
Bad JSON, unknown methods, malformed commands, invalid player IDs, nonadjacent
steps and off-board movement return structured errors and keep the session alive.
Failed actions leave gameplay state and revision unchanged.

Snapshots include `schema_version`, monotonically increasing `revision`,
`selected_player` (the active investigator, stable IDs **0–3**), `moves`, `tiles`, sorted `players`, and
`legal_moves` with directions and destination coordinates. Accepted commands increment
revision; movement also increments moves. `select` only accepts the active player.
The schema-2 `turn` field reports round, active player, action budget, phase and
winner. `end_turn` starts a private handoff; `begin_turn` reveals the next notebook. Reset restores positions,
selection and move count while still increasing revision. A snapshot uses
coordinates and player IDs, never unstable Bevy entity IDs.

Coordinates use pointy-top axial `(q,r)`:
`e=(1,0)`, `ne=(1,-1)`, `nw=(0,-1)`, `w=(-1,0)`, `sw=(-1,1)`, `se=(0,1)`.
The board is the radius-3 hex centered on `(0,0)`.

`scripts/playtest.py` provides an importable Python `Driver`, plus a real-process
smoke test that visits every tile with every investigator:

```sh
rtk proxy python3 scripts/playtest.py
```

The CLI is the supported external transport in this slice. BRP is deferred
because the available package index did not provide `bevy_remote` 0.20 during
implementation. A future adapter can call the same synchronous `dispatch`
function; it does not need to implement gameplay again.

## Workspace

Inspired by `alveus-idle-cli`'s domain crates and central semantic command API,
with fewer crates for this smaller scope:

| Crate | Responsibility |
|---|---|
| `robbers-case` | Authored JSON, precise evidence rules, public briefings |
| `robbers-validator` | Bounded exhaustive uniqueness certification and authoring reports |
| `robbers-types` | Hex coordinates, directions, tile kinds, investigator identity |
| `robbers-game` | Deterministic board, ECS entities, session, stable snapshots |
| `robbers-command` | Semantic commands, validation, atomic mutations, feedback, discovery |
| `robbers-headless` | JSON-lines transport, no rendering dependencies |
| `robbers-view` | Tile sprites, pawns, witness markers, notebook, numbered route sketches, mode-aware input, screenshots |
| root `robbers` | Arguments and plugin composition |

Dependencies flow from view/headless through command/game to types. GUI input
emits `GameCommand` events; the observer calls the same `dispatch` function used
by the CLI and tests. Player positions live on ECS entities as `Hex`; graphical
transforms only project those positions. No gameplay depends on animation or
wall-clock time. The board resource is the immutable layout, also spawned into
tile entities for rendering and inspection.

To embed: create an `App`, add `MinimalPlugins`, `GamePlugin`, and `CommandPlugin`.
Call `dispatch(app.world_mut(), command)` and `snapshot(app.world_mut())` directly.
Setup is synchronous, so no startup frame is needed. The command enum, components,
board, and session are reflected for future remote tooling.

### Bevy 0.20 patterns

- HUD scenes use current **BSN**: `@scene()` inclusions, `--` child separators,
  and template defaults, rather than copying the old UI from the reference game.
- Native **flat pointer events** (`PointerClick`, `PointerOver`, `PointerOut`)
  and `bevy_ui_widgets::Button` replace deprecated interaction patterns.
- Sprite picking respects transparent PNG corners, so the hit target is a hex.
- Independent presentation systems use **`chain_weak()`**; correctness of game
  commands uses explicit synchronous execution, not incidental scheduler order.
- The view uses the 0.20 sprite API/backend and explicit `FontSize` conversions.

See the [Bevy 0.20 announcement](https://bevy.org/news/bevy-0-20/) and
[migration guide](https://bevy.org/learn/migration-guides/0-19-to-0-20/).

## Assets and verification

All original PNG art is drawn by `tools/generate_assets.py` using Pillow. There
is no image model, remote asset download, or proprietary input. PNGs are checked
in so playing does not require Python. To regenerate:

```sh
rtk proxy python3 -m pip install -r tools/requirements.txt
rtk proxy python3 tools/generate_assets.py
```

`assets/tile_catalog.png` previews the eight tiles. The graphical executable
loads the workspace's `assets/` directory regardless of the current working
directory; packaged distributions would need a relocatable asset path.

Occupancy fills/rims and selected, legal-move, and hover outlines use cached
`Mesh2d` geometry with `ColorMaterial`. The 1–4-player sectors follow the exact
hex edges; no overlay PNGs are generated or loaded. All visuals inherit the
board transform and ignore picking. Movement updates mesh handles and positions;
it does not rebuild geometry. See `hex_mesh.rs`, `occupancy.rs`, and `highlights.rs`
in `robbers-view`.

To reproduce a shared-tile screenshot, replay normal movement commands before
opening the view:

```sh
rtk cargo run -- --replay scripts/fixtures/shared_tile.json --screenshot docs/shared-tile.png
```

`--replay` also works with `--headless`. It reads a JSON array of the same command
objects used by the CLI and stops on the first invalid action.

```sh
# Fast, GPU-free rules and protocol tests
rtk cargo test --workspace --no-default-features --exclude robbers-view
rtk cargo build --no-default-features
rtk proxy python3 scripts/playtest.py
rtk proxy python3 scripts/case_playtest.py
rtk cargo run --no-default-features -- --validate-case

# Graphical build and captured frame (requires a display and GPU/software adapter)
rtk cargo run -- --screenshot docs/board.png

rtk cargo fmt --all -- --check
rtk cargo clippy --workspace --all-targets -- -D warnings
rtk cargo test -p robbers-view
```
