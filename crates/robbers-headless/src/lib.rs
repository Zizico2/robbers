//! Renderer-free, deterministic JSON-lines controller.
use bevy::prelude::*;
use robbers_command::{GameCommand, dispatch, schema};
use robbers_game::snapshot;
use serde_json::{Value, json};
use std::io::{self, BufRead, Write};

/// One line in, one flushed line out. No background loop or timing assumptions.
pub fn run_stdio(app: &mut App) -> io::Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::BufWriter::new(io::stdout().lock());
    for line in stdin.lock().lines() {
        let line = line?;
        let response = process_line(app.world_mut(), &line);
        let quit = response
            .get("result")
            .and_then(|r| r.get("quit"))
            .and_then(Value::as_bool)
            == Some(true);
        serde_json::to_writer(&mut stdout, &response)?;
        writeln!(stdout)?;
        stdout.flush()?;
        if quit {
            break;
        }
    }
    Ok(())
}

pub fn process_line(world: &mut World, line: &str) -> Value {
    let request: Value = match serde_json::from_str(line) {
        Ok(value) => value,
        Err(error) => {
            return json!({"id":null,"error":{"code":"invalid_json","message":error.to_string()}});
        }
    };
    let id = request.get("id").cloned().unwrap_or(Value::Null);
    let result = match request.get("method").and_then(Value::as_str) {
        Some("state") => Ok(json!(snapshot(world))),
        Some("schema") => Ok(schema()),
        Some("quit") => Ok(json!({"quit":true})),
        Some("command") => {
            match serde_json::from_value::<GameCommand>(
                request.get("command").cloned().unwrap_or(Value::Null),
            ) {
                Ok(command) => dispatch(world, command)
                    .map(|state| json!(state))
                    .map_err(|error| json!(error)),
                Err(error) => Err(json!({"code":"invalid_command","message":error.to_string()})),
            }
        }
        _ => Err(json!({"code":"unknown_method","message":"Use state, schema, command or quit."})),
    };
    match result {
        Ok(result) => json!({"id":id,"result":result}),
        Err(error) => json!({"id":id,"error":error}),
    }
}
