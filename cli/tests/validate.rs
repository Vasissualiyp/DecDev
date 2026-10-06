mod fixtures;

use assert_cmd::Command;
use fixtures::{write_components, INVALID_MISSING_CATEGORY, MOVEMENT_A};
use predicates::prelude::*;

#[test]
fn validate_fixture_with_one_valid_and_one_invalid_reports_both_and_exits_nonzero() {
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
        .arg("validate")
        .assert()
        .failure()
        .stdout(predicate::str::contains("OK a").and(predicate::str::contains("FAIL invalid")));
}

#[test]
fn validate_all_valid_fixture_set_exits_zero() {
    let dir = tempfile::tempdir().unwrap();
    write_components(dir.path(), &[("a.yaml", MOVEMENT_A)]);

    Command::cargo_bin("decdev")
        .unwrap()
        .current_dir(dir.path())
        .arg("validate")
        .assert()
        .success()
        .stdout(predicate::str::contains("OK a"));
}

#[test]
fn validate_explicit_path_argument_validates_just_that_file() {
    let dir = tempfile::tempdir().unwrap();
    write_components(dir.path(), &[("a.yaml", MOVEMENT_A)]);
    let path = dir.path().join("components").join("a.yaml");

    Command::cargo_bin("decdev")
        .unwrap()
        .current_dir(dir.path())
        .arg("validate")
        .arg(&path)
        .assert()
        .success()
        .stdout(predicate::str::contains("OK a"));
}
