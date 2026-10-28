# Investigation API

Each investigator has a separate notebook and route sketch. This is a solo
deduction loop with selectable investigators, not competitive turns. `state`
and every accepted command return only the selected investigator's notebook.
The public `case` field names the bank, airport, route length and clue sites.
Snapshots never expose the hidden route or undiscovered clues.

Additional command objects (inside the usual JSON-lines `command` request):

| Type | Behavior |
|---|---|
| `investigate` | Record all new evidence at the investigator's current tile |
| `plan_start` | Begin/restart a sketch with the named bank as tile 1 |
| `plan_step`, `q`, `r` | Append an unvisited adjacent tile, independent of pawn movement |
| `plan_undo` | Remove the last sketch tile |
| `plan_clear` | Empty the sketch |
| `submit_route` | Check a complete legal sketch against the exact ordered answer |

`plan_legal_moves` lists possible sketch extensions using public rules only.
The named airport must be the final tile. Incomplete submissions are errors and
do not count as attempts. A complete wrong route is an accepted attempt with
`last_submission: "incorrect"`; it reveals no partial answer. A correct route
sets `solved: true` and locks that notebook's sketch until reset. Reset clears
all notebooks, attempts, sketches, and pawn positions while increasing revision.

`--case PATH` loads a different authored JSON case on the same city. The CLI
certifies uniqueness before accepting a custom case. Rust embeddings may insert
`Mystery::new(case, board)` before adding `GamePlugin`. This validates authoring;
call the separate validator when uniqueness certification is required.

Case files contain spoilers by design, for authors and tests. `Mystery` and
notebooks are deliberately not reflected or serialized as whole ECS resources.
This is a local prototype: switching investigators deliberately switches which
notebook you control; it is not a network authentication mechanism.

```sh
rtk cargo build --no-default-features
rtk proxy python3 scripts/case_playtest.py
```
