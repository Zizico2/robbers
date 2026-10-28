//! Authored cases and precise clue semantics. No ECS state or rendering.
use robbers_types::{Direction, Hex, TileKind};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// Small read-only adapter, implemented by the game's board and test fixtures.
pub trait CityMap {
    fn kind(&self, hex: Hex) -> Option<TileKind>;
    fn tile_count(&self) -> usize;
}

/// Authoring format: contains spoilers. Never serialize this into gameplay snapshots.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CaseDefinition {
    pub id: String,
    pub title: String,
    pub briefing: String,
    pub bank: Hex,
    pub airport: Hex,
    /// Counts both endpoints. A route cannot revisit a tile.
    pub route_length: usize,
    pub clues: Vec<Clue>,
    pub solution: Vec<Hex>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Clue {
    pub id: String,
    pub at: Hex,
    pub witness: String,
    pub evidence: Evidence,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Evidence {
    Visits {
        kind: TileKind,
    },
    Avoids {
        kind: TileKind,
    },
    AtStep {
        step: usize,
        kind: TileKind,
    },
    Before {
        first: TileKind,
        second: TileKind,
    },
    ImmediatelyAfter {
        first: TileKind,
        second: TileKind,
    },
    Departs {
        kind: TileKind,
        direction: Direction,
    },
    FinalMove {
        direction: Direction,
    },
}

impl Evidence {
    pub fn matches(&self, route: &[Hex], map: &impl CityMap) -> bool {
        let first_index = |kind| route.iter().position(|h| map.kind(*h) == Some(kind));
        match *self {
            Self::Visits { kind } => first_index(kind).is_some(),
            Self::Avoids { kind } => first_index(kind).is_none(),
            Self::AtStep { step, kind } => step
                .checked_sub(1)
                .and_then(|i| route.get(i))
                .is_some_and(|h| map.kind(*h) == Some(kind)),
            Self::Before { first, second } => first_index(first)
                .zip(first_index(second))
                .is_some_and(|(a, b)| a < b),
            Self::ImmediatelyAfter { first, second } => route
                .windows(2)
                .any(|pair| map.kind(pair[0]) == Some(first) && map.kind(pair[1]) == Some(second)),
            Self::Departs { kind, direction } => first_index(kind)
                .and_then(|i| {
                    route
                        .get(i + 1)
                        .map(|next| route[i].neighbor(direction) == *next)
                })
                .unwrap_or(false),
            Self::FinalMove { direction } => {
                route.len() >= 2
                    && route[route.len() - 2].neighbor(direction) == route[route.len() - 1]
            }
        }
    }

    /// Text is generated from the actual rule, preventing contradictory prose.
    pub fn statement(&self) -> String {
        match *self {
            Self::Visits { kind } => {
                format!("The route visited at least one {} tile.", kind.label())
            }
            Self::Avoids { kind } => format!("The route never visited a {} tile.", kind.label()),
            Self::AtStep { step, kind } => format!(
                "Tile {step} of the route was a {}. Count the bank as tile 1.",
                kind.label()
            ),
            Self::Before { first, second } => format!(
                "The first {} was visited before the first {}. Both were on the route.",
                first.label(),
                second.label()
            ),
            Self::ImmediatelyAfter { first, second } => format!(
                "At least once, a {} tile was immediately followed by a {} tile.",
                first.label(),
                second.label()
            ),
            Self::Departs { kind, direction } => format!(
                "Immediately after the first {} tile, the robbers moved {}.",
                kind.label(),
                direction_name(direction)
            ),
            Self::FinalMove { direction } => format!(
                "The final move into the airport was {}.",
                direction_name(direction)
            ),
        }
    }
}

fn direction_name(direction: Direction) -> &'static str {
    match direction {
        Direction::E => "east",
        Direction::Ne => "northeast",
        Direction::Nw => "northwest",
        Direction::W => "west",
        Direction::Sw => "southwest",
        Direction::Se => "southeast",
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CaseBriefing {
    pub id: String,
    pub title: String,
    pub briefing: String,
    pub bank: Hex,
    pub airport: Hex,
    pub route_length: usize,
    /// Public places to investigate. Evidence stays hidden until discovered.
    pub clue_sites: Vec<Hex>,
}

impl CaseDefinition {
    pub fn public_briefing(&self) -> CaseBriefing {
        CaseBriefing {
            id: self.id.clone(),
            title: self.title.clone(),
            briefing: self.briefing.clone(),
            bank: self.bank,
            airport: self.airport,
            route_length: self.route_length,
            clue_sites: self
                .clues
                .iter()
                .map(|c| c.at)
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect(),
        }
    }

    pub fn validate_route(&self, route: &[Hex], map: &impl CityMap) -> Result<(), String> {
        if route.len() != self.route_length {
            return Err(format!(
                "The route needs exactly {} tiles, including the bank and airport.",
                self.route_length
            ));
        }
        if route.first() != Some(&self.bank) || route.last() != Some(&self.airport) {
            return Err("Start at the named bank and finish at the named airport.".into());
        }
        if route.iter().any(|h| map.kind(*h).is_none()) {
            return Err("Every route tile must be on the board.".into());
        }
        if route.iter().collect::<BTreeSet<_>>().len() != route.len() {
            return Err("The route cannot revisit a tile.".into());
        }
        if route.windows(2).any(|pair| pair[0].distance(pair[1]) != 1) {
            return Err("Every route step must enter an adjacent hex.".into());
        }
        Ok(())
    }

    /// Authoring checks; uniqueness is the validator crate's separate concern.
    pub fn validate(&self, map: &impl CityMap) -> Result<(), String> {
        if self.id.trim().is_empty()
            || self.title.trim().is_empty()
            || self.briefing.trim().is_empty()
        {
            return Err("Case identity, title and briefing must be nonempty.".into());
        }
        if !(2..=map.tile_count()).contains(&self.route_length) {
            return Err("Route length must fit the board.".into());
        }
        if map.kind(self.bank) != Some(TileKind::Bank)
            || map.kind(self.airport) != Some(TileKind::Airport)
        {
            return Err("Case endpoints must be a bank and an airport.".into());
        }
        let mut ids = BTreeSet::new();
        for clue in &self.clues {
            if clue.id.trim().is_empty() || !ids.insert(&clue.id) {
                return Err("Clue IDs must be nonempty and unique.".into());
            }
            if clue.witness.trim().is_empty() || map.kind(clue.at).is_none() {
                return Err(format!("Invalid witness or location for clue {}.", clue.id));
            }
            if let Evidence::AtStep { step, .. } = clue.evidence
                && !(1..=self.route_length).contains(&step)
            {
                return Err(format!("Invalid step in clue {}.", clue.id));
            }
        }
        self.validate_route(&self.solution, map)?;
        if let Some(clue) = self
            .clues
            .iter()
            .find(|c| !c.evidence.matches(&self.solution, map))
        {
            return Err(format!("Authored solution contradicts clue {}.", clue.id));
        }
        Ok(())
    }
}

pub fn bundled_case() -> CaseDefinition {
    serde_json::from_str(include_str!("../cases/morning_escape.json"))
        .expect("bundled case JSON is valid")
}
