//! Mesh fills and rims share cached geometry; player order matches pawn placement.
use crate::{
    Pawn,
    hex_mesh::{HexMeshes, OverlayMaterials},
    layout::BoardRoot,
    position,
};
use bevy::prelude::*;
use robbers_types::{Hex, Investigator};

#[derive(Component)]
struct OccupancyPart {
    player: u8,
    rim: bool,
}

pub(super) struct OccupancyPlugin;
impl Plugin for OccupancyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_overlays.after(crate::setup))
            .add_systems(
                PostUpdate,
                (sync_overlays, sync_pawns)
                    .chain_weak()
                    .before(TransformSystems::Propagate),
            );
    }
}

fn spawn_overlays(
    mut commands: Commands,
    roots: Query<Entity, With<BoardRoot>>,
    players: Query<&Investigator>,
    meshes: Res<HexMeshes>,
    materials: Res<OverlayMaterials>,
) {
    let root = roots.single().expect("board root exists after setup");
    for player in &players {
        for rim in [false, true] {
            let sector = &meshes.sectors[1][0];
            let (mesh, material) = if rim {
                (&sector.rim, &materials.rims[player.id as usize])
            } else {
                (&sector.fill, &materials.fills[player.id as usize])
            };
            commands.spawn((
                OccupancyPart {
                    player: player.id,
                    rim,
                },
                Mesh2d(mesh.clone()),
                MeshMaterial2d(material.clone()),
                Transform::default(),
                Pickable::IGNORE,
                ChildOf(root),
            ));
        }
    }
}

fn sync_overlays(
    players: Query<(&Investigator, &Hex)>,
    meshes: Res<HexMeshes>,
    mut overlays: Query<(&OccupancyPart, &mut Mesh2d, &mut Transform)>,
) {
    for (part, mut mesh, mut transform) in &mut overlays {
        let hex = *players.iter().find(|(p, _)| p.id == part.player).unwrap().1;
        let mut occupants: Vec<_> = players
            .iter()
            .filter(|(_, h)| **h == hex)
            .map(|(p, _)| p.id)
            .collect();
        occupants.sort_unstable();
        let index = occupants.iter().position(|id| *id == part.player).unwrap();
        let sector = &meshes.sectors[occupants.len()][index];
        let desired = if part.rim { &sector.rim } else { &sector.fill };
        if mesh.0 != *desired {
            mesh.0 = desired.clone();
        }
        transform.translation = position(hex).extend(if part.rim { 0.6 } else { 0.5 });
    }
}

fn sync_pawns(mut players: Query<(&Investigator, &Hex, &mut Transform, &mut Sprite), With<Pawn>>) {
    let positions: Vec<_> = players.iter().map(|(p, h, _, _)| (p.id, *h)).collect();
    for (player, hex, mut transform, mut sprite) in &mut players {
        let mut occupants: Vec<_> = positions
            .iter()
            .filter(|(_, h)| h == hex)
            .map(|(id, _)| *id)
            .collect();
        occupants.sort_unstable();
        let offset = if occupants.len() == 1 {
            Vec2::new(0.0, 5.0)
        } else {
            let index = occupants.iter().position(|id| *id == player.id).unwrap();
            let angle = std::f32::consts::FRAC_PI_2
                - (index as f32 + 0.5) * std::f32::consts::TAU / occupants.len() as f32;
            Vec2::new(angle.cos(), angle.sin()) * 25.0
        };
        transform.translation = (position(*hex) + offset).extend(5.0 + player.id as f32 * 0.01);
        let size = if occupants.len() == 1 {
            Vec2::new(34.0, 43.0)
        } else {
            Vec2::new(29.0, 37.0)
        };
        if sprite.custom_size != Some(size) {
            sprite.custom_size = Some(size);
        }
    }
}
