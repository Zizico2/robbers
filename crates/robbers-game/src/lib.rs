//! Authoritative board and ECS state. No camera, assets, window, or wall-clock rules.
mod mystery;
mod turn;
use bevy::prelude::*;
pub use mystery::*;
use robbers_types::*;
use serde::{Deserialize, Serialize};
pub use turn::*;

pub const BOARD_RADIUS: i32 = 3;
pub const STARTS: [Hex; 4] = [
    Hex::new(-2, 1),
    Hex::new(2, -1),
    Hex::new(0, 2),
    Hex::new(0, -2),
];

#[derive(Resource, Reflect, Debug)]
#[reflect(Resource)]
pub struct Session {
    pub selected_player: u8,
    /// Increases after each accepted command, including reset. Never rolls back.
    pub revision: u64,
    pub moves: u64,
    pub round: u32,
    pub actions_remaining: u8,
    pub phase: TurnPhase,
    pub winner: Option<u8>,
}
impl Default for Session {
    fn default() -> Self {
        Self {
            selected_player: 0,
            revision: 0,
            moves: 0,
            round: 1,
            actions_remaining: ACTIONS_PER_TURN,
            phase: TurnPhase::Playing,
            winner: None,
        }
    }
}
impl Session {
    pub fn can_spend_action(&self) -> bool {
        self.phase == TurnPhase::Playing && self.actions_remaining > 0
    }
    pub fn visible_notebook(
        &self,
        mystery: &Mystery,
        investigation: &Investigation,
    ) -> NotebookSnapshot {
        if self.phase == TurnPhase::Handoff {
            NotebookSnapshot::default()
        } else {
            mystery.notebook_snapshot(&investigation.notebooks[self.selected_player as usize])
        }
    }
    pub fn turn_snapshot(&self) -> TurnSnapshot {
        TurnSnapshot {
            round: self.round,
            active_player: self.selected_player,
            actions_remaining: self.actions_remaining,
            phase: self.phase,
            winner: self.winner,
        }
    }
}

#[derive(Resource, Reflect, Debug)]
#[reflect(Resource)]
pub struct Board {
    pub tiles: Vec<Tile>,
}
#[derive(Reflect, Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Tile {
    pub hex: Hex,
    pub kind: TileKind,
}
impl Default for Board {
    fn default() -> Self {
        let mut tiles = Vec::new();
        for r in -BOARD_RADIUS..=BOARD_RADIUS {
            for q in -BOARD_RADIUS..=BOARD_RADIUS {
                let hex = Hex::new(q, r);
                if hex.distance(Hex::default()) > BOARD_RADIUS as u64 {
                    continue;
                }
                let kind = match (q, r) {
                    (-3, 1) | (2, 0) => TileKind::Bank,
                    (1, -3) | (1, 2) => TileKind::Airport,
                    (-2, 1) | (2, -1) => TileKind::Police,
                    (-1, -1) => TileKind::Hospital,
                    (0, 2) => TileKind::School,
                    (0, 0) | (-2, 2) | (0, -2) => TileKind::Park,
                    (1, 0) | (-1, 2) | (0, -1) => TileKind::Market,
                    _ => TileKind::Residential,
                };
                tiles.push(Tile { hex, kind });
            }
        }
        Self { tiles }
    }
}
impl Board {
    pub fn contains(&self, hex: Hex) -> bool {
        self.tiles.iter().any(|tile| tile.hex == hex)
    }
    pub fn kind(&self, hex: Hex) -> Option<TileKind> {
        self.tiles
            .iter()
            .find(|tile| tile.hex == hex)
            .map(|tile| tile.kind)
    }
}

impl robbers_case::CityMap for Board {
    fn kind(&self, hex: Hex) -> Option<TileKind> {
        self.kind(hex)
    }
    fn tile_count(&self) -> usize {
        self.tiles.len()
    }
}

pub struct GamePlugin;
impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Board>()
            .init_resource::<Session>()
            .init_resource::<Mystery>()
            .init_resource::<Investigation>()
            .register_type::<Hex>()
            .register_type::<Investigator>()
            .register_type::<TileKind>()
            .register_type::<Tile>()
            .register_type::<Board>()
            .register_type::<Session>();
        // Synchronous initialization lets embedding harnesses inspect immediately.
        let tiles = app.world().resource::<Board>().tiles.clone();
        for tile in tiles {
            app.world_mut().spawn((tile.hex, tile.kind));
        }
        for (id, hex) in STARTS.into_iter().enumerate() {
            app.world_mut().spawn((Investigator { id: id as u8 }, hex));
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct PlayerSnapshot {
    pub id: u8,
    pub name: String,
    pub hex: Hex,
}
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct LegalMove {
    pub direction: Direction,
    pub destination: Hex,
}
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Snapshot {
    pub schema_version: u32,
    pub revision: u64,
    pub selected_player: u8,
    pub moves: u64,
    pub tiles: Vec<Tile>,
    pub players: Vec<PlayerSnapshot>,
    pub legal_moves: Vec<LegalMove>,
    pub case: robbers_case::CaseBriefing,
    /// Only the selected investigator's knowledge and sketch are exposed.
    pub notebook: NotebookSnapshot,
    pub plan_legal_moves: Vec<Hex>,
    pub turn: TurnSnapshot,
}
pub fn snapshot(world: &mut World) -> Snapshot {
    let mut query = world.query::<(&Investigator, &Hex)>();
    let mut players: Vec<_> = query
        .iter(world)
        .map(|(player, hex)| PlayerSnapshot {
            id: player.id,
            name: PLAYER_NAMES[player.id as usize].into(),
            hex: *hex,
        })
        .collect();
    players.sort_by_key(|player| player.id);
    let session = world.resource::<Session>();
    let board = world.resource::<Board>();
    let mystery = world.resource::<Mystery>();
    let notebooks = world.resource::<Investigation>();
    let notebook = &notebooks.notebooks[session.selected_player as usize];
    let position = players
        .iter()
        .find(|p| p.id == session.selected_player)
        .unwrap()
        .hex;
    let legal_moves = Direction::ALL
        .into_iter()
        .filter(|_| session.can_spend_action())
        .filter_map(|direction| {
            let destination = position.neighbor(direction);
            board.contains(destination).then_some(LegalMove {
                direction,
                destination,
            })
        })
        .collect();
    Snapshot {
        schema_version: 2,
        revision: session.revision,
        selected_player: session.selected_player,
        moves: session.moves,
        tiles: board.tiles.clone(),
        players,
        legal_moves,
        case: mystery.briefing(),
        notebook: session.visible_notebook(mystery, notebooks),
        plan_legal_moves: if session.phase == TurnPhase::Playing {
            mystery.plan_destinations(notebook, board)
        } else {
            vec![]
        },
        turn: session.turn_snapshot(),
    }
}
