use bevy::{app::ScheduleRunnerPlugin, prelude::*};
use robbers_command::CommandPlugin;
use robbers_game::GamePlugin;
use std::time::Duration;

fn main() {
    if let Err(error) = run() {
        eprintln!("robbers: {error}");
        std::process::exit(1);
    }
}
fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        println!(
            "Robbers — hex-city movement prototype\n\n  robbers                 graphical board\n  robbers --headless      deterministic JSON-lines on stdin/stdout\n  robbers --screenshot PATH   save a graphical frame, then exit\n\nHeadless-only build: cargo run --no-default-features -- --headless"
        );
        return Ok(());
    }
    let mut headless = false;
    let mut screenshot = None;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--headless" => headless = true,
            "--screenshot" => {
                index += 1;
                screenshot = Some(args.get(index).ok_or("--screenshot needs a path")?.clone());
            }
            unknown => return Err(format!("unknown option {unknown}; use --help").into()),
        }
        index += 1;
    }
    if headless && screenshot.is_some() {
        return Err("--screenshot requires graphics".into());
    }
    let mut app = App::new();
    if headless {
        app.add_plugins(
            MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(Duration::from_millis(10))),
        );
    } else {
        #[cfg(feature = "graphics")]
        {
            app.add_plugins(
                DefaultPlugins
                    .set(AssetPlugin {
                        file_path: format!("{}/assets", env!("CARGO_MANIFEST_DIR")),
                        ..default()
                    })
                    .set(WindowPlugin {
                        primary_window: Some(Window {
                            title: "ROBBERS / City patrol".into(),
                            resolution: (1360, 900).into(),
                            ..default()
                        }),
                        ..default()
                    }),
            );
        }
        #[cfg(not(feature = "graphics"))]
        return Err(
            "this build has no graphics; use --headless or build with --features graphics".into(),
        );
    }
    app.add_plugins((GamePlugin, CommandPlugin));
    if !headless {
        #[cfg(feature = "graphics")]
        {
            app.add_plugins(robbers_view::ViewPlugin);
            if let Some(path) = screenshot {
                robbers_view::capture_and_exit(&mut app, path);
            }
        }
    }
    if headless {
        robbers_headless::run_stdio(&mut app)?;
    } else {
        app.run();
    }
    Ok(())
}
