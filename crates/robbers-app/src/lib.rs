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
    let plugins = DefaultPlugins.set(assets).set(WindowPlugin {
        primary_window: Some(window),
        ..default()
    });
    #[cfg(target_os = "android")]
    let plugins = plugins.set(bevy::render::RenderPlugin {
        render_creation: android_render_settings(bevy::render::settings::WgpuSettings::default())
            .into(),
        ..default()
    });
    app.add_plugins(plugins);
    #[cfg(target_os = "android")]
    app.insert_resource(bevy::winit::WinitSettings::mobile());
}

#[cfg(all(feature = "graphics", any(target_os = "android", test)))]
fn android_render_settings(
    mut settings: bevy::render::settings::WgpuSettings,
) -> bevy::render::settings::WgpuSettings {
    // PowerVR's Vulkan shader compiler can abort on SPIR-V debug source metadata
    // ("spvcompiler: unexpected source language"). APK debug signing / Rust debug
    // builds do not require GPU shader debug info. Keep the other instance flags,
    // including validation, and the normal adapter/backend selection.
    // https://github.com/CyberTimon/RapidRAW/issues/1723
    settings
        .instance_flags
        .remove(bevy::render::settings::InstanceFlags::DEBUG);
    settings
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

#[cfg(all(test, feature = "graphics"))]
mod tests {
    use super::android_render_settings;
    use bevy::render::settings::{Backends, InstanceFlags, WgpuSettings};

    #[test]
    fn android_disables_shader_debug_metadata_without_disabling_validation() {
        let original_flags = InstanceFlags::all();
        let settings = android_render_settings(WgpuSettings {
            instance_flags: original_flags,
            backends: Some(Backends::VULKAN),
            ..Default::default()
        });
        assert!(!settings.instance_flags.contains(InstanceFlags::DEBUG));
        assert!(settings.instance_flags.contains(InstanceFlags::VALIDATION));
        assert_eq!(
            settings.instance_flags,
            original_flags & !InstanceFlags::DEBUG
        );
        assert_eq!(settings.backends, Some(Backends::VULKAN));
    }
}
