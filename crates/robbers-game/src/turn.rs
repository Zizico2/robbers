//! Explicit local pass-and-play turn state. Only command dispatch advances it.
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub const ACTIONS_PER_TURN: u8 = 2;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Reflect)]
#[serde(rename_all = "snake_case")]
pub enum TurnPhase {
    #[default]
    Playing,
    Handoff,
    Finished,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TurnSnapshot {
    pub round: u32,
    pub active_player: u8,
    pub actions_remaining: u8,
    pub phase: TurnPhase,
    pub winner: Option<u8>,
}
