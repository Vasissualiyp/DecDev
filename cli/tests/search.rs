mod fixtures;

use assert_cmd::Command;
use fixtures::{write_components, CAMERA_B, MOVEMENT_A};
use predicates::prelude::*;

fn setup() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    write_components(dir.path(), &[("a.yaml", MOVEMENT_A), ("b.yaml", CAMERA_B)]);
    dir
}

#[test]
fn search_matches_on_name() {
    let dir = setup();
    Command::cargo_bin("decdev")
        .unwrap()
        .current_dir(dir.path())
        .args(["search", "movement a"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Movement A"));
}

#[test]
fn search_matches_on_summary() {
    let dir = setup();
    Command::cargo_bin("decdev")
        .unwrap()
        .current_dir(dir.path())
        .args(["search", "camera component for testing"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Camera B"));
}

#[test]
fn search_matches_on_description() {
    let dir = setup();
    Command::cargo_bin("decdev")
        .unwrap()
        .current_dir(dir.path())
        .args(["search", "quake-style strafing"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Movement A"));
}

#[test]
fn search_matches_on_reference_games() {
    let dir = setup();
    Command::cargo_bin("decdev")
        .unwrap()
        .current_dir(dir.path())
        .args(["search", "quake"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Movement A"));
}

#[test]
fn search_with_no_matches_returns_empty_result_and_exit_zero() {
    let dir = setup();
    Command::cargo_bin("decdev")
        .unwrap()
        .current_dir(dir.path())
        .args(["search", "nonexistent-query-xyz"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

#[test]
fn search_combined_with_category_narrows_correctly() {
    let dir = setup();
    // Both fixtures match "a", but only Movement A is category=movement.
    Command::cargo_bin("decdev")
        .unwrap()
        .current_dir(dir.path())
        .args(["search", "a", "--category", "movement"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Movement A").and(predicate::str::contains("Camera B").not()),
        );
}

#[test]
fn search_engine_matches_non_first_implementation_entry() {
    let dir = setup();
    // Camera B's Godot implementation is implementations[1], not [0] — this
    // is the regression test for the implementations[0]-only bug.
    Command::cargo_bin("decdev")
        .unwrap()
        .current_dir(dir.path())
        .args(["search", "camera", "--engine", "Godot"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Camera B"));
}

#[test]
fn search_license_matches_non_first_implementation_entry() {
    let dir = setup();
    // Camera B's GPL-3.0 license is on implementations[1], not [0].
    Command::cargo_bin("decdev")
        .unwrap()
        .current_dir(dir.path())
        .args(["search", "camera", "--license", "GPL-3.0"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Camera B"));
}
