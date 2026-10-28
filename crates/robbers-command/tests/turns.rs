use bevy::prelude::*;
use robbers_command::{CommandPlugin, GameCommand, dispatch};
use robbers_game::{GamePlugin, TurnPhase, snapshot};
use robbers_types::Direction;

fn app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, GamePlugin, CommandPlugin));
    app
}

#[test]
fn cannot_select_a_different_player_and_two_actions_do_not_auto_pass() {
    let mut app = app();
    let initial = snapshot(app.world_mut());
    assert_eq!(
        dispatch(app.world_mut(), GameCommand::Select { player: 1 })
            .unwrap_err()
            .code,
        "not_your_turn"
    );
    assert_eq!(snapshot(app.world_mut()), initial);
    dispatch(app.world_mut(), GameCommand::Investigate {}).unwrap();
    let spent = dispatch(
        app.world_mut(),
        GameCommand::Move {
            direction: Direction::E,
        },
    )
    .unwrap();
    assert_eq!(spent.turn.actions_remaining, 0);
    assert_eq!(spent.turn.active_player, 0);
    assert!(spent.legal_moves.is_empty());
    for command in [
        GameCommand::Move {
            direction: Direction::E,
        },
        GameCommand::Investigate {},
        GameCommand::SubmitRoute {},
    ] {
        assert_eq!(
            dispatch(app.world_mut(), command).unwrap_err().code,
            "no_actions"
        );
        assert_eq!(snapshot(app.world_mut()), spent);
    }
    let planned = dispatch(app.world_mut(), GameCommand::PlanStart {}).unwrap();
    assert_eq!(planned.turn.actions_remaining, 0);
    assert_eq!(planned.notebook.plan.len(), 1);
}

#[test]
fn handoff_hides_the_next_notebook_until_begin_turn_and_wraps_rounds() {
    let mut app = app();
    dispatch(app.world_mut(), GameCommand::Investigate {}).unwrap();
    for player in [1, 2, 3, 0] {
        let handoff = dispatch(app.world_mut(), GameCommand::EndTurn {}).unwrap();
        assert_eq!(handoff.turn.phase, TurnPhase::Handoff);
        assert_eq!(handoff.selected_player, player);
        assert!(handoff.notebook.clues.is_empty() && handoff.notebook.plan.is_empty());
        for command in [
            GameCommand::Select { player },
            GameCommand::EndTurn {},
            GameCommand::PlanStart {},
            GameCommand::Move {
                direction: Direction::E,
            },
        ] {
            assert_eq!(
                dispatch(app.world_mut(), command).unwrap_err().code,
                "handoff_pending"
            );
            assert_eq!(snapshot(app.world_mut()), handoff);
        }
        let begun = dispatch(app.world_mut(), GameCommand::BeginTurn {}).unwrap();
        assert_eq!(begun.turn.actions_remaining, 2);
        assert_eq!(begun.turn.phase, TurnPhase::Playing);
        if player == 0 {
            assert_eq!(begun.notebook.clues.len(), 1);
            assert_eq!(begun.turn.round, 2);
        }
        assert!(dispatch(app.world_mut(), GameCommand::BeginTurn {}).is_err());
        assert_eq!(snapshot(app.world_mut()), begun);
    }
}

#[test]
fn failed_actions_do_not_cost_points_and_reset_restores_the_first_turn() {
    let mut app = app();
    let initial = snapshot(app.world_mut());
    assert!(dispatch(app.world_mut(), GameCommand::Step { q: 99, r: 99 }).is_err());
    assert_eq!(snapshot(app.world_mut()), initial);
    dispatch(app.world_mut(), GameCommand::EndTurn {}).unwrap();
    let reset = dispatch(app.world_mut(), GameCommand::Reset {}).unwrap();
    assert_eq!(reset.turn, initial.turn);
    assert!(reset.revision > initial.revision);
}
