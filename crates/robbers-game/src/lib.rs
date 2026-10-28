//! Authoritative board and ECS state. No camera, assets, window, or wall-clock rules.
use bevy::prelude::*;
use robbers_types::*;
use serde::{Deserialize, Serialize};

pub const BOARD_RADIUS: i32 = 3;
pub const STARTS: [Hex; 4] = [
    Hex::new(-2, 1),
    Hex::new(2, -1),
    Hex::new(0, 2),
    Hex::new(0, -2),
];

#[derive(Resource, Reflect, Debug, Default)]
#[reflect(Resource)]
pub struct Session {
    pub selected_player: u8,
    /// Increases after each accepted command, including reset. Never rolls back.
    pub revision: u64,
    pub moves: u64,
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
    let position = players
        .iter()
        .find(|p| p.id == session.selected_player)
        .unwrap()
        .hex;
    let legal_moves = Direction::ALL
        .into_iter()
        .filter_map(|direction| {
            let destination = position.neighbor(direction);
            board.contains(destination).then_some(LegalMove {
                direction,
                destination,
            })
        })
        .collect();
    Snapshot {
        schema_version: 1,
        revision: session.revision,
        selected_player: session.selected_player,
        moves: session.moves,
        tiles: board.tiles.clone(),
        players,
        legal_moves,
    }
}
