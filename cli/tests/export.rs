mod fixtures;

use assert_cmd::Command;
use fixtures::{write_components, CAMERA_B, INVALID_MISSING_CATEGORY, MOVEMENT_A};
use predicates::prelude::*;

#[test]
fn export_all_valid_prints_json_array_and_exits_zero() {
    let dir = tempfile::tempdir().unwrap();
    write_components(dir.path(), &[("a.yaml", MOVEMENT_A), ("b.yaml", CAMERA_B)]);

    Command::cargo_bin("decdev")
        .unwrap()
        .current_dir(dir.path())
        .arg("export")
        .assert()
        .success()
        .stdout(predicate::str::starts_with("[").and(predicate::str::contains("Movement A")));
}

#[test]
fn export_with_one_invalid_file_prints_nothing_and_exits_nonzero() {
    let dir = tempfile::tempdir().unwrap();
    write_components(
        dir.path(),
        &[
            ("a.yaml", MOVEMENT_A),
            ("c-invalid.yaml", INVALID_MISSING_CATEGORY),
        ],
    );

    Command::cargo_bin("decdev")
        .unwrap()
        .current_dir(dir.path())
        .arg("export")
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::is_empty().not());
}
