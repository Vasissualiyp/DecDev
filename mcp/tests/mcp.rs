//! Tests for the MCP tool handlers and JSON-RPC dispatcher, driven directly
//! with fixture `Component`s — no stdin/stdout, no process (per
//! `specs/07-testing.md`).

use decdev_mcp::{handle_message, Catalog};
use serde_json::{json, Value};

const MOVEMENT: &str = r#"
name: Strafe Movement
category: movement
summary: Quake-style air control for testing.
capability:
  inputs: []
  outputs: []
  events: []
  dependencies: []
  determinism: none
implementations:
  - engine: Unity
    language: C#
    url: https://example.invalid/strafe
    license: MIT
license: MIT
provenance:
  type: original
"#;

const CAMERA: &str = r#"
name: Orbit Camera
category: camera
summary: Third-person orbit camera for testing.
capability:
  inputs: []
  outputs: []
  events: []
  dependencies: []
  determinism: none
implementations:
  - engine: Godot
    language: GDScript
    url: https://example.invalid/orbit
    license: MIT
license: MIT
provenance:
  type: original
"#;

fn catalog() -> Catalog {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("strafe-movement.yaml"), MOVEMENT).unwrap();
    std::fs::write(dir.path().join("orbit-camera.yaml"), CAMERA).unwrap();
    let (components, errors) = decdev_core::load_and_validate_components(dir.path());
    assert!(errors.is_empty(), "fixtures must be valid: {errors:?}");
    Catalog::new(components)
}

#[test]
fn tool_list_returns_every_component() {
    let listed = catalog().tool_list();
    assert_eq!(listed["components"].as_array().unwrap().len(), 2);
}

#[test]
fn tool_search_matches_and_returns_slug_records() {
    let found = catalog().tool_search("orbit");
    let array = found["components"].as_array().unwrap();
    assert_eq!(array.len(), 1);
    assert_eq!(array[0]["slug"], "orbit-camera");
}

#[test]
fn tool_show_returns_the_full_component_or_an_error() {
    let catalog = catalog();
    assert_eq!(
        catalog.tool_show("strafe-movement").unwrap()["slug"],
        "strafe-movement"
    );
    assert_eq!(
        catalog.tool_show("nope").unwrap_err(),
        "unknown component: nope"
    );
}

fn tool_text(response: &Value) -> Value {
    let text = response["result"]["content"][0]["text"]
        .as_str()
        .expect("tool result must carry text content");
    serde_json::from_str(text).expect("tool text must be JSON")
}

#[test]
fn initialize_echoes_the_requested_protocol_version() {
    let response = handle_message(
        &catalog(),
        &json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05"}}),
    )
    .unwrap();
    assert_eq!(response["result"]["protocolVersion"], "2024-11-05");
    assert_eq!(response["result"]["serverInfo"]["name"], "decdev");
}

#[test]
fn initialized_notification_gets_no_response() {
    let response = handle_message(
        &catalog(),
        &json!({"jsonrpc":"2.0","method":"notifications/initialized"}),
    );
    assert!(response.is_none());
}

#[test]
fn tools_list_exposes_list_show_and_search() {
    let response = handle_message(
        &catalog(),
        &json!({"jsonrpc":"2.0","id":2,"method":"tools/list"}),
    )
    .unwrap();
    let names: Vec<&str> = response["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["name"].as_str().unwrap())
        .collect();
    assert_eq!(names, vec!["list", "show", "search"]);
}

#[test]
fn tools_call_search_returns_matches() {
    let response = handle_message(
        &catalog(),
        &json!({"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"search","arguments":{"query":"orbit"}}}),
    )
    .unwrap();
    assert_eq!(response["result"]["isError"], false);
    let found = tool_text(&response);
    assert_eq!(found["components"].as_array().unwrap().len(), 1);
    assert_eq!(found["components"][0]["slug"], "orbit-camera");
}

#[test]
fn tools_call_show_missing_slug_is_a_tool_error_not_a_protocol_error() {
    let response = handle_message(
        &catalog(),
        &json!({"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"show","arguments":{}}}),
    )
    .unwrap();
    assert_eq!(response["result"]["isError"], true);
    assert!(response.get("error").is_none());
}

#[test]
fn unknown_method_with_an_id_is_a_method_not_found_error() {
    let response = handle_message(
        &catalog(),
        &json!({"jsonrpc":"2.0","id":5,"method":"does/not/exist"}),
    )
    .unwrap();
    assert_eq!(response["error"]["code"], -32601);
}
