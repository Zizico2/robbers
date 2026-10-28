//! Viewport fitting only. Gameplay and board artwork use unscaled local hex units.
//!
//! A single parent transform fits the board between the heading/footer and a
//! fixed-width inspector. Bevy's sprite picking follows that transform, so input
//! needs no parallel pixel-to-hex conversion. Works with tiling window managers.
use bevy::prelude::*;

pub(crate) const RADIUS: f32 = 64.0;
const INSPECTOR_SPACE: f32 = 350.0;
const HORIZONTAL_MARGIN: f32 = 60.0;
const VERTICAL_MARGIN: f32 = 240.0;
const BOARD_WIDTH: f32 = 7.0 * 1.732_050_8 * RADIUS;
const BOARD_HEIGHT: f32 = 11.0 * RADIUS;

#[derive(Component)]
pub(crate) struct BoardRoot;

#[derive(Resource)]
pub(crate) struct BoardLayout {
    pub center: Vec2,
    pub scale: f32,
    hud_scale: f32,
}

impl Default for BoardLayout {
    fn default() -> Self {
        Self::fit(Vec2::new(1440.0, 900.0))
    }
}

impl BoardLayout {
    fn fit(viewport: Vec2) -> Self {
        let hud_scale = (viewport.x / 900.0).min(viewport.y / 900.0).clamp(0.1, 1.0);
        let inspector = INSPECTOR_SPACE * hud_scale;
        let available_width = (viewport.x - inspector - HORIZONTAL_MARGIN * hud_scale).max(100.0);
        let available_height = (viewport.y - VERTICAL_MARGIN * hud_scale).max(100.0);
        let scale = (available_width / BOARD_WIDTH)
            .min(available_height / BOARD_HEIGHT)
            .min(1.2);
        // On tall windows keep the city near the controls instead of pushing it
        // into the middle of a large empty column.
        let center_y = (viewport.y / 2.0).min(140.0 * hud_scale + BOARD_HEIGHT * scale / 2.0);
        Self {
            center: Vec2::new(-inspector / 2.0, viewport.y / 2.0 - center_y),
            scale,
            hud_scale,
        }
    }
}

pub(crate) struct BoardLayoutPlugin;
impl Plugin for BoardLayoutPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BoardLayout>()
            .add_systems(Update, fit_board);
    }
}

fn fit_board(
    windows: Query<&Window>,
    mut roots: Query<&mut Transform, With<BoardRoot>>,
    mut layout: ResMut<BoardLayout>,
    mut ui_scale: ResMut<UiScale>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let next = BoardLayout::fit(Vec2::new(window.width(), window.height()));
    if ui_scale.0 != next.hud_scale {
        ui_scale.0 = next.hud_scale;
    }
    if layout.center == next.center && layout.scale == next.scale {
        return;
    }
    for mut root in &mut roots {
        root.translation = next.center.extend(0.0);
        root.scale = Vec3::splat(next.scale);
    }
    *layout = next;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn city_fits_standard_ultrawide_and_tall_tiled_windows() {
        for (width, height) in [
            (1440.0, 810.0),
            (1920.0, 1080.0),
            (3440.0, 1440.0),
            (5120.0, 1440.0),
            (957.0, 1356.0),
            (1919.0, 1368.0),
            (900.0, 900.0),
        ] {
            let viewport = Vec2::new(width, height);
            let layout = BoardLayout::fit(viewport);
            let center = Vec2::new(
                viewport.x / 2.0 + layout.center.x,
                viewport.y / 2.0 - layout.center.y,
            );
            let extent = Vec2::new(BOARD_WIDTH, BOARD_HEIGHT) * layout.scale / 2.0;
            assert!(
                center.x - extent.x >= HORIZONTAL_MARGIN * layout.hud_scale / 2.0 - 0.01,
                "left crop at {viewport}"
            );
            assert!(
                center.x + extent.x
                    <= width - (INSPECTOR_SPACE + HORIZONTAL_MARGIN / 2.0) * layout.hud_scale
                        + 0.01,
                "inspector overlap at {viewport}"
            );
            assert!(
                center.y - extent.y >= 120.0 * layout.hud_scale - 0.01,
                "heading overlap at {viewport}"
            );
            assert!(
                center.y + extent.y <= height - 120.0 * layout.hud_scale + 0.01,
                "footer overlap at {viewport}"
            );
        }
    }
}
