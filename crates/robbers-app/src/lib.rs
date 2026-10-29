//! Shared graphical startup for desktop, browser and the Android activity.
#[cfg(feature = "graphics")]
pub use robbers_view::{ViewPlugin, capture_and_exit};

#[cfg(feature = "graphics")]
pub fn configure_graphics(app: &mut bevy::app::App) {
    use bevy::prelude::*;

    let window = Window {
        title: "ROBBERS / Investigation".into(),
        resolution: (1360, 900).into(),
        ..default()
    };
    // Android reads packaged assets through AAssetManager; the browser fetches
    // relative URLs. Only the desktop development build uses the checkout path.
    #[cfg(not(any(target_arch = "wasm32", target_os = "android")))]
    let assets = AssetPlugin {
        file_path: format!("{}/../../assets", env!("CARGO_MANIFEST_DIR")),
        ..default()
    };
    #[cfg(any(target_arch = "wasm32", target_os = "android"))]
    let assets = AssetPlugin::default();
    #[cfg(target_arch = "wasm32")]
    let window = Window {
        canvas: Some("#robbers".into()),
        fit_canvas_to_parent: true,
        prevent_default_event_handling: true,
        ..window
    };
    #[cfg(target_os = "android")]
    let window = Window {
        mode: bevy::window::WindowMode::BorderlessFullscreen(MonitorSelection::Primary),
        ..window
    };
    app.add_plugins(DefaultPlugins.set(assets).set(WindowPlugin {
        primary_window: Some(window),
        ..default()
    }));
    #[cfg(target_os = "android")]
    app.insert_resource(bevy::winit::WinitSettings::mobile());
}

/// Starts the bundled case with the same command layer on every graphical host.
#[cfg(feature = "graphics")]
pub fn run() {
    let mut app = bevy::app::App::new();
    configure_graphics(&mut app);
    app.add_plugins((
        robbers_game::GamePlugin,
        robbers_command::CommandPlugin,
        ViewPlugin,
    ));
    app.run();
}
