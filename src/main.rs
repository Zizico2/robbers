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
    if args.first().is_some_and(|arg| arg == "--validate-case") {
        if args.len() > 2 {
            return Err("use --validate-case [JSON_PATH]".into());
        }
        let case = if let Some(path) = args.get(1) {
            serde_json::from_reader(std::fs::File::open(path)?)?
        } else {
            robbers_case::bundled_case()
        };
        let report =
            robbers_validator::validate_case(&case, &robbers_game::Board::default(), 1_000_000)?;
        println!("{}", serde_json::to_string(&report)?);
        if !report.unique {
            return Err("case did not certify exactly one solution".into());
        }
        return Ok(());
    }
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        println!(
            "Robbers — hex-city deduction prototype\n\n  robbers                 graphical board\n  robbers --headless      deterministic JSON-lines on stdin/stdout\n  robbers --screenshot PATH   save a graphical frame, then exit\n  robbers --replay PATH   apply a JSON array of game commands before starting\n  robbers --case PATH     load and validate an authored case\n  robbers --validate-case [PATH]   certify case uniqueness\n\nHeadless-only build: cargo run --no-default-features -- --headless"
        );
        return Ok(());
    }
    let mut headless = false;
    let mut screenshot = None;
    let mut replay = None;
    let mut case_path = None;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--headless" => headless = true,
            "--screenshot" => {
                index += 1;
                screenshot = Some(args.get(index).ok_or("--screenshot needs a path")?.clone());
            }
            "--replay" => {
                index += 1;
                replay = Some(args.get(index).ok_or("--replay needs a path")?.clone());
            }
            "--case" => {
                index += 1;
                case_path = Some(args.get(index).ok_or("--case needs a JSON path")?.clone());
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
                            title: "ROBBERS / Investigation".into(),
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
    if let Some(path) = case_path {
        let case: robbers_case::CaseDefinition =
            serde_json::from_reader(std::fs::File::open(path)?)?;
        let board = robbers_game::Board::default();
        if !robbers_validator::validate_case(&case, &board, 1_000_000)?.unique {
            return Err("custom case must certify exactly one solution".into());
        }
        app.insert_resource(robbers_game::Mystery::new(case, &board)?);
    }
    app.add_plugins((GamePlugin, CommandPlugin));
    if let Some(path) = replay {
        let commands: Vec<robbers_command::GameCommand> =
            serde_json::from_reader(std::fs::File::open(path)?)?;
        for (index, command) in commands.into_iter().enumerate() {
            robbers_command::dispatch(app.world_mut(), command)
                .map_err(|error| format!("replay command {}: {}", index + 1, error.message))?;
        }
    }
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
