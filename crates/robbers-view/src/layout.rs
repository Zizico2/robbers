//! Viewport fitting only. Gameplay and board artwork use unscaled local hex units.
//!
//! A single parent transform fits the board between the heading/footer and an
//! inspector. Short landscape windows keep the HUD at readable size and show
//! notes over the board. All responsive rules live here. Bevy's sprite picking
//! follows the board transform, so input needs no parallel pixel-to-hex conversion.
use bevy::prelude::*;

pub(crate) const RADIUS: f32 = 64.0;
const INSPECTOR_SPACE: f32 = 430.0;
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
    pub hud_scale: f32,
    pub compact: bool,
    pub panel_width: f32,
    pub board_width: f32,
    viewport: Vec2,
}

impl Default for BoardLayout {
    fn default() -> Self {
        Self::fit(Vec2::new(1440.0, 900.0))
    }
}

impl BoardLayout {
    fn fit(viewport: Vec2) -> Self {
        if viewport.y < 650.0 {
            let panel_width = (viewport.x * 0.36)
                .clamp(260.0, 340.0)
                .min(viewport.x - 160.0);
            let board_width = (viewport.x - panel_width - 36.0).max(100.0);
            let available_height = (viewport.y - 70.0).max(100.0);
            return Self {
                center: Vec2::new(
                    12.0 + board_width / 2.0 - viewport.x / 2.0,
                    viewport.y / 2.0 - 12.0 - available_height / 2.0,
                ),
                scale: (board_width / BOARD_WIDTH).min(available_height / BOARD_HEIGHT),
                hud_scale: 1.0,
                compact: true,
                panel_width,
                board_width,
                viewport,
            };
        }
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
            compact: false,
            panel_width: 360.0,
            board_width: available_width / hud_scale,
            viewport,
        }
    }
}

/// Presentation state only; opening notes never changes gameplay.
#[derive(Resource, Default)]
pub(crate) struct NotebookOpen(pub bool);

#[derive(Component, Clone, Default)]
pub(crate) enum ResponsiveNode {
    #[default]
    DesktopOnly,
    CompactOnly,
    Panel,
    Footer,
    Notebook,
    TurnTitle,
    Status,
}

pub(crate) struct BoardLayoutPlugin;
impl Plugin for BoardLayoutPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BoardLayout>()
            .init_resource::<NotebookOpen>()
            .add_systems(Update, fit_board);
    }
}

fn fit_board(
    windows: Query<&Window>,
    mut roots: Query<&mut Transform, With<BoardRoot>>,
    mut layout: ResMut<BoardLayout>,
    mut ui_scale: ResMut<UiScale>,
    notebook: Res<NotebookOpen>,
    mut nodes: Query<(
        &ResponsiveNode,
        &mut Node,
        Option<&mut BackgroundColor>,
        Option<&mut TextFont>,
    )>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let next = BoardLayout::fit(Vec2::new(window.width(), window.height()));
    let geometry_changed = layout.viewport != next.viewport;
    if !geometry_changed && !notebook.is_changed() {
        return;
    }
    if ui_scale.0 != next.hud_scale {
        ui_scale.0 = next.hud_scale;
    }
    for (role, mut node, background, font) in &mut nodes {
        match role {
            ResponsiveNode::DesktopOnly => {
                node.display = if next.compact {
                    Display::None
                } else {
                    Display::Flex
                }
            }
            ResponsiveNode::CompactOnly => {
                node.display = if next.compact {
                    Display::Flex
                } else {
                    Display::None
                }
            }
            ResponsiveNode::Panel => {
                node.right = px(if next.compact { 12.0 } else { 24.0 });
                node.top = px(if next.compact { 12.0 } else { 20.0 });
                node.bottom = node.top;
                node.width = px(next.panel_width);
                node.row_gap = px(if next.compact { 5.0 } else { 8.0 });
            }
            ResponsiveNode::Footer => {
                node.display = if next.compact && notebook.0 {
                    Display::None
                } else {
                    Display::Flex
                };
                node.left = px(if next.compact { 12.0 } else { 40.0 });
                node.bottom = px(if next.compact { 8.0 } else { 25.0 });
                node.width = if next.compact {
                    px(next.board_width)
                } else {
                    Val::Auto
                };
                node.height = if next.compact { px(46) } else { Val::Auto };
                node.justify_content = if next.compact {
                    JustifyContent::FlexEnd
                } else {
                    JustifyContent::FlexStart
                };
                node.max_width = if next.compact {
                    px(next.board_width)
                } else {
                    percent(52)
                };
            }
            ResponsiveNode::TurnTitle => {
                if let Some(mut font) = font {
                    font.font_size = px(if next.compact { 18.0 } else { 21.0 }).into();
                }
            }
            ResponsiveNode::Status => {
                if let Some(mut font) = font {
                    font.font_size = px(if next.compact { 13.0 } else { 15.0 }).into();
                }
            }
            ResponsiveNode::Notebook => {
                node.display = if !next.compact || notebook.0 {
                    Display::Flex
                } else {
                    Display::None
                };
                node.position_type = if next.compact {
                    PositionType::Absolute
                } else {
                    PositionType::Relative
                };
                node.left = if next.compact {
                    px(-next.board_width - 12.0)
                } else {
                    Val::Auto
                };
                node.top = if next.compact { px(0) } else { Val::Auto };
                node.bottom = node.top;
                node.width = if next.compact {
                    px(next.board_width)
                } else {
                    Val::Auto
                };
                node.padding = UiRect::all(px(if next.compact { 12.0 } else { 0.0 }));
                if let Some(mut background) = background {
                    background.0 = if next.compact {
                        crate::INK
                    } else {
                        Color::NONE
                    };
                }
            }
        }
    }
    if geometry_changed {
        for mut root in &mut roots {
            root.translation = next.center.extend(0.0);
            root.scale = Vec3::splat(next.scale);
        }
        *layout = next;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn landscape_phone_keeps_board_clear_of_controls_without_shrinking_hud() {
        for (w, h) in [
            (640.0, 360.0),
            (844.0, 390.0),
            (915.0, 412.0),
            (1024.0, 600.0),
        ] {
            let layout = BoardLayout::fit(Vec2::new(w, h));
            assert!(layout.compact);
            assert_eq!(layout.hud_scale, 1.0);
            let center = Vec2::new(w / 2.0 + layout.center.x, h / 2.0 - layout.center.y);
            let extent = Vec2::new(BOARD_WIDTH, BOARD_HEIGHT) * layout.scale / 2.0;
            assert!(center.x - extent.x >= 11.9);
            assert!(center.x + extent.x <= w - layout.panel_width - 23.9);
            assert!(center.y - extent.y >= 11.9);
            assert!(center.y + extent.y <= h - 57.9);
        }
    }

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
