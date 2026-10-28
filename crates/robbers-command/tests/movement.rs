use bevy::prelude::*;
use robbers_command::{CommandPlugin, GameCommand, dispatch};
use robbers_game::{GamePlugin, Snapshot, snapshot};
use robbers_types::{Direction, Hex, TileKind};

fn app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, GamePlugin, CommandPlugin));
    app
}
fn send(app: &mut App, command: GameCommand) -> Snapshot {
    dispatch(app.world_mut(), command).unwrap()
}

#[test]
fn city_has_unique_connected_hexes_and_essential_landmarks() {
    let state = snapshot(app().world_mut());
    assert_eq!(state.tiles.len(), 37);
    let hexes: std::collections::BTreeSet<_> = state.tiles.iter().map(|t| t.hex).collect();
    assert_eq!(hexes.len(), 37);
    for tile in &state.tiles {
        assert!(tile.hex.distance(Hex::default()) <= 3);
        if tile.hex != Hex::default() {
            assert!(
                Direction::ALL
                    .iter()
                    .any(|d| tile.hex.neighbor(*d).distance(Hex::default())
                        < tile.hex.distance(Hex::default()))
            );
        }
    }
    for kind in TileKind::ALL {
        assert!(state.tiles.iter().any(|t| t.kind == kind));
    }
    assert_eq!(state.players.len(), 4);
}

#[test]
fn all_six_directions_are_single_hex_steps_without_road_constraints() {
    for direction in Direction::ALL {
        let mut app = app();
        send(
            &mut app,
            GameCommand::Move {
                direction: Direction::E,
            },
        );
        send(
            &mut app,
            GameCommand::Move {
                direction: Direction::Ne,
            },
        );
        assert_eq!(snapshot(app.world_mut()).players[0].hex, Hex::default());
        let state = send(&mut app, GameCommand::Move { direction });
        assert_eq!(state.players[0].hex, Hex::default().neighbor(direction));
        assert_eq!(state.moves, 3);
    }
}

#[test]
fn invalid_commands_are_atomic_and_cannot_teleport() {
    let mut app = app();
    let initial = snapshot(app.world_mut());
    for command in [
        GameCommand::Select { player: 4 },
        GameCommand::Step { q: 0, r: 0 },
        GameCommand::Step { q: -2, r: 1 },
        GameCommand::Step {
            q: i32::MAX,
            r: i32::MIN,
        },
    ] {
        assert!(dispatch(app.world_mut(), command).is_err());
        assert_eq!(snapshot(app.world_mut()), initial);
    }
    send(
        &mut app,
        GameCommand::Move {
            direction: Direction::W,
        },
    );
    let boundary = snapshot(app.world_mut());
    assert_eq!(
        dispatch(
            app.world_mut(),
            GameCommand::Move {
                direction: Direction::W
            }
        )
        .unwrap_err()
        .code,
        "off_board"
    );
    assert_eq!(snapshot(app.world_mut()), boundary);
}

#[test]
fn selection_moves_only_that_player_and_reset_preserves_revision_order() {
    let mut app = app();
    let initial = snapshot(app.world_mut());
    send(&mut app, GameCommand::Select { player: 2 });
    let moved = send(
        &mut app,
        GameCommand::Move {
            direction: Direction::W,
        },
    );
    for id in [0, 1, 3] {
        assert_eq!(moved.players[id], initial.players[id]);
    }
    assert_eq!(
        moved.players[2].hex,
        initial.players[2].hex.neighbor(Direction::W)
    );
    let reset = send(&mut app, GameCommand::Reset {});
    assert_eq!(reset.players, initial.players);
    assert_eq!(reset.selected_player, 0);
    assert_eq!(reset.moves, 0);
    assert_eq!(reset.revision, 3);
}

#[test]
fn investigators_can_share_tiles() {
    let mut app = app();
    send(
        &mut app,
        GameCommand::Move {
            direction: Direction::E,
        },
    );
    send(
        &mut app,
        GameCommand::Move {
            direction: Direction::Ne,
        },
    );
    send(&mut app, GameCommand::Select { player: 3 });
    send(
        &mut app,
        GameCommand::Move {
            direction: Direction::Se,
        },
    );
    let state = send(
        &mut app,
        GameCommand::Move {
            direction: Direction::Se,
        },
    );
    assert_eq!(state.players[0].hex, state.players[3].hex);
}

#[test]
fn observer_and_direct_controller_produce_the_same_state() {
    let mut direct = app();
    let mut observed = app();
    let commands = [
        GameCommand::Select { player: 1 },
        GameCommand::Move {
            direction: Direction::W,
        },
        GameCommand::Move {
            direction: Direction::Sw,
        },
    ];
    for command in commands {
        send(&mut direct, command.clone());
        observed.world_mut().trigger(command);
        observed.world_mut().flush();
    }
    assert_eq!(snapshot(direct.world_mut()), snapshot(observed.world_mut()));
}

#[test]
fn long_deterministic_walk_always_reports_correct_legal_moves() {
    let mut first = app();
    let mut second = app();
    let mut seed = 41u64;
    for _ in 0..500 {
        let state = snapshot(first.world_mut());
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        let next = &state.legal_moves[seed as usize % state.legal_moves.len()];
        let command = GameCommand::Move {
            direction: next.direction,
        };
        let a = send(&mut first, command.clone());
        let b = send(&mut second, command);
        assert_eq!(a, b);
        assert_eq!(a.players[0].hex, next.destination);
        for legal in &a.legal_moves {
            assert!(a.tiles.iter().any(|t| t.hex == legal.destination));
            assert_eq!(a.players[0].hex.distance(legal.destination), 1);
        }
    }
}
