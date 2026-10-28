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
and consistency of every clue with the author's route. A separate exhaustive
validator will certify uniqueness using the public constraints.
