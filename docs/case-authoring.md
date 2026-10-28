# Case authoring

`robbers-case` contains JSON case definitions and exact clue semantics. The
bundled `morning_escape.json` is an authoring file containing **spoilers**.
Gameplay should expose `CaseBriefing` and discovered clue statements only.

A route follows hex adjacency, includes both endpoints in its stated length,
and never repeats a tile. Banks and airports are named coordinates. `CityMap`
adapts a board without coupling case rules to rendering or an ECS resource.

Clue prose is generated from typed evidence: visits, avoids, one-based tile
number, first-occurrence ordering, an adjacent landmark pair, departure direction
after the first landmark, or the final move. This keeps prose and validation in
agreement. Every clue has a stable ID, witness, and location to investigate.

`CaseDefinition::validate` checks identity, locations, endpoints, route structure,
and consistency of every clue with the author's route. `robbers-validator`
enumerates legal simple paths using endpoints, route length and the public clues,
independently of the stored solution. Distance-to-airport pruning removes only
branches that cannot reach the endpoint in time. Search is bounded at a million
nodes; an incomplete search never certifies uniqueness.

```sh
rtk cargo run --no-default-features -- --validate-case
rtk cargo run --no-default-features -- --validate-case path/to/case.json
```

This authoring command reports candidate and matching route counts, search work,
completion and uniqueness. It exits nonzero for invalid, ambiguous or incomplete
cases. It does not print the hidden path. This is separate from the normal
JSON-lines gameplay API.
