//! Case controls and notebook. Every gameplay action uses the shared command API.
use crate::{GOLD, MUTED, PAPER};
use bevy::{
    ecs::system::SystemParam,
    input::mouse::{MouseScrollUnit, MouseWheel},
    picking::pointer::PointerButton,
    prelude::*,
    ui::InteractionDisabled,
    ui_widgets::Button,
};
use robbers_command::GameCommand;
use robbers_game::{Board, Investigation, Mystery, Session, TurnPhase, Verdict};
use robbers_types::{Direction, Hex, PLAYER_COLORS, PLAYER_NAMES};

#[derive(Resource, Default)]
pub(super) struct PlanningMode(pub bool);
#[derive(Event)]
struct TogglePlanning;
#[derive(Component, Clone, Default)]
struct TurnBadge(u8);
#[derive(Component, Clone, Default)]
enum CaseText {
    #[default]
    Briefing,
    TurnTitle,
    TurnStatus,
    TurnRules,
    Progress,
    Notebook,
    Verdict,
    Mode,
    Investigator,
}
#[derive(Component, Clone, Default)]
struct NotebookScroll;
#[derive(Clone, Default)]
enum Action {
    #[default]
    Primary,
    Command(GameCommand),
    Toggle,
}

#[derive(Component, Clone, Default)]
struct ButtonAction(Action);
#[derive(Component, Clone, Default)]
struct ButtonLabel(Action);

#[derive(SystemParam)]
pub(super) struct CaseInteraction<'w> {
    mode: Res<'w, PlanningMode>,
    session: Res<'w, Session>,
    investigation: Res<'w, Investigation>,
    mystery: Res<'w, Mystery>,
}
impl CaseInteraction<'_> {
    pub fn planning(&self) -> bool {
        self.mode.0
    }
    pub fn can_move(&self) -> bool {
        self.session.can_spend_action()
    }
    pub fn selected(&self) -> u8 {
        self.session.selected_player
    }
    pub fn plan(&self) -> &[Hex] {
        if self.session.phase == TurnPhase::Handoff {
            &[]
        } else {
            &self.investigation.notebooks[self.selected() as usize].plan
        }
    }
    pub fn notebook(&self) -> robbers_game::NotebookSnapshot {
        self.session
            .visible_notebook(&self.mystery, &self.investigation)
    }
    pub fn plan_destinations(&self, board: &Board) -> Vec<Hex> {
        if self.session.phase != TurnPhase::Playing {
            return vec![];
        }
        self.mystery.plan_destinations(
            &self.investigation.notebooks[self.selected() as usize],
            board,
        )
    }
    pub fn tile_command(&self, hex: Hex) -> GameCommand {
        if self.planning() {
            if self.plan().is_empty() && hex == self.mystery.briefing().bank {
                GameCommand::PlanStart {}
            } else {
                GameCommand::PlanStep { q: hex.q, r: hex.r }
            }
        } else {
            GameCommand::Step { q: hex.q, r: hex.r }
        }
    }
    pub fn direction_command(&self, direction: Direction) -> GameCommand {
        if self.planning() {
            self.plan()
                .last()
                .map(|h| self.tile_command(h.neighbor(direction)))
                .unwrap_or(GameCommand::PlanStart {})
        } else {
            GameCommand::Move { direction }
        }
    }
}

pub(super) struct CaseUiPlugin;
impl Plugin for CaseUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlanningMode>()
            .add_observer(toggle_planning)
            .add_observer(|event: On<GameCommand>, mut mode: ResMut<PlanningMode>| {
                if matches!(
                    event.event(),
                    GameCommand::Reset {} | GameCommand::EndTurn {} | GameCommand::BeginTurn {}
                ) {
                    mode.0 = false;
                }
            })
            .add_systems(Startup, setup_ui)
            .add_systems(Update, (case_keys, scroll_notebook))
            .add_systems(PostUpdate, (sync_case_ui, sync_action_buttons).chain_weak());
    }
}

fn toggle_planning(
    _: On<TogglePlanning>,
    mut mode: ResMut<PlanningMode>,
    session: Res<Session>,
    investigation: Res<Investigation>,
    mut commands: Commands,
) {
    let notebook = &investigation.notebooks[session.selected_player as usize];
    if notebook.solved || session.phase != TurnPhase::Playing {
        mode.0 = false;
        return;
    }
    mode.0 = !mode.0;
    if mode.0 && notebook.plan.is_empty() {
        commands.trigger(GameCommand::PlanStart {});
    }
}

fn action_button(label: &str, action: Action) -> impl Scene {
    let label_action = action.clone();
    let event_action = action.clone();
    bsn! {
        Button
        ButtonAction(action)
        Node { flex_grow: 1.0, flex_basis: px(0), min_height: px(34), padding: UiRect::all(px(8)),
            border: px(1), border_radius: BorderRadius::all(px(5)), justify_content: JustifyContent::Center, align_items: AlignItems::Center }
        BorderColor::all(MUTED)
        BackgroundColor(Color::srgb(0.12,0.21,0.24))
        on(move |event: On<PointerClick>, disabled:Query<Has<InteractionDisabled>>,session:Res<Session>, mut commands: Commands| {
            if event.button != PointerButton::Primary || disabled.get(event.entity).unwrap_or(true) {return;}
            match &event_action {
                Action::Command(command) => { commands.trigger(command.clone()); }
                Action::Toggle => { commands.trigger(TogglePlanning); }
                Action::Primary => { commands.trigger(primary_command(&session)); }
            }
        })
        Children [ Text(label) ButtonLabel(label_action) TextFont { font_size: px(13) } TextColor(PAPER) ]
    }
}

fn primary_command(session: &Session) -> GameCommand {
    match session.phase {
        TurnPhase::Playing => GameCommand::EndTurn {},
        TurnPhase::Handoff => GameCommand::BeginTurn {},
        TurnPhase::Finished => GameCommand::Reset {},
    }
}

fn turn_badge(id: u8) -> impl Scene {
    let label = format!("{} {}", id + 1, PLAYER_NAMES[id as usize]);
    let color = Color::srgb_from_array(PLAYER_COLORS[id as usize]);
    bsn! {
        TurnBadge(id)
        Node {flex_grow:1.0,height:px(27),border_radius:BorderRadius::all(px(4)),justify_content:JustifyContent::Center,align_items:AlignItems::Center}
        BackgroundColor(Color::srgb(0.12,0.21,0.24))
        Children [Text(label) TextFont {font_size:px(10)} TextColor(color)]
    }
}

fn panel() -> impl Scene {
    bsn! {
        Node {position_type:PositionType::Absolute,right:px(24),top:px(20),bottom:px(20),width:px(360),flex_direction:FlexDirection::Column,row_gap:px(8)}
        Children [
            Text("") CaseText::TurnTitle TextFont {font_size:px(21)} TextColor(GOLD)
            --
            Text("") CaseText::TurnStatus TextFont {font_size:px(15)} TextColor(PAPER)
            --
            @action_button("End turn / E",Action::Primary)
            Node {flex_grow:0.0,flex_basis:Val::Auto,flex_shrink:0.0}
            --
            Text("") CaseText::TurnRules TextFont {font_size:px(12)} TextColor(MUTED)
            --
            Text("TURN ORDER / advances when you end your turn") TextFont {font_size:px(10)} TextColor(MUTED)
            --
            Node {column_gap:px(8),flex_shrink:0.0}
            Children [@turn_badge(0) -- @turn_badge(1) -- @turn_badge(2) -- @turn_badge(3)]
            --
            Text("") CaseText::Briefing TextFont {font_size:px(13)} TextColor(PAPER)
            --
            Node {column_gap:px(8),flex_shrink:0.0}
            Children [@action_button("Investigate (1)",Action::Command(GameCommand::Investigate {})) -- @action_button("Sketch route (free)",Action::Toggle)]
            --
            Node {column_gap:px(8),flex_shrink:0.0}
            Children [@action_button("Undo (free)",Action::Command(GameCommand::PlanUndo {})) -- @action_button("Clear (free)",Action::Command(GameCommand::PlanClear {})) -- @action_button("Submit (1)",Action::Command(GameCommand::SubmitRoute {}))]
            --
            Text("") CaseText::Mode TextFont {font_size:px(12)} TextColor(GOLD)
            --
            Text("") CaseText::Progress TextFont {font_size:px(12)} TextColor(PAPER)
            --
            Text("") CaseText::Verdict TextFont {font_size:px(13)} TextColor(PAPER)
            --
            Text("") CaseText::Investigator TextFont {font_size:px(12)} TextColor(GOLD)
            --
            Node {flex_grow:1.0,min_height:px(0),overflow:Overflow::scroll_y(),padding:UiRect::right(px(8))}
            NotebookScroll
            Children [Text("") CaseText::Notebook TextFont {font_size:px(14)} TextColor(PAPER)]
            --
            Text("Move: Q W / A D / Z X   I: investigate
P: sketch/move   Enter: submit   E: end turn
Backspace: undo   ?: witness   R: reset")
            TextFont {font_size:px(10)} TextColor(MUTED)
        ]
    }
}

fn setup_ui(mut commands: Commands) {
    commands.queue_spawn_scene(panel());
}

fn case_keys(
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut mode: ResMut<PlanningMode>,
    session: Res<Session>,
) {
    if keys.just_pressed(KeyCode::KeyI) {
        commands.trigger(GameCommand::Investigate {});
    }
    if keys.just_pressed(KeyCode::KeyP) {
        commands.trigger(TogglePlanning);
    }
    if keys.just_pressed(KeyCode::Escape) {
        mode.0 = false;
    }
    if keys.just_pressed(KeyCode::Backspace) {
        commands.trigger(GameCommand::PlanUndo {});
    }
    if keys.just_pressed(KeyCode::KeyC) {
        commands.trigger(GameCommand::PlanClear {});
    }
    if keys.just_pressed(KeyCode::KeyE) && session.phase == TurnPhase::Playing {
        commands.trigger(GameCommand::EndTurn {});
    }
    if keys.just_pressed(KeyCode::Enter) {
        match session.phase {
            TurnPhase::Handoff => {
                commands.trigger(GameCommand::BeginTurn {});
            }
            TurnPhase::Playing => {
                commands.trigger(GameCommand::SubmitRoute {});
            }
            TurnPhase::Finished => {}
        }
    }
}

fn sync_case_ui(
    session: Res<Session>,
    mystery: Res<Mystery>,
    investigation: Res<Investigation>,
    mut mode: ResMut<PlanningMode>,
    mut texts: Query<(&CaseText, &mut Text, &mut TextColor)>,
    mut buttons: Query<(&TurnBadge, &mut BackgroundColor)>,
) {
    let briefing = mystery.briefing();
    let notebook = session.visible_notebook(&mystery, &investigation);
    if notebook.solved || session.phase != TurnPhase::Playing {
        mode.0 = false;
    }
    for (button, mut background) in &mut buttons {
        let desired = if button.0 == session.selected_player {
            Color::srgb(0.23, 0.32, 0.34)
        } else {
            Color::srgb(0.12, 0.21, 0.24)
        };
        if background.0 != desired {
            background.0 = desired;
        }
    }
    for (kind, mut text, mut color) in &mut texts {
        let value = match kind {
            CaseText::TurnTitle => match session.phase {
                TurnPhase::Playing => format!(
                    "PLAYER {} / {}'S TURN",
                    session.selected_player + 1,
                    PLAYER_NAMES[session.selected_player as usize].to_uppercase()
                ),
                TurnPhase::Handoff => format!(
                    "PASS TO PLAYER {} / {}",
                    session.selected_player + 1,
                    PLAYER_NAMES[session.selected_player as usize].to_uppercase()
                ),
                TurnPhase::Finished => format!(
                    "{} WINS",
                    PLAYER_NAMES[session.selected_player as usize].to_uppercase()
                ),
            },
            CaseText::TurnStatus => match session.phase {
                TurnPhase::Playing if session.actions_remaining == 0 => {
                    format!("Round {} / No actions left. End your turn.", session.round)
                }
                TurnPhase::Playing => format!(
                    "Round {} / {} of 2 actions left",
                    session.round, session.actions_remaining
                ),
                TurnPhase::Handoff => format!("Round {} / Private notebook hidden", session.round),
                TurnPhase::Finished => {
                    "Case closed. Start a new investigation to play again.".into()
                }
            },
            CaseText::TurnRules => match session.phase {
                TurnPhase::Playing => "YOUR TURN: spend up to 2 actions
Move 1 hex, investigate, or submit = 1 action.
Read and edit your sketch freely.
End turn to pass to the next investigator."
                    .into(),
                TurnPhase::Handoff => format!(
                    "Pass the device to {}.
Begin turn when they are ready to see their notes.",
                    PLAYER_NAMES[session.selected_player as usize]
                ),
                TurnPhase::Finished => {
                    "The first investigator to submit the exact route wins.".into()
                }
            },
            CaseText::Briefing => format!(
                "{}\nBank {},{} -> Airport {},{}\nExactly {} tiles including endpoints. No repeats.",
                briefing.title,
                briefing.bank.q,
                briefing.bank.r,
                briefing.airport.q,
                briefing.airport.r,
                briefing.route_length
            ),
            CaseText::Investigator if session.phase == TurnPhase::Handoff => format!(
                "{}'S NOTEBOOK / hidden until Begin turn",
                PLAYER_NAMES[session.selected_player as usize]
            ),
            CaseText::Investigator => format!(
                "{}'S NOTEBOOK / {} clues / scroll to read",
                PLAYER_NAMES[session.selected_player as usize],
                notebook.clues.len()
            ),
            CaseText::Mode => {
                if session.phase == TurnPhase::Handoff {
                    "HANDOFF / controls locked".into()
                } else if session.phase == TurnPhase::Finished {
                    "CASE CLOSED".into()
                } else if session.actions_remaining == 0 && !mode.0 {
                    "NO ACTIONS / sketch freely or end turn".into()
                } else if mode.0 {
                    "SKETCH MODE / click adjacent hexes".into()
                } else {
                    "MOVE MODE / visit ? witnesses".into()
                }
            }
            CaseText::Progress if session.phase == TurnPhase::Handoff => {
                "Begin turn to reveal your sketch and evidence.".into()
            }
            CaseText::Progress => format!(
                "Sketch: {} / {} tiles   Attempts: {}",
                notebook.plan.len(),
                briefing.route_length,
                notebook.attempts
            ),
            CaseText::Verdict if session.phase == TurnPhase::Handoff => String::new(),
            CaseText::Verdict => {
                let (value, tint) = match notebook.last_submission {
                    Some(Verdict::Correct) => (
                        "CASE SOLVED / Exact route found.",
                        Color::srgb(0.4, 0.85, 0.66),
                    ),
                    Some(Verdict::Incorrect) => (
                        "Incorrect route. Edit your sketch and retry.",
                        Color::srgb(0.98, 0.52, 0.46),
                    ),
                    None => ("Collect evidence, then submit your full route.", MUTED),
                };
                if color.0 != tint {
                    color.0 = tint;
                }
                value.into()
            }
            CaseText::Notebook => {
                if session.phase == TurnPhase::Handoff {
                    "Notebook hidden during handoff.".into()
                } else if session.phase == TurnPhase::Finished && notebook.clues.is_empty() {
                    "No clues recorded. The winning route is shown on the board.".into()
                } else if notebook.clues.is_empty() {
                    "Your notebook is empty.\n\nVisit a tile marked ? and investigate. Each investigator keeps their own notes.".into()
                } else {
                    notebook
                        .clues
                        .iter()
                        .enumerate()
                        .map(|(i, c)| {
                            format!(
                                "{}. {} / {},{}\n{}",
                                i + 1,
                                c.witness,
                                c.at.q,
                                c.at.r,
                                c.statement
                            )
                        })
                        .collect::<Vec<_>>()
                        .join("\n\n")
                }
            }
        };
        if text.0 != value {
            text.0 = value;
        }
    }
}

fn sync_action_buttons(
    case: CaseInteraction,
    board: Res<Board>,
    players: Query<(&robbers_types::Investigator, &Hex)>,
    mut buttons: Query<(
        Entity,
        &ButtonAction,
        &mut BackgroundColor,
        &mut BorderColor,
        Has<InteractionDisabled>,
    )>,
    mut labels: Query<(&ChildOf, &ButtonLabel, &mut Text, &mut TextColor)>,
    mut commands: Commands,
) {
    let session = &case.session;
    let notebook = &case.investigation.notebooks[session.selected_player as usize];
    let current = *players
        .iter()
        .find(|(p, _)| p.id == session.selected_player)
        .unwrap()
        .1;
    let mut enabled_states = Vec::new();
    for (entity, action, mut background, mut border, disabled) in &mut buttons {
        let enabled = match &action.0 {
            Action::Primary => true,
            _ if session.phase != TurnPhase::Playing => false,
            Action::Toggle => true,
            Action::Command(GameCommand::Investigate {}) => {
                session.can_spend_action()
                    && case
                        .mystery
                        .clues_at(current)
                        .iter()
                        .any(|c| !notebook.discovered.contains(&c.id))
            }
            Action::Command(GameCommand::SubmitRoute {}) => {
                session.can_spend_action()
                    && case.mystery.validate_route(&notebook.plan, &board).is_ok()
            }
            Action::Command(GameCommand::PlanUndo {} | GameCommand::PlanClear {}) => {
                !notebook.plan.is_empty()
            }
            Action::Command(_) => true,
        };
        enabled_states.push((entity, enabled));
        if enabled == disabled {
            if enabled {
                commands.entity(entity).remove::<InteractionDisabled>();
            } else {
                commands.entity(entity).insert(InteractionDisabled);
            }
        }
        let primary = matches!(action.0, Action::Primary);
        background.0 = if primary {
            Color::srgb(0.31, 0.29, 0.18)
        } else if enabled {
            Color::srgb(0.12, 0.21, 0.24)
        } else {
            Color::srgb(0.08, 0.14, 0.16)
        };
        *border = BorderColor::all(if primary {
            GOLD
        } else if enabled {
            MUTED
        } else {
            Color::srgb(0.20, 0.26, 0.27)
        });
    }
    for (parent, action, mut text, mut color) in &mut labels {
        let enabled = enabled_states
            .iter()
            .find(|(entity, _)| *entity == parent.parent())
            .is_some_and(|(_, enabled)| *enabled);
        color.0 = if enabled {
            PAPER
        } else {
            Color::srgb(0.37, 0.43, 0.43)
        };
        if matches!(action.0, Action::Primary) {
            let label = match session.phase {
                TurnPhase::Playing if session.actions_remaining > 0 => {
                    format!("End turn / E ({} unused)", session.actions_remaining)
                }
                TurnPhase::Playing => "End turn / E".into(),
                TurnPhase::Handoff => format!(
                    "Begin {}'s turn / Enter",
                    PLAYER_NAMES[session.selected_player as usize]
                ),
                TurnPhase::Finished => "Start a new investigation".into(),
            };
            if text.0 != label {
                text.0 = label;
            }
        }
    }
}

fn scroll_notebook(
    mut wheel: MessageReader<MouseWheel>,
    windows: Query<&Window>,
    ui_scale: Res<UiScale>,
    session: Res<Session>,
    mut last_player: Local<Option<u8>>,
    mut panels: Query<(&ComputedNode, &mut ScrollPosition), With<NotebookScroll>>,
) {
    if *last_player != Some(session.selected_player) {
        for (_, mut position) in &mut panels {
            position.0.y = 0.0;
        }
        *last_player = Some(session.selected_player);
    }
    let over_panel = windows.single().ok().is_some_and(|w| {
        w.cursor_position()
            .is_some_and(|p| p.x > w.width() - 410.0 * ui_scale.0)
    });
    for event in wheel.read() {
        if !over_panel {
            continue;
        }
        let delta = if event.unit == MouseScrollUnit::Line {
            -event.y * 40.0
        } else {
            -event.y
        };
        for (computed, mut position) in &mut panels {
            let max = ((computed.content_size().y - computed.size().y)
                * computed.inverse_scale_factor())
            .max(0.0);
            position.0.y = (position.0.y + delta).clamp(0.0, max);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;
    use robbers_command::{CommandPlugin, dispatch};
    use robbers_game::{GamePlugin, snapshot};

    fn app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, GamePlugin, CommandPlugin))
            .init_resource::<PlanningMode>()
            .init_resource::<ButtonInput<KeyCode>>();
        app
    }

    #[test]
    fn keyboard_directions_sketch_in_planning_mode_and_move_pawns_in_move_mode() {
        let mut app = app();
        let initial = snapshot(app.world_mut());
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyD);
        app.world_mut()
            .run_system_once(crate::keyboard_input)
            .unwrap();
        app.world_mut().flush();
        let moved = snapshot(app.world_mut());
        assert_ne!(moved.players[0], initial.players[0]);
        dispatch(app.world_mut(), GameCommand::PlanStart {}).unwrap();
        app.world_mut().resource_mut::<PlanningMode>().0 = true;
        app.world_mut()
            .run_system_once(crate::keyboard_input)
            .unwrap();
        app.world_mut().flush();
        let planned = snapshot(app.world_mut());
        assert_eq!(planned.players, moved.players);
        assert_eq!(
            planned.notebook.plan,
            vec![Hex::new(-3, 1), Hex::new(-2, 1)]
        );
    }

    #[test]
    fn planning_clicks_restart_at_the_bank_after_clearing_the_sketch() {
        let mut app = app();
        app.world_mut().resource_mut::<PlanningMode>().0 = true;
        let command = app
            .world_mut()
            .run_system_once(|case: CaseInteraction| case.tile_command(Hex::new(-3, 1)))
            .unwrap();
        dispatch(app.world_mut(), command).unwrap();
        assert_eq!(snapshot(app.world_mut()).notebook.plan.len(), 1);
        dispatch(app.world_mut(), GameCommand::PlanClear {}).unwrap();
        let command = app
            .world_mut()
            .run_system_once(|case: CaseInteraction| case.tile_command(Hex::new(-3, 1)))
            .unwrap();
        dispatch(app.world_mut(), command).unwrap();
        assert_eq!(snapshot(app.world_mut()).notebook.plan.len(), 1);
    }

    #[test]
    fn old_selection_shortcuts_cannot_change_the_player_mid_turn() {
        let mut app = app();
        let initial = snapshot(app.world_mut());
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Digit2);
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Tab);
        app.world_mut()
            .run_system_once(crate::keyboard_input)
            .unwrap();
        app.world_mut().flush();
        assert_eq!(snapshot(app.world_mut()), initial);
    }

    #[test]
    fn view_adapter_hides_private_notes_during_handoff() {
        let mut app = app();
        dispatch(app.world_mut(), GameCommand::Investigate {}).unwrap();
        for _ in 0..3 {
            dispatch(app.world_mut(), GameCommand::EndTurn {}).unwrap();
            dispatch(app.world_mut(), GameCommand::BeginTurn {}).unwrap();
        }
        dispatch(app.world_mut(), GameCommand::EndTurn {}).unwrap();
        let hidden = app
            .world_mut()
            .run_system_once(|case: CaseInteraction| case.notebook())
            .unwrap();
        assert!(hidden.clues.is_empty());
        dispatch(app.world_mut(), GameCommand::BeginTurn {}).unwrap();
        let revealed = app
            .world_mut()
            .run_system_once(|case: CaseInteraction| case.notebook())
            .unwrap();
        assert_eq!(revealed.clues.len(), 1);
    }
}
