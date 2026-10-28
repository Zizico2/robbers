//! Cached hex sectors and rings. All vertices use board-local coordinates.
//! Geometry is built once; movement only changes handles and transforms.
use crate::{GOLD, PAPER, layout::RADIUS};
use bevy::{asset::RenderAssetUsages, mesh::PrimitiveTopology, prelude::*};
use robbers_types::PLAYER_COLORS;
use std::f32::consts::{FRAC_PI_3, TAU};

pub(super) struct SectorMeshes {
    pub fill: Handle<Mesh>,
    pub rim: Handle<Mesh>,
}

#[derive(Resource)]
pub(super) struct HexMeshes {
    /// Index by occupant count, then clockwise sector index. Entry zero is empty.
    pub sectors: Vec<Vec<SectorMeshes>>,
    pub selected: Handle<Mesh>,
    pub legal: Handle<Mesh>,
    pub hovered: Handle<Mesh>,
}

impl FromWorld for HexMeshes {
    fn from_world(world: &mut World) -> Self {
        let mut meshes = world.resource_mut::<Assets<Mesh>>();
        let mut sectors = vec![Vec::new()];
        for count in 1..=4 {
            sectors.push(
                (0..count)
                    .map(|index| {
                        let start = index as f32 * TAU / count as f32;
                        let end = (index + 1) as f32 * TAU / count as f32;
                        SectorMeshes {
                            fill: meshes.add(sector_mesh(RADIUS * 151.0 / 160.0, 0.0, start, end)),
                            rim: meshes.add(sector_mesh(
                                RADIUS * 151.0 / 160.0,
                                RADIUS * 139.0 / 160.0,
                                start,
                                end,
                            )),
                        }
                    })
                    .collect(),
            );
        }
        Self {
            sectors,
            selected: meshes.add(sector_mesh(RADIUS - 1.0, RADIUS - 3.0, 0.0, TAU)),
            legal: meshes.add(sector_mesh(RADIUS - 5.0, RADIUS - 6.3, 0.0, TAU)),
            hovered: meshes.add(sector_mesh(RADIUS - 7.0, RADIUS - 8.5, 0.0, TAU)),
        }
    }
}

#[derive(Resource)]
pub(super) struct OverlayMaterials {
    pub fills: [Handle<ColorMaterial>; 4],
    pub rims: [Handle<ColorMaterial>; 4],
    pub selected: Handle<ColorMaterial>,
    pub legal: Handle<ColorMaterial>,
    pub hovered: Handle<ColorMaterial>,
}

impl FromWorld for OverlayMaterials {
    fn from_world(world: &mut World) -> Self {
        let mut materials = world.resource_mut::<Assets<ColorMaterial>>();
        Self {
            fills: std::array::from_fn(|id| {
                let [r, g, b] = PLAYER_COLORS[id];
                materials.add(ColorMaterial::from_color(Color::srgba(r, g, b, 0.345)))
            }),
            rims: std::array::from_fn(|id| {
                let [r, g, b] = PLAYER_COLORS[id];
                materials.add(ColorMaterial::from_color(Color::srgba(r, g, b, 0.92)))
            }),
            selected: materials.add(ColorMaterial::from_color(GOLD)),
            legal: materials.add(ColorMaterial::from_color(GOLD.with_alpha(0.65))),
            hovered: materials.add(ColorMaterial::from_color(PAPER)),
        }
    }
}

pub(super) struct HexMeshPlugin;
impl Plugin for HexMeshPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HexMeshes>()
            .init_resource::<OverlayMaterials>();
    }
}

/// Clockwise angle from north; intersect a ray with the exact hex boundary.
fn boundary(radius: f32, angle: f32) -> Vec2 {
    let side_midpoint = ((angle / FRAC_PI_3).floor() + 0.5) * FRAC_PI_3;
    let distance = radius * (FRAC_PI_3 / 2.0).cos() / (angle - side_midpoint).cos();
    Vec2::new(angle.sin(), angle.cos()) * distance
}

/// A convex fan, or an annular sector when inner_radius > 0. Include every
/// crossed hex corner so pie cuts end on straight hex edges, never a circle.
fn sector_vertices(radius: f32, inner_radius: f32, start: f32, end: f32) -> Vec<[f32; 3]> {
    let mut angles = vec![start];
    for corner in 1..=6 {
        let angle = corner as f32 * FRAC_PI_3;
        if angle > start + 0.00001 && angle < end - 0.00001 {
            angles.push(angle);
        }
    }
    angles.push(end);
    let mut vertices = Vec::new();
    for segment in angles.windows(2) {
        let outer_a = boundary(radius, segment[0]);
        let outer_b = boundary(radius, segment[1]);
        let inner_a = boundary(inner_radius, segment[0]);
        let inner_b = boundary(inner_radius, segment[1]);
        // Boundary traversal is clockwise; reverse triangle winding for +Z.
        for point in [inner_a, outer_b, outer_a] {
            vertices.push([point.x, point.y, 0.0]);
        }
        if inner_radius > 0.0 {
            for point in [inner_a, inner_b, outer_b] {
                vertices.push([point.x, point.y, 0.0]);
            }
        }
    }
    vertices
}

fn sector_mesh(radius: f32, inner_radius: f32, start: f32, end: f32) -> Mesh {
    let vertices = sector_vertices(radius, inner_radius, start, end);
    let uvs: Vec<_> = vertices
        .iter()
        .map(|p| [0.5 + p[0] / (radius * 2.0), 0.5 - p[1] / (radius * 2.0)])
        .collect();
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_NORMAL,
        vec![[0.0, 0.0, 1.0]; vertices.len()],
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vertices)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn area(vertices: &[[f32; 3]]) -> f32 {
        vertices
            .as_chunks::<3>()
            .0
            .iter()
            .map(|tri| {
                let a = Vec2::new(tri[0][0], tri[0][1]);
                let b = Vec2::new(tri[1][0], tri[1][1]);
                let c = Vec2::new(tri[2][0], tri[2][1]);
                let area = (b - a).perp_dot(c - a) / 2.0;
                assert!(area > 0.0, "degenerate or backwards triangle: {tri:?}");
                area
            })
            .sum()
    }

    #[test]
    fn pie_slices_partition_the_hex_and_rim_into_equal_areas() {
        let hex_area = 3.0 * 3.0_f32.sqrt() / 2.0 * 60.0_f32.powi(2);
        for count in 1..=4 {
            for index in 0..count {
                let start = index as f32 * TAU / count as f32;
                let end = (index + 1) as f32 * TAU / count as f32;
                let fill = sector_vertices(60.0, 0.0, start, end);
                let rim = sector_vertices(60.0, 54.0, start, end);
                assert!((area(&fill) - hex_area / count as f32).abs() < 0.01);
                assert!(
                    (area(&rim) - hex_area * (1.0 - 0.9_f32.powi(2)) / count as f32).abs() < 0.01
                );
                // All generated geometry remains within the regular hex's six half planes.
                for p in fill.iter().chain(&rim) {
                    for side in 0..6 {
                        let angle = (side as f32 + 0.5) * FRAC_PI_3;
                        let normal = Vec2::new(angle.sin(), angle.cos());
                        assert!(
                            Vec2::new(p[0], p[1]).dot(normal)
                                <= 60.0 * (FRAC_PI_3 / 2.0).cos() + 0.001
                        );
                    }
                }
            }
        }
    }
}
