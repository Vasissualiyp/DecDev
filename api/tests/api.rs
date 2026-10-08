//! Integration tests for the read-only API router. Drive the router
//! directly with `tower::ServiceExt::oneshot` — no real socket, no server
//! task, per `specs/07-testing.md`.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::Router;
use decdev_api::router;
use tower::ServiceExt;

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

fn app() -> Router {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("strafe-movement.yaml"), MOVEMENT).unwrap();
    std::fs::write(dir.path().join("orbit-camera.yaml"), CAMERA).unwrap();
    let (components, errors) = decdev_core::load_and_validate_components(dir.path());
    assert!(errors.is_empty(), "fixtures must be valid: {errors:?}");
    router(components)
}

async fn get(app: Router, uri: &str) -> (StatusCode, serde_json::Value) {
    let response = app
        .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json = serde_json::from_slice(&bytes).expect("response body must be JSON");
    (status, json)
}

#[tokio::test]
async fn list_returns_every_component() {
    let (status, json) = get(app(), "/components").await;
    assert_eq!(status, StatusCode::OK);
    let array = json.as_array().expect("expected a JSON array");
    assert_eq!(array.len(), 2);
}

#[tokio::test]
async fn show_returns_the_component_for_a_known_slug() {
    let (status, json) = get(app(), "/components/strafe-movement").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["name"], "Strafe Movement");
    assert_eq!(json["slug"], "strafe-movement");
}

#[tokio::test]
async fn show_returns_404_json_for_an_unknown_slug() {
    let (status, json) = get(app(), "/components/no-such-component").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(json["error"], "unknown component: no-such-component");
}

#[tokio::test]
async fn search_filters_by_query_param() {
    let (status, json) = get(app(), "/search?q=orbit").await;
    assert_eq!(status, StatusCode::OK);
    let array = json.as_array().expect("expected a JSON array");
    assert_eq!(array.len(), 1);
    assert_eq!(array[0]["slug"], "orbit-camera");
}

#[tokio::test]
async fn search_with_no_match_returns_an_empty_array_and_200() {
    let (status, json) = get(app(), "/search?q=zzzznotfound").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json.as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn search_without_the_q_parameter_is_a_400() {
    let (status, json) = get(app(), "/search").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(json["error"], "missing required query parameter: q");
}
