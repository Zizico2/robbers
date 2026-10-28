//! Witness markers and the investigator's numbered route sketch, all board-local.
use crate::{GOLD, INK, PAPER, layout::BoardRoot, position};
use bevy::prelude::*;
use robbers_game::Mystery;
use robbers_types::Hex;

#[derive(Component)]
struct Witness(Hex);
#[derive(Component)]
struct WitnessLabel(Hex);
#[derive(Component)]
struct RoutePoint(usize);
#[derive(Component)]
struct RouteSegment(usize);
#[derive(Resource)]
struct WitnessMaterials {
    pending: Handle<ColorMaterial>,
    recorded: Handle<ColorMaterial>,
}

pub(super) struct CaseBoardPlugin;
impl Plugin for CaseBoardPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup.after(crate::setup))
            .add_systems(PostUpdate, sync.before(TransformSystems::Propagate));
    }
}

fn setup(
    mut commands: Commands,
    roots: Query<Entity, With<BoardRoot>>,
    mystery: Res<Mystery>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let root = roots.single().expect("board root exists");
    let circle = meshes.add(Circle::new(9.0));
    let line = meshes.add(Rectangle::new(1.0, 1.0));
    let gold = materials.add(ColorMaterial::from_color(GOLD));
    let paper = materials.add(ColorMaterial::from_color(PAPER));
    let recorded = materials.add(ColorMaterial::from_color(Color::srgb(0.35, 0.65, 0.54)));
    let briefing = mystery.briefing();
    for hex in briefing.clue_sites {
        let marker = commands
            .spawn((
                Witness(hex),
                Mesh2d(circle.clone()),
                MeshMaterial2d(gold.clone()),
                Transform::from_translation((position(hex) + Vec2::new(-32.0, -20.0)).extend(3.0)),
                Pickable::IGNORE,
                ChildOf(root),
            ))
            .id();
        commands.spawn((
            WitnessLabel(hex),
            Text2d::new("?"),
            TextFont {
                font_size: px(14).into(),
                ..default()
            },
            TextColor(INK),
            Transform::from_xyz(0.0, 0.0, 0.1),
            Pickable::IGNORE,
            ChildOf(marker),
        ));
    }
    for index in 0..briefing.route_length {
        let badge = commands
            .spawn((
                RoutePoint(index),
                Mesh2d(circle.clone()),
                MeshMaterial2d(paper.clone()),
                Transform::default(),
                Visibility::Hidden,
                Pickable::IGNORE,
                ChildOf(root),
            ))
            .id();
        commands.spawn((
            Text2d::new((index + 1).to_string()),
            TextFont {
                font_size: px(12).into(),
                ..default()
            },
            TextColor(INK),
            Transform::from_xyz(0.0, 0.0, 0.1),
            Pickable::IGNORE,
            ChildOf(badge),
        ));
        if index + 1 < briefing.route_length {
            commands.spawn((
                RouteSegment(index),
                Mesh2d(line.clone()),
                MeshMaterial2d(gold.clone()),
                Transform::default(),
                Visibility::Hidden,
                Pickable::IGNORE,
                ChildOf(root),
            ));
        }
    }
    commands.insert_resource(WitnessMaterials {
        pending: gold,
        recorded,
    });
}

fn sync(
    case: crate::case_ui::CaseInteraction,
    materials: Res<WitnessMaterials>,
    mut witnesses: Query<(&Witness, &mut MeshMaterial2d<ColorMaterial>)>,
    mut labels: Query<(&WitnessLabel, &mut Text2d)>,
    mut points: Query<(&RoutePoint, &mut Transform, &mut Visibility), Without<RouteSegment>>,
    mut segments: Query<(&RouteSegment, &mut Transform, &mut Visibility), Without<RoutePoint>>,
) {
    let notebook = case.notebook();
    let recorded = |hex| notebook.clues.iter().any(|c| c.at == hex);
    for (witness, mut material) in &mut witnesses {
        let desired = if recorded(witness.0) {
            &materials.recorded
        } else {
            &materials.pending
        };
        if material.0 != *desired {
            material.0 = desired.clone();
        }
    }
    for (label, mut text) in &mut labels {
        let desired = if recorded(label.0) { "+" } else { "?" };
        if text.0 != desired {
            text.0 = desired.into();
        }
    }
    let route_position = |hex| position(hex) + Vec2::new(0.0, 22.0);
    for (point, mut transform, mut visibility) in &mut points {
        *visibility = if let Some(hex) = notebook.plan.get(point.0) {
            transform.translation = route_position(*hex).extend(2.4);
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
    for (segment, mut transform, mut visibility) in &mut segments {
        *visibility = if let Some(pair) = notebook.plan.get(segment.0..segment.0 + 2) {
            let a = route_position(pair[0]);
            let b = route_position(pair[1]);
            let delta = b - a;
            transform.translation = ((a + b) / 2.0).extend(2.0);
            transform.rotation = Quat::from_rotation_z(delta.y.atan2(delta.x));
            transform.scale = Vec3::new(delta.length(), 2.5, 1.0);
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}
