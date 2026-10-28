//! Graphical presentation and human input. Gameplay lives in robbers-command.
mod hex_mesh;
mod highlights;
mod layout;
mod occupancy;
use bevy::{
    picking::pointer::PointerButton,
    prelude::*,
    render::view::screenshot::{Screenshot, ScreenshotCaptured, save_to_disk},
    ui_widgets::Button,
};
use layout::{BoardLayout, BoardLayoutPlugin, BoardRoot, RADIUS};
use robbers_command::{Feedback, GameCommand};
use robbers_game::{Board, Session};
use robbers_types::*;

const INK: Color = Color::srgb(0.075, 0.145, 0.17);
const PAPER: Color = Color::srgb(0.94, 0.90, 0.80);
const MUTED: Color = Color::srgb(0.58, 0.68, 0.67);
const GOLD: Color = Color::srgb(0.89, 0.74, 0.45);

#[derive(Resource, Default)]
struct Hovered(Option<Hex>);
#[derive(Component, Default, Clone)]
struct Pawn;
#[derive(Component, Default, Clone)]
struct PlayerCard(u8);
#[derive(Component, Default, Clone)]
enum HudText {
    Player(u8),
    #[default]
    Status,
    Counter,
    Hover,
}

pub struct ViewPlugin;
impl Plugin for ViewPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(INK))
            .init_resource::<Hovered>()
            .add_plugins((
                BoardLayoutPlugin,
                hex_mesh::HexMeshPlugin,
                occupancy::OccupancyPlugin,
                highlights::HighlightPlugin,
            ))
            .add_systems(Startup, setup)
            .add_systems(Update, keyboard_input)
            .add_systems(PostUpdate, sync_ui);
    }
}

fn position(hex: Hex) -> Vec2 {
    hex.world_position(RADIUS)
}

fn setup(
    mut commands: Commands,
    assets: Res<AssetServer>,
    layout: Res<BoardLayout>,
    tiles: Query<(Entity, &Hex, &TileKind)>,
    players: Query<(Entity, &Investigator)>,
) {
    commands.spawn(Camera2d);
    let root = commands
        .spawn((
            BoardRoot,
            Transform::from_translation(layout.center.extend(0.0))
                .with_scale(Vec3::splat(layout.scale)),
            Visibility::default(),
        ))
        .id();
    for (entity, hex, kind) in &tiles {
        commands
            .entity(entity)
            .insert((
                Sprite {
                    image: assets.load(format!("tiles/{}.png", kind.slug())),
                    custom_size: Some(Vec2::splat(RADIUS * 2.0)),
                    ..default()
                },
                Transform::from_translation(position(*hex).extend(0.0)),
                Pickable::default(),
                ChildOf(root),
            ))
            .observe(tile_click)
            .observe(tile_hover)
            .observe(tile_leave);
        commands.spawn((
            Text2d::new(format!("{},{}", hex.q, hex.r)),
            TextFont {
                font_size: px(10).into(),
                ..default()
            },
            TextColor(INK),
            Transform::from_translation((position(*hex) + Vec2::new(0.0, 42.0)).extend(1.0)),
            Pickable::IGNORE,
            ChildOf(root),
        ));
    }
    for (entity, player) in &players {
        commands
            .entity(entity)
            .insert((
                Pawn,
                Sprite {
                    image: assets.load(format!("pawns/{}.png", player.id)),
                    custom_size: Some(Vec2::new(29.0, 37.0)),
                    ..default()
                },
                Transform::from_xyz(0.0, 0.0, 5.0),
                Pickable::default(),
                ChildOf(root),
            ))
            .observe(pawn_click);
    }
    // Bevy 0.20 BSN: @scene inclusions and -- child separators.
    commands.queue_spawn_scene(header());
    commands.queue_spawn_scene(sidebar());
    commands.queue_spawn_scene(footer());
    for id in 0..4 {
        commands.queue_spawn_scene(player_card(id));
    }
}

fn header() -> impl Scene {
    bsn! {
        Node { position_type: PositionType::Absolute, left: px(40), top: px(26),
            flex_direction: FlexDirection::Column, row_gap: px(5) }
        Pickable::IGNORE
        Children [
            Text("R O B B E R S")
            TextFont { font_size: px(38) }
            TextColor(PAPER)
            --
            Text("CITY PATROL   /   MOVEMENT STUDY 01")
            TextFont { font_size: px(12) }
            TextColor(GOLD)
        ]
    }
}

fn sidebar() -> impl Scene {
    bsn! {
        Node { position_type: PositionType::Absolute, right: px(34), top: px(32), width: px(280),
            flex_direction: FlexDirection::Column, row_gap: px(10) }
        Children [
            Text("THE CITY IS YOURS")
            TextFont { font_size: px(12) }
            TextColor(GOLD)
            --
            Text("37 blocks. Six directions.")
            TextFont { font_size: px(20) }
            TextColor(PAPER)
            --
            Text("Choose an investigator, then click a neighboring hex to move.")
            TextFont { font_size: px(15) }
            TextColor(MUTED)
            --
            Node { height: px(376) }
            --
            Text("YOUR NEXT STEP")
            TextFont { font_size: px(12) }
            TextColor(GOLD)
            --
            Text("Q  NW       W  NE\nA  W         D  E\nZ  SW       X  SE\n\n1 - 4       select investigator\nTab         next investigator\nR             reset patrol")
            TextFont { font_size: px(15) }
            TextColor(PAPER)
            --
            @reset_button()
            --
            Text("Free exploration. All tiles are walkable; investigators may share a block.")
            TextFont { font_size: px(13) }
            TextColor(MUTED)
        ]
    }
}

fn reset_button() -> impl Scene {
    bsn! {
        Button
        Node { margin: UiRect::vertical(px(8)), padding: UiRect::all(px(12)),
            border: px(1), border_radius: BorderRadius::all(px(6)), justify_content: JustifyContent::Center }
        BorderColor::all(MUTED)
        BackgroundColor(Color::srgb(0.12, 0.21, 0.24))
        on(|event: On<PointerClick>, mut commands: Commands| {
            if event.button == PointerButton::Primary { commands.trigger(GameCommand::Reset {}); }
        })
        Children [ Text("Reset patrol  /  R") TextFont { font_size: px(14) } TextColor(PAPER) ]
    }
}

fn player_card(id: u8) -> impl Scene {
    let color = Color::srgb_from_array(PLAYER_COLORS[id as usize]);
    let name = PLAYER_NAMES[id as usize];
    bsn! {
        Button
        PlayerCard(id)
        Node { position_type: PositionType::Absolute, right: px(34), top: px(184 + id as u32 * 84),
            width: px(280), height: px(72), padding: UiRect::all(px(14)),
            border: px(1), border_radius: BorderRadius::all(px(8)), align_items: AlignItems::Center,
            column_gap: px(14) }
        BackgroundColor(Color::srgb(0.11, 0.19, 0.22))
        BorderColor::all(MUTED)
        on(move |event: On<PointerClick>, mut commands: Commands| {
            if event.button == PointerButton::Primary { commands.trigger(GameCommand::Select { player: id }); }
        })
        Children [
            Node { width: px(10), height: px(34), border_radius: BorderRadius::all(px(5)) }
            BackgroundColor(color)
            --
            Text(name)
            HudText::Player(id)
            TextFont { font_size: px(16) }
            TextColor(PAPER)
        ]
    }
}

fn footer() -> impl Scene {
    bsn! {
        Node { position_type: PositionType::Absolute, left: px(40), bottom: px(25),
            flex_direction: FlexDirection::Column, row_gap: px(7) }
        Pickable::IGNORE
        Children [
            Text("HOVER A BLOCK TO IDENTIFY IT") HudText::Hover
            TextFont { font_size: px(12) } TextColor(GOLD)
            --
            Text("Explore the city. The highlighted hexes are your possible moves.") HudText::Status
            TextFont { font_size: px(15) } TextColor(PAPER)
            --
            Text("0 MOVES  /  PROTOTYPE") HudText::Counter
            TextFont { font_size: px(11) } TextColor(MUTED)
        ]
    }
}

fn tile_click(event: On<PointerClick>, tiles: Query<&Hex, With<TileKind>>, mut commands: Commands) {
    if event.button == PointerButton::Primary
        && let Ok(hex) = tiles.get(event.entity)
    {
        commands.trigger(GameCommand::Step { q: hex.q, r: hex.r });
    }
}
fn tile_hover(
    event: On<PointerOver>,
    tiles: Query<&Hex, With<TileKind>>,
    mut hovered: ResMut<Hovered>,
) {
    if let Ok(hex) = tiles.get(event.entity) {
        hovered.0 = Some(*hex);
    }
}
fn tile_leave(
    event: On<PointerOut>,
    tiles: Query<&Hex, With<TileKind>>,
    mut hovered: ResMut<Hovered>,
) {
    if let Ok(hex) = tiles.get(event.entity)
        && hovered.0 == Some(*hex)
    {
        hovered.0 = None;
    }
}
fn pawn_click(event: On<PointerClick>, players: Query<&Investigator>, mut commands: Commands) {
    if event.button == PointerButton::Primary
        && let Ok(player) = players.get(event.entity)
    {
        commands.trigger(GameCommand::Select { player: player.id });
    }
}
fn keyboard_input(keys: Res<ButtonInput<KeyCode>>, session: Res<Session>, mut commands: Commands) {
    for (key, direction) in [
        (KeyCode::KeyD, Direction::E),
        (KeyCode::KeyW, Direction::Ne),
        (KeyCode::KeyQ, Direction::Nw),
        (KeyCode::KeyA, Direction::W),
        (KeyCode::KeyZ, Direction::Sw),
        (KeyCode::KeyX, Direction::Se),
    ] {
        if keys.just_pressed(key) {
            commands.trigger(GameCommand::Move { direction });
        }
    }
    for (id, key) in [
        KeyCode::Digit1,
        KeyCode::Digit2,
        KeyCode::Digit3,
        KeyCode::Digit4,
    ]
    .into_iter()
    .enumerate()
    {
        if keys.just_pressed(key) {
            commands.trigger(GameCommand::Select { player: id as u8 });
        }
    }
    if keys.just_pressed(KeyCode::Tab) {
        commands.trigger(GameCommand::Select {
            player: (session.selected_player + 1) % 4,
        });
    }
    if keys.just_pressed(KeyCode::KeyR) {
        commands.trigger(GameCommand::Reset {});
    }
}

fn sync_ui(
    session: Res<Session>,
    feedback: Res<Feedback>,
    hovered: Res<Hovered>,
    board: Res<Board>,
    players: Query<(&Investigator, &Hex)>,
    mut cards: Query<(&PlayerCard, &mut BorderColor, &mut BackgroundColor)>,
    mut texts: Query<(&mut Text, &HudText)>,
) {
    for (card, mut border, mut background) in &mut cards {
        let selected = session.selected_player == card.0;
        *border = BorderColor::all(if selected {
            GOLD
        } else {
            Color::srgb(0.22, 0.32, 0.34)
        });
        background.0 = if selected {
            Color::srgb(0.17, 0.26, 0.28)
        } else {
            Color::srgb(0.10, 0.18, 0.21)
        };
    }
    for (mut text, kind) in &mut texts {
        let value = match kind {
            HudText::Player(id) => {
                let (_, hex) = players.iter().find(|(p, _)| p.id == *id).unwrap();
                format!(
                    "{}  /  {}\n{},{}  /  {}",
                    id + 1,
                    PLAYER_NAMES[*id as usize],
                    hex.q,
                    hex.r,
                    if *id == session.selected_player {
                        "SELECTED"
                    } else {
                        "READY"
                    }
                )
            }
            HudText::Status => {
                if feedback.0.is_empty() {
                    "Explore the city. Highlighted hexes are your possible moves.".into()
                } else {
                    feedback.0.clone()
                }
            }
            HudText::Counter => {
                format!(
                    "{} MOVES   /   FREE EXPLORATION   /   REV {}",
                    session.moves, session.revision
                )
            }
            HudText::Hover => hovered
                .0
                .and_then(|hex| {
                    board.kind(hex).map(|kind| {
                        format!("{}   /   {},{}", kind.label().to_uppercase(), hex.q, hex.r)
                    })
                })
                .unwrap_or_else(|| "HOVER A BLOCK TO IDENTIFY IT".into()),
        };
        if text.0 != value {
            text.0 = value;
        }
    }
}

#[derive(Resource)]
struct Capture {
    path: String,
    frames: u32,
}
pub fn capture_and_exit(app: &mut App, path: String) {
    app.insert_resource(Capture { path, frames: 0 })
        .add_systems(Update, capture_frame);
}
fn capture_frame(
    mut capture: ResMut<Capture>,
    assets: Res<AssetServer>,
    sprites: Query<&Sprite>,
    mut commands: Commands,
) {
    if capture.frames == u32::MAX {
        return;
    }
    capture.frames += 1;
    if capture.frames < 30
        || !sprites
            .iter()
            .all(|s| assets.is_loaded_with_dependencies(s.image.id()))
    {
        return;
    }
    capture.frames = u32::MAX;
    commands
        .spawn(Screenshot::primary_window())
        .observe(save_to_disk(capture.path.clone()))
        .observe(
            |_: On<ScreenshotCaptured>, mut exit: MessageWriter<AppExit>| {
                exit.write(AppExit::Success);
            },
        );
}
