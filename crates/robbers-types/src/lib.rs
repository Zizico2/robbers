//! Stable domain vocabulary, independent of rendering and input devices.
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Pointy-top axial coordinates. +q is east; +r is southeast.
#[derive(
    Component,
    Reflect,
    Serialize,
    Deserialize,
    Debug,
    Default,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
)]
#[reflect(Component)]
pub struct Hex {
    pub q: i32,
    pub r: i32,
}

impl Hex {
    pub const fn new(q: i32, r: i32) -> Self {
        Self { q, r }
    }
    pub fn neighbor(self, direction: Direction) -> Self {
        let (q, r) = direction.delta();
        Self::new(self.q + q, self.r + r)
    }
    pub fn distance(self, other: Self) -> u64 {
        let q = i64::from(self.q) - i64::from(other.q);
        let r = i64::from(self.r) - i64::from(other.r);
        q.abs().max(r.abs()).max((q + r).abs()) as u64
    }
    pub fn world_position(self, radius: f32) -> Vec2 {
        Vec2::new(
            3.0_f32.sqrt() * (self.q as f32 + self.r as f32 / 2.0) * radius,
            -1.5 * self.r as f32 * radius,
        )
    }
}

#[derive(Reflect, Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    E,
    Ne,
    Nw,
    W,
    Sw,
    Se,
}
impl Direction {
    pub const ALL: [Self; 6] = [Self::E, Self::Ne, Self::Nw, Self::W, Self::Sw, Self::Se];
    pub const fn delta(self) -> (i32, i32) {
        match self {
            Self::E => (1, 0),
            Self::Ne => (1, -1),
            Self::Nw => (0, -1),
            Self::W => (-1, 0),
            Self::Sw => (-1, 1),
            Self::Se => (0, 1),
        }
    }
}

#[derive(Component, Reflect, Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[reflect(Component)]
#[serde(rename_all = "snake_case")]
pub enum TileKind {
    Residential,
    Market,
    Park,
    Bank,
    Airport,
    Police,
    Hospital,
    School,
}
impl TileKind {
    pub const ALL: [Self; 8] = [
        Self::Residential,
        Self::Market,
        Self::Park,
        Self::Bank,
        Self::Airport,
        Self::Police,
        Self::Hospital,
        Self::School,
    ];
    pub const fn slug(self) -> &'static str {
        match self {
            Self::Residential => "residential",
            Self::Market => "market",
            Self::Park => "park",
            Self::Bank => "bank",
            Self::Airport => "airport",
            Self::Police => "police",
            Self::Hospital => "hospital",
            Self::School => "school",
        }
    }
    pub const fn label(self) -> &'static str {
        match self {
            Self::Residential => "Residences",
            Self::Market => "Market",
            Self::Park => "Park",
            Self::Bank => "Bank",
            Self::Airport => "Airport",
            Self::Police => "Police station",
            Self::Hospital => "Hospital",
            Self::School => "School",
        }
    }
}

#[derive(Component, Reflect, Debug, Clone, Copy)]
#[reflect(Component)]
pub struct Investigator {
    pub id: u8,
}
pub const PLAYER_NAMES: [&str; 4] = ["Scarlet", "Azure", "Amber", "Jade"];
pub const PLAYER_COLORS: [[f32; 3]; 4] = [
    [0.87, 0.30, 0.29],
    [0.30, 0.64, 0.84],
    [0.94, 0.70, 0.28],
    [0.31, 0.72, 0.57],
];
