# Investigation API

Each investigator has a separate notebook and route sketch. Four players take
turns in order. Player 0 starts round 1 with two actions. `state`
and every accepted command return only the selected investigator's notebook.
During handoff the notebook is empty until `begin_turn`.
The public `case` field names the bank, airport, route length and clue sites.
Snapshots never expose the hidden route or undiscovered clues.

Additional command objects (inside the usual JSON-lines `command` request):

| Type | Behavior |
|---|---|
| `investigate` | Record new evidence at the active investigator's tile; costs 1 action |
| `plan_start` | Begin/restart a sketch with the named bank as tile 1 |
| `plan_step`, `q`, `r` | Append an unvisited adjacent tile, independent of pawn movement |
| `plan_undo` | Remove the last sketch tile |
| `plan_clear` | Empty the sketch |
| `submit_route` | Check a complete legal sketch; costs 1 action, including wrong guesses |
| `end_turn` | Forfeit unused actions and pass to the next investigator; notebook hidden |
| `begin_turn` | End handoff, reveal the new investigator's notebook and grant 2 actions |
| `select` | Only accepts the already-active investigator; cannot bypass turn order |

Movement costs 1 action per hex. Reading and all `plan_*` commands are free but
only available to the active player outside handoff. Exhausting actions does not
auto-pass: editing remains possible until `end_turn`. `no_actions`, `not_your_turn`
and `handoff_pending` errors leave state and budget unchanged. A new round starts
after player 3. The first correct submission sets phase `finished` and a winner;
all gameplay stops until reset. Schema version 2 exposes these rules in `turn`.

`plan_legal_moves` lists possible sketch extensions using public rules only.
The named airport must be the final tile. Incomplete submissions are errors and
do not count as attempts. A complete wrong route is an accepted attempt with
`last_submission: "incorrect"`; it reveals no partial answer. A correct route
sets `solved: true` and ends the game until reset. Reset clears
all notebooks, attempts, sketches, and pawn positions while increasing revision.

`--case PATH` loads a different authored JSON case on the same city. The CLI
certifies uniqueness before accepting a custom case. Rust embeddings may insert
`Mystery::new(case, board)` before adding `GamePlugin`. This validates authoring;
call the separate validator when uniqueness certification is required.

Case files contain spoilers by design, for authors and tests. `Mystery` and
notebooks are deliberately not reflected or serialized as whole ECS resources.
The local handoff screen keeps the next notebook hidden until its owner begins their turn.

```sh
rtk cargo build --no-default-features
rtk proxy python3 scripts/case_playtest.py
```
