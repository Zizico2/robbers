//! The only gameplay write path. GUI, reflected events, CLI, BRP and tests use this API.
use bevy::prelude::*;
use robbers_game::{Board, Investigation, Mystery, STARTS, Session, Snapshot, Verdict, snapshot};
use robbers_types::*;
use serde::{Deserialize, Serialize};

#[derive(Event, Reflect, Serialize, Deserialize, Debug, Clone)]
#[reflect(Event)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum GameCommand {
    Select {
        player: u8,
    },
    Move {
        direction: Direction,
    },
    /// A single adjacent hex, never a teleport or pathfinding shortcut.
    Step {
        q: i32,
        r: i32,
    },
    Reset {},
    Investigate {},
    PlanStart {},
    PlanStep {
        q: i32,
        r: i32,
    },
    PlanUndo {},
    PlanClear {},
    SubmitRoute {},
}
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct CommandError {
    pub code: String,
    pub message: String,
}
#[derive(Resource, Default)]
pub struct Feedback(pub String);
pub struct CommandPlugin;
impl Plugin for CommandPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Feedback>()
            .register_type::<Direction>()
            .register_type::<GameCommand>()
            .add_observer(observe_command);
    }
}
fn observe_command(event: On<GameCommand>, mut commands: Commands) {
    let command = event.event().clone();
    commands.queue(move |world: &mut World| {
        let _ = dispatch(world, command);
    });
}
/// Synchronous and atomic: errors leave board, players and session unchanged.
pub fn dispatch(world: &mut World, command: GameCommand) -> Result<Snapshot, CommandError> {
    let result = apply(world, &command);
    world.resource_mut::<Feedback>().0 = match &result {
        Ok(()) => match command {
            GameCommand::Select { player } => {
                format!("{} is ready to move.", PLAYER_NAMES[player as usize])
            }
            GameCommand::Reset {} => "New patrol. Explore the city.".into(),
            GameCommand::Investigate {} => "Evidence added to this investigator's notebook.".into(),
            GameCommand::PlanStart {} => "Route sketch started at the bank.".into(),
            GameCommand::PlanStep { .. } => "Route sketch extended by one tile.".into(),
            GameCommand::PlanUndo {} => "Last route tile removed.".into(),
            GameCommand::PlanClear {} => "Route sketch cleared.".into(),
            GameCommand::SubmitRoute {} => {
                let selected = world.resource::<Session>().selected_player as usize;
                if world.resource::<Investigation>().notebooks[selected].solved {
                    "Case solved! You reconstructed the getaway route.".into()
                } else {
                    "That route is incorrect. Review your clues and try again.".into()
                }
            }
            _ => "Moved one hex. Where next?".into(),
        },
        Err(error) => error.message.clone(),
    };
    result.map(|()| snapshot(world))
}
fn error(code: &str, message: &str) -> CommandError {
    CommandError {
        code: code.into(),
        message: message.into(),
    }
}
fn apply(world: &mut World, command: &GameCommand) -> Result<(), CommandError> {
    let selected = world.resource::<Session>().selected_player as usize;
    if matches!(
        command,
        GameCommand::PlanStart {}
            | GameCommand::PlanStep { .. }
            | GameCommand::PlanUndo {}
            | GameCommand::PlanClear {}
            | GameCommand::SubmitRoute {}
    ) && world.resource::<Investigation>().notebooks[selected].solved
    {
        return Err(error(
            "already_solved",
            "This investigator solved the case. Reset to start over.",
        ));
    }
    match *command {
        GameCommand::Select { player } => {
            if usize::from(player) >= PLAYER_NAMES.len() {
                return Err(error(
                    "unknown_player",
                    "Choose an investigator from 0 to 3.",
                ));
            }
            world.resource_mut::<Session>().selected_player = player;
        }
        GameCommand::Move { .. } | GameCommand::Step { .. } => {
            let selected = world.resource::<Session>().selected_player;
            let mut query = world.query::<(Entity, &Investigator, &Hex)>();
            let (entity, _, current) = query
                .iter(world)
                .find(|(_, p, _)| p.id == selected)
                .unwrap();
            let destination = match *command {
                GameCommand::Move { direction } => current.neighbor(direction),
                GameCommand::Step { q, r } => Hex::new(q, r),
                _ => unreachable!(),
            };
            if !world.resource::<Board>().contains(destination) {
                return Err(error("off_board", "That hex is outside the city."));
            }
            if current.distance(destination) != 1 {
                return Err(error(
                    "not_adjacent",
                    "Move to one of the six neighboring hexes.",
                ));
            }
            *world.get_mut::<Hex>(entity).unwrap() = destination;
            world.resource_mut::<Session>().moves += 1;
        }
        GameCommand::Reset {} => {
            let mut query = world.query::<(&Investigator, &mut Hex)>();
            for (player, mut hex) in query.iter_mut(world) {
                *hex = STARTS[player.id as usize];
            }
            let mut session = world.resource_mut::<Session>();
            session.selected_player = 0;
            session.moves = 0;
            world.insert_resource(Investigation::default());
        }
        GameCommand::Investigate {} => {
            let mut query = world.query::<(&Investigator, &Hex)>();
            let hex = *query
                .iter(world)
                .find(|(p, _)| p.id as usize == selected)
                .unwrap()
                .1;
            let discovered = &world.resource::<Investigation>().notebooks[selected].discovered;
            let clues = world.resource::<Mystery>().clues_at(hex);
            if clues.is_empty() {
                return Err(error(
                    "no_clue",
                    "There is no witness to investigate on this tile.",
                ));
            }
            let new_ids: Vec<_> = clues
                .iter()
                .filter(|c| !discovered.contains(&c.id))
                .map(|c| c.id.clone())
                .collect();
            if new_ids.is_empty() {
                return Err(error(
                    "already_investigated",
                    "This investigator already recorded the evidence here.",
                ));
            }
            world.resource_mut::<Investigation>().notebooks[selected]
                .discovered
                .extend(new_ids);
        }
        GameCommand::PlanStart {} => {
            let bank = world.resource::<Mystery>().briefing().bank;
            let mut investigation = world.resource_mut::<Investigation>();
            investigation.notebooks[selected].plan = vec![bank];
            investigation.notebooks[selected].last_submission = None;
        }
        GameCommand::PlanStep { q, r } => {
            let hex = Hex::new(q, r);
            let notebook = &world.resource::<Investigation>().notebooks[selected];
            if notebook.plan.is_empty() {
                return Err(error(
                    "plan_not_started",
                    "Start a route sketch at the bank first.",
                ));
            }
            if !world
                .resource::<Mystery>()
                .plan_destinations(notebook, world.resource::<Board>())
                .contains(&hex)
            {
                return Err(error(
                    "invalid_plan_step",
                    "Choose an unvisited adjacent hex; the final tile must be the named airport.",
                ));
            }
            let mut investigation = world.resource_mut::<Investigation>();
            investigation.notebooks[selected].plan.push(hex);
            investigation.notebooks[selected].last_submission = None;
        }
        GameCommand::PlanUndo {} => {
            if world.resource::<Investigation>().notebooks[selected]
                .plan
                .is_empty()
            {
                return Err(error("empty_plan", "There is no route tile to undo."));
            }
            let mut investigation = world.resource_mut::<Investigation>();
            investigation.notebooks[selected].plan.pop();
            investigation.notebooks[selected].last_submission = None;
        }
        GameCommand::PlanClear {} => {
            let mut investigation = world.resource_mut::<Investigation>();
            investigation.notebooks[selected].plan.clear();
            investigation.notebooks[selected].last_submission = None;
        }
        GameCommand::SubmitRoute {} => {
            let plan = &world.resource::<Investigation>().notebooks[selected].plan;
            world
                .resource::<Mystery>()
                .validate_route(plan, world.resource::<Board>())
                .map_err(|message| error("incomplete_route", &message))?;
            let correct = world.resource::<Mystery>().is_solution(plan);
            let mut investigation = world.resource_mut::<Investigation>();
            let notebook = &mut investigation.notebooks[selected];
            notebook.attempts += 1;
            notebook.last_submission = Some(if correct {
                Verdict::Correct
            } else {
                Verdict::Incorrect
            });
            notebook.solved = correct;
        }
    }
    world.resource_mut::<Session>().revision += 1;
    Ok(())
}
/// Machine-readable discovery shared by both transports.
pub fn schema() -> serde_json::Value {
    serde_json::json!({
        "schema_version": 1,
        "commands": [
            {"type":"select", "player":"integer 0..3"},
            {"type":"move", "direction":["e","ne","nw","w","sw","se"]},
            {"type":"step", "q":"integer", "r":"integer"},
            {"type":"reset"}
            ,{"type":"investigate"}
            ,{"type":"plan_start"}
            ,{"type":"plan_step", "q":"integer", "r":"integer"}
            ,{"type":"plan_undo"}
            ,{"type":"plan_clear"}
            ,{"type":"submit_route"}
        ],
        "coordinates":"pointy-top axial: e=(1,0), ne=(1,-1), nw=(0,-1), w=(-1,0), sw=(-1,1), se=(0,1)",
        "rules": "One neighboring hex per command. Every tile is walkable. Investigators may share tiles. No turns or dice yet.",
        "cli": {"methods":["state","schema","command","quit"], "framing":"one JSON object per line; one response per line"},
        "event_type": "robbers_command::GameCommand"
    })
}
