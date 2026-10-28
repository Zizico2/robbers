//! Persistent, non-interactive mesh rings in the same coordinate space as tiles.
use crate::{
    Hovered,
    hex_mesh::{HexMeshes, OverlayMaterials},
    layout::BoardRoot,
    position,
};
use bevy::prelude::*;
use robbers_game::{Board, Session};
use robbers_types::{Direction, Hex, Investigator};

#[derive(Component)]
enum Highlight {
    Selected,
    Legal(usize),
    Hovered,
}

pub(super) struct HighlightPlugin;
impl Plugin for HighlightPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_highlights.after(crate::setup))
            .add_systems(
                PostUpdate,
                sync_highlights.before(TransformSystems::Propagate),
            );
    }
}

fn spawn_highlights(
    mut commands: Commands,
    roots: Query<Entity, With<BoardRoot>>,
    meshes: Res<HexMeshes>,
    materials: Res<OverlayMaterials>,
) {
    let root = roots.single().expect("board root exists after setup");
    for kind in [Highlight::Selected, Highlight::Hovered]
        .into_iter()
        .chain((0..6).map(Highlight::Legal))
    {
        let (mesh, material) = match kind {
            Highlight::Selected => (&meshes.selected, &materials.selected),
            Highlight::Legal(_) => (&meshes.legal, &materials.legal),
            Highlight::Hovered => (&meshes.hovered, &materials.hovered),
        };
        commands.spawn((
            kind,
            Mesh2d(mesh.clone()),
            MeshMaterial2d(material.clone()),
            Transform::default(),
            Visibility::Hidden,
            Pickable::IGNORE,
            ChildOf(root),
        ));
    }
}

fn sync_highlights(
    session: Res<Session>,
    board: Res<Board>,
    hovered: Res<Hovered>,
    players: Query<(&Investigator, &Hex)>,
    mut highlights: Query<(&Highlight, &mut Transform, &mut Visibility)>,
) {
    let current = *players
        .iter()
        .find(|(p, _)| p.id == session.selected_player)
        .unwrap()
        .1;
    for (kind, mut transform, mut visibility) in &mut highlights {
        let hex = match *kind {
            Highlight::Selected => Some(current),
            Highlight::Hovered => hovered.0,
            Highlight::Legal(index) => {
                let hex = current.neighbor(Direction::ALL[index]);
                board.contains(hex).then_some(hex)
            }
        };
        let desired = if hex.is_some() {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if *visibility != desired {
            *visibility = desired;
        }
        if let Some(hex) = hex {
            transform.translation = position(hex).extend(1.5);
        }
    }
}
