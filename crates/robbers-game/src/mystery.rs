//! Private case state and investigator-specific knowledge. No reflected spoilers.
use crate::Board;
use bevy::prelude::*;
use robbers_case::{CaseBriefing, CaseDefinition, Clue, bundled_case};
use robbers_types::{Direction, Hex};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Resource)]
pub struct Mystery {
    definition: CaseDefinition,
}
impl Default for Mystery {
    fn default() -> Self {
        Self {
            definition: bundled_case(),
        }
    }
}
impl Mystery {
    pub fn new(definition: CaseDefinition, board: &Board) -> Result<Self, String> {
        definition.validate(board)?;
        Ok(Self { definition })
    }
    pub fn briefing(&self) -> CaseBriefing {
        self.definition.public_briefing()
    }
    pub fn clues_at(&self, hex: Hex) -> Vec<&Clue> {
        self.definition
            .clues
            .iter()
            .filter(|c| c.at == hex)
            .collect()
    }
    pub fn validate_route(&self, route: &[Hex], board: &Board) -> Result<(), String> {
        self.definition.validate_route(route, board)
    }
    pub fn is_solution(&self, route: &[Hex]) -> bool {
        route == self.definition.solution
    }
    pub fn notebook_snapshot(&self, notebook: &Notebook) -> NotebookSnapshot {
        NotebookSnapshot {
            clues: self
                .definition
                .clues
                .iter()
                .filter(|c| notebook.discovered.contains(&c.id))
                .map(|c| DiscoveredClue {
                    id: c.id.clone(),
                    at: c.at,
                    witness: c.witness.clone(),
                    statement: c.evidence.statement(),
                })
                .collect(),
            plan: notebook.plan.clone(),
            attempts: notebook.attempts,
            last_submission: notebook.last_submission,
            solved: notebook.solved,
        }
    }
    pub fn plan_destinations(&self, notebook: &Notebook, board: &Board) -> Vec<Hex> {
        if notebook.solved || notebook.plan.len() >= self.definition.route_length {
            return vec![];
        }
        let Some(last) = notebook.plan.last() else {
            return vec![];
        };
        Direction::ALL
            .into_iter()
            .map(|d| last.neighbor(d))
            .filter(|hex| {
                board.contains(*hex)
                    && !notebook.plan.contains(hex)
                    && if notebook.plan.len() + 1 == self.definition.route_length {
                        *hex == self.definition.airport
                    } else {
                        *hex != self.definition.airport
                    }
            })
            .collect()
    }
}

#[derive(Resource, Default)]
pub struct Investigation {
    pub notebooks: [Notebook; 4],
}
#[derive(Default)]
pub struct Notebook {
    pub discovered: BTreeSet<String>,
    pub plan: Vec<Hex>,
    pub attempts: u32,
    pub last_submission: Option<Verdict>,
    pub solved: bool,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Verdict {
    Correct,
    Incorrect,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DiscoveredClue {
    pub id: String,
    pub at: Hex,
    pub witness: String,
    pub statement: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NotebookSnapshot {
    pub clues: Vec<DiscoveredClue>,
    pub plan: Vec<Hex>,
    pub attempts: u32,
    pub last_submission: Option<Verdict>,
    pub solved: bool,
}
