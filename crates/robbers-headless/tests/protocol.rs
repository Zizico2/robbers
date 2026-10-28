use bevy::prelude::*;
use robbers_command::CommandPlugin;
use robbers_game::GamePlugin;
use robbers_headless::process_line;

#[test]
fn malformed_requests_do_not_kill_the_session_or_mutate_state() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, GamePlugin, CommandPlugin));
    let world = app.world_mut();
    let initial = process_line(world, r#"{"id":1,"method":"state"}"#);
    for request in [
        "not json",
        "null",
        "[]",
        r#"{"method":"nope"}"#,
        r#"{"method":"command","command":{"type":"move","direction":"north"}}"#,
        r#"{"method":"command","command":{"type":"reset","extra":true}}"#,
    ] {
        assert!(process_line(world, request).get("error").is_some());
    }
    assert_eq!(
        initial["result"],
        process_line(world, r#"{"method":"state"}"#)["result"]
    );
    let moved = process_line(
        world,
        r#"{"id":"test","method":"command","command":{"type":"move","direction":"e"}}"#,
    );
    assert_eq!(moved["id"], "test");
    assert_eq!(moved["result"]["moves"], 1);
    assert_eq!(moved["result"]["players"][0]["hex"]["q"], -1);
    assert!(process_line(world, r#"{"method":"schema"}"#)["result"]["commands"].is_array());
}

#[test]
fn normal_snapshots_and_errors_never_disclose_hidden_evidence_or_the_solution() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, GamePlugin, CommandPlugin));
    for request in [
        r#"{"method":"state"}"#,
        r#"{"method":"command","command":{"type":"submit_route"}}"#,
        r#"{"method":"schema"}"#,
    ] {
        let output = process_line(app.world_mut(), request).to_string();
        assert!(!output.contains("\"solution\""));
        assert!(!output.contains("patrol_radio"));
        assert!(!output.contains("crossing_guard"));
        assert!(!output.contains("evidence"));
    }
    process_line(
        app.world_mut(),
        r#"{"method":"command","command":{"type":"investigate"}}"#,
    );
    let output = process_line(app.world_mut(), r#"{"method":"state"}"#).to_string();
    assert!(output.contains("patrol_radio"));
    assert!(!output.contains("crossing_guard"));
    process_line(
        app.world_mut(),
        r#"{"method":"command","command":{"type":"select","player":1}}"#,
    );
    assert!(
        !process_line(app.world_mut(), r#"{"method":"state"}"#)
            .to_string()
            .contains("patrol_radio")
    );
}
