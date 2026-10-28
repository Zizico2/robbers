use bevy::prelude::*;
use robbers_command::{CommandPlugin, GameCommand, dispatch};
use robbers_game::{GamePlugin, Snapshot, Verdict, snapshot};

fn app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, GamePlugin, CommandPlugin));
    app
}
fn send(app: &mut App, command: GameCommand) -> Snapshot {
    if let GameCommand::Select { player } = command {
        while snapshot(app.world_mut()).selected_player != player {
            dispatch(app.world_mut(), GameCommand::EndTurn {}).unwrap();
            dispatch(app.world_mut(), GameCommand::BeginTurn {}).unwrap();
        }
    }
    dispatch(app.world_mut(), command).unwrap()
}
fn plan(app: &mut App, route: &[(i32, i32)]) {
    send(app, GameCommand::PlanStart {});
    for &(q, r) in &route[1..] {
        send(app, GameCommand::PlanStep { q, r });
    }
}
const ANSWER: [(i32, i32); 8] = [
    (-3, 1),
    (-2, 1),
    (-2, 2),
    (-1, 2),
    (0, 2),
    (0, 1),
    (1, 1),
    (1, 2),
];
const WRONG: [(i32, i32); 8] = [
    (-3, 1),
    (-2, 0),
    (-1, 0),
    (0, 0),
    (0, 1),
    (0, 2),
    (1, 1),
    (1, 2),
];

#[test]
fn investigation_requires_presence_and_each_investigator_has_private_knowledge() {
    let mut app = app();
    let initial = snapshot(app.world_mut());
    assert!(initial.notebook.clues.is_empty());
    let found = send(&mut app, GameCommand::Investigate {});
    assert_eq!(found.notebook.clues.len(), 1);
    assert_eq!(found.notebook.clues[0].id, "patrol_radio");
    assert!(dispatch(app.world_mut(), GameCommand::Investigate {}).is_err());
    assert_eq!(snapshot(app.world_mut()), found);
    let other = send(&mut app, GameCommand::Select { player: 1 });
    assert!(other.notebook.clues.is_empty());
    let other = send(&mut app, GameCommand::Investigate {});
    assert_eq!(other.notebook.clues[0].id, "school_camera");
    let returned = send(&mut app, GameCommand::Select { player: 0 });
    assert_eq!(returned.notebook.clues, found.notebook.clues);
    send(
        &mut app,
        GameCommand::Move {
            direction: robbers_types::Direction::E,
        },
    );
    let before = snapshot(app.world_mut());
    assert!(dispatch(app.world_mut(), GameCommand::Investigate {}).is_err());
    assert_eq!(snapshot(app.world_mut()), before);
}

#[test]
fn planning_is_independent_of_movement_and_rejects_illegal_steps_atomically() {
    let mut app = app();
    let initial = snapshot(app.world_mut());
    send(&mut app, GameCommand::PlanStart {});
    for command in [
        GameCommand::PlanStep { q: 0, r: 0 },
        GameCommand::PlanStep { q: -3, r: 1 },
        GameCommand::SubmitRoute {},
    ] {
        let before = snapshot(app.world_mut());
        assert!(dispatch(app.world_mut(), command).is_err());
        assert_eq!(snapshot(app.world_mut()), before);
    }
    send(&mut app, GameCommand::PlanStep { q: -2, r: 1 });
    assert_eq!(snapshot(app.world_mut()).players, initial.players);
    let undone = send(&mut app, GameCommand::PlanUndo {});
    assert_eq!(undone.notebook.plan.len(), 1);
    let cleared = send(&mut app, GameCommand::PlanClear {});
    assert!(cleared.notebook.plan.is_empty());
}

#[test]
fn wrong_guesses_allow_retry_correct_guesses_solve_and_reset_clears_every_notebook() {
    let mut app = app();
    plan(&mut app, &WRONG);
    let wrong = send(&mut app, GameCommand::SubmitRoute {});
    assert_eq!(wrong.notebook.last_submission, Some(Verdict::Incorrect));
    assert_eq!(wrong.notebook.attempts, 1);
    assert!(!wrong.notebook.solved);
    assert_eq!(wrong.turn.actions_remaining, 1);
    plan(&mut app, &ANSWER);
    let won = send(&mut app, GameCommand::SubmitRoute {});
    assert_eq!(won.notebook.last_submission, Some(Verdict::Correct));
    assert_eq!(won.notebook.attempts, 2);
    assert!(won.notebook.solved);
    assert_eq!(won.turn.phase, robbers_game::TurnPhase::Finished);
    assert_eq!(won.turn.winner, Some(0));
    for command in [
        GameCommand::PlanUndo {},
        GameCommand::EndTurn {},
        GameCommand::BeginTurn {},
        GameCommand::Select { player: 1 },
        GameCommand::Move {
            direction: robbers_types::Direction::E,
        },
    ] {
        assert_eq!(
            dispatch(app.world_mut(), command).unwrap_err().code,
            "game_finished"
        );
        assert_eq!(snapshot(app.world_mut()), won);
    }
    let reset = send(&mut app, GameCommand::Reset {});
    assert!(reset.notebook.plan.is_empty() && reset.notebook.clues.is_empty());
    assert_eq!(reset.notebook.attempts, 0);
    assert_eq!(reset.turn.winner, None);
    assert_eq!(reset.turn.actions_remaining, 2);
    assert!(reset.revision > won.revision);
    assert!(
        send(&mut app, GameCommand::Select { player: 1 })
            .notebook
            .clues
            .is_empty()
    );
}
