mod fixtures;

use assert_cmd::Command;
use fixtures::{write_components, CAMERA_B, INVALID_MISSING_CATEGORY, MOVEMENT_A};
use predicates::prelude::*;

#[test]
fn list_returns_all_fixture_components() {
    let dir = tempfile::tempdir().unwrap();
    write_components(dir.path(), &[("a.yaml", MOVEMENT_A), ("b.yaml", CAMERA_B)]);

    Command::cargo_bin("decdev")
        .unwrap()
        .current_dir(dir.path())
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("Movement A").and(predicate::str::contains("Camera B")));
}

#[test]
fn list_category_filters_correctly() {
    let dir = tempfile::tempdir().unwrap();
    write_components(dir.path(), &[("a.yaml", MOVEMENT_A), ("b.yaml", CAMERA_B)]);

    Command::cargo_bin("decdev")
        .unwrap()
        .current_dir(dir.path())
        .args(["list", "--category", "camera"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Camera B").and(predicate::str::contains("Movement A").not()),
        );
}

#[test]
fn list_silently_excludes_a_component_that_fails_validation() {
    let dir = tempfile::tempdir().unwrap();
    write_components(
        dir.path(),
        &[
            ("a.yaml", MOVEMENT_A),
            ("invalid.yaml", INVALID_MISSING_CATEGORY),
        ],
    );

    Command::cargo_bin("decdev")
        .unwrap()
        .current_dir(dir.path())
        .arg("list")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Movement A")
                .and(predicate::str::contains("Invalid Component").not()),
        );
}

#[test]
fn list_json_output_is_valid_json_matching_fixture_data() {
    let dir = tempfile::tempdir().unwrap();
    write_components(dir.path(), &[("a.yaml", MOVEMENT_A)]);

    let output = Command::cargo_bin("decdev")
        .unwrap()
        .current_dir(dir.path())
        .args(["list", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let parsed: serde_json::Value =
        serde_json::from_slice(&output).expect("stdout must be valid JSON");
    let array = parsed.as_array().expect("expected a JSON array");
    assert_eq!(array.len(), 1);
    assert_eq!(array[0]["name"], "Movement A");
}
