mod fixtures;

use assert_cmd::Command;
use fixtures::{write_components, MOVEMENT_A};
use predicates::prelude::*;

#[test]
fn show_known_slug_prints_the_full_spec() {
    let dir = tempfile::tempdir().unwrap();
    write_components(dir.path(), &[("a.yaml", MOVEMENT_A)]);

    Command::cargo_bin("decdev")
        .unwrap()
        .current_dir(dir.path())
        .args(["show", "a"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Movement A")
                .and(predicate::str::contains("Quake"))
                .and(predicate::str::contains("MIT")),
        );
}

#[test]
fn show_unknown_slug_exits_nonzero_with_a_close_typo_suggestion() {
    let dir = tempfile::tempdir().unwrap();
    write_components(dir.path(), &[("quake-strafe.yaml", MOVEMENT_A)]);

    Command::cargo_bin("decdev")
        .unwrap()
        .current_dir(dir.path())
        .args(["show", "quake-strafee"])
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("Unknown component: quake-strafee")
                .and(predicate::str::contains("quake-strafe")),
        );
}
