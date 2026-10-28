//! Case controls and notebook. Every gameplay action uses the shared command API.
use crate::{GOLD, MUTED, PAPER};
use bevy::{
    ecs::system::SystemParam,
    input::mouse::{MouseScrollUnit, MouseWheel},
    picking::pointer::PointerButton,
    prelude::*,
    ui_widgets::Button,
};
use robbers_command::GameCommand;
use robbers_game::{Board, Investigation, Mystery, Session, Verdict};
use robbers_types::{Direction, Hex, PLAYER_COLORS, PLAYER_NAMES};

#[derive(Resource, Default)]
pub(super) struct PlanningMode(pub bool);
#[derive(Event)]
struct TogglePlanning;
#[derive(Component, Clone, Default)]
struct InvestigatorButton(u8);
#[derive(Component, Clone, Default)]
enum CaseText {
    #[default]
    Briefing,
    Progress,
    Notebook,
    Verdict,
    Mode,
    Investigator,
}
#[derive(Component, Clone, Default)]
struct NotebookScroll;
#[derive(Clone)]
enum Action {
    Command(GameCommand),
    Toggle,
}

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
    pub fn selected(&self) -> u8 {
        self.session.selected_player
    }
    pub fn plan(&self) -> &[Hex] {
        &self.investigation.notebooks[self.selected() as usize].plan
    }
    pub fn notebook(&self) -> robbers_game::NotebookSnapshot {
        self.mystery
            .notebook_snapshot(&self.investigation.notebooks[self.selected() as usize])
    }
    pub fn plan_destinations(&self, board: &Board) -> Vec<Hex> {
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
                if matches!(event.event(), GameCommand::Reset {}) {
                    mode.0 = false;
                }
            })
            .add_systems(Startup, setup_ui)
            .add_systems(Update, (case_keys, scroll_notebook))
            .add_systems(PostUpdate, sync_case_ui);
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
    if notebook.solved {
        mode.0 = false;
        return;
    }
    mode.0 = !mode.0;
    if mode.0 && notebook.plan.is_empty() {
        commands.trigger(GameCommand::PlanStart {});
    }
}

fn action_button(label: &str, action: Action) -> impl Scene {
    bsn! {
        Button
        Node { flex_grow: 1.0, flex_basis: px(0), min_height: px(34), padding: UiRect::all(px(8)),
            border: px(1), border_radius: BorderRadius::all(px(5)), justify_content: JustifyContent::Center, align_items: AlignItems::Center }
        BorderColor::all(MUTED)
        BackgroundColor(Color::srgb(0.12,0.21,0.24))
        on(move |event: On<PointerClick>, mut commands: Commands| {
            if event.button == PointerButton::Primary { match &action {
                Action::Command(command) => { commands.trigger(command.clone()); }
                Action::Toggle => { commands.trigger(TogglePlanning); }
            }}
        })
        Children [ Text(label) TextFont { font_size: px(13) } TextColor(PAPER) ]
    }
}

fn investigator_button(id: u8) -> impl Scene {
    let label = format!("{}", id + 1);
    let color = Color::srgb_from_array(PLAYER_COLORS[id as usize]);
    bsn! {
        @action_button(&label, Action::Command(GameCommand::Select {player:id}))
        InvestigatorButton(id)
        BorderColor::all(color)
    }
}

fn panel() -> impl Scene {
    bsn! {
        Node { position_type: PositionType::Absolute, right: px(24), top: px(20), bottom: px(20), width: px(360),
            flex_direction: FlexDirection::Column, row_gap: px(9) }
        Children [
            Text("CASE FILE  /  01") TextFont { font_size: px(12) } TextColor(GOLD)
            --
            Text("") CaseText::Briefing TextFont { font_size: px(14) } TextColor(PAPER)
            --
            Node { column_gap: px(8), flex_shrink: 0.0 }
            Children [ @investigator_button(0) -- @investigator_button(1) -- @investigator_button(2) -- @investigator_button(3) ]
            --
            Text("") CaseText::Investigator TextFont { font_size: px(14) } TextColor(GOLD)
            --
            Node { column_gap: px(8), flex_shrink: 0.0 }
            Children [ @action_button("Investigate / I", Action::Command(GameCommand::Investigate {}))
                -- @action_button("Sketch / P", Action::Toggle) ]
            --
            Node { column_gap: px(8), flex_shrink: 0.0 }
            Children [ @action_button("Undo", Action::Command(GameCommand::PlanUndo {}))
                -- @action_button("Clear", Action::Command(GameCommand::PlanClear {}))
                -- @action_button("Submit / Enter", Action::Command(GameCommand::SubmitRoute {})) ]
            --
            Text("") CaseText::Mode TextFont { font_size: px(13) } TextColor(GOLD)
            --
            Text("") CaseText::Progress TextFont { font_size: px(13) } TextColor(PAPER)
            --
            Text("") CaseText::Verdict TextFont { font_size: px(14) } TextColor(PAPER)
            --
            Text("NOTEBOOK  /  SCROLL TO READ") TextFont { font_size: px(12) } TextColor(GOLD)
            --
            Node { flex_grow:1.0, min_height:px(0), overflow:Overflow::scroll_y(), padding:UiRect::right(px(8)) }
            NotebookScroll
            Children [ Text("") CaseText::Notebook TextFont { font_size: px(14) } TextColor(PAPER) ]
            --
            Text("Move: Q W / A D / Z X    Select: 1-4\nP: sketch/move   Esc: move   Backspace: undo\n?: witness   Gold numbers: your route sketch")
            TextFont { font_size: px(11) } TextColor(MUTED)
            --
            @action_button("Reset investigation / R", Action::Command(GameCommand::Reset {}))
            Node { flex_grow:0.0, flex_basis:Val::Auto, flex_shrink:0.0 }
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
    if keys.just_pressed(KeyCode::Enter) {
        commands.trigger(GameCommand::SubmitRoute {});
    }
}

fn sync_case_ui(
    session: Res<Session>,
    mystery: Res<Mystery>,
    investigation: Res<Investigation>,
    mut mode: ResMut<PlanningMode>,
    mut texts: Query<(&CaseText, &mut Text, &mut TextColor)>,
    mut buttons: Query<(&InvestigatorButton, &mut BackgroundColor)>,
) {
    let briefing = mystery.briefing();
    let notebook =
        mystery.notebook_snapshot(&investigation.notebooks[session.selected_player as usize]);
    if notebook.solved {
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
            CaseText::Briefing => format!(
                "{}\n\n{}\n\nBank {},{} -> Airport {},{}\nExactly {} tiles, including both endpoints. No repeated tiles.",
                briefing.title,
                briefing.briefing,
                briefing.bank.q,
                briefing.bank.r,
                briefing.airport.q,
                briefing.airport.r,
                briefing.route_length
            ),
            CaseText::Investigator => format!(
                "{} / {} clues recorded",
                PLAYER_NAMES[session.selected_player as usize],
                notebook.clues.len()
            ),
            CaseText::Mode => {
                if mode.0 {
                    "SKETCH MODE / click adjacent hexes".into()
                } else {
                    "MOVE MODE / visit ? witnesses".into()
                }
            }
            CaseText::Progress => format!(
                "Sketch: {} / {} tiles   Attempts: {}",
                notebook.plan.len(),
                briefing.route_length,
                notebook.attempts
            ),
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
                if notebook.clues.is_empty() {
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
}
