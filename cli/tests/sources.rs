//! Tests for the repeatable global `--source` flag: loading additional
//! component directories on top of the discovered `components/` directory.
//! These cover the "plugin"/external-source mechanism described in
//! `specs/04-cli.md` — each command must see the extra sources, and a
//! duplicate slug across sources must fail rather than silently shadow.

mod fixtures;

use assert_cmd::Command;
use fixtures::{write_components, write_source, MOVEMENT_A, SOURCE_ONLY};
use predicates::prelude::*;

#[test]
fn list_includes_components_from_an_external_source() {
    let dir = tempfile::tempdir().unwrap();
    write_components(dir.path(), &[("a.yaml", MOVEMENT_A)]);
    let source = dir.path().join("external");
    std::fs::create_dir_all(&source).unwrap();
    write_source(&source, &[("sourced.yaml", SOURCE_ONLY)]);

    Command::cargo_bin("decdev")
        .unwrap()
        .current_dir(dir.path())
        .arg("list")
        .arg("--source")
        .arg(&source)
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Movement A")
                .and(predicate::str::contains("Sourced Component")),
        );
}

#[test]
fn search_finds_a_component_that_only_exists_in_an_external_source() {
    let dir = tempfile::tempdir().unwrap();
    write_components(dir.path(), &[("a.yaml", MOVEMENT_A)]);
    let source = dir.path().join("external");
    std::fs::create_dir_all(&source).unwrap();
    write_source(&source, &[("sourced.yaml", SOURCE_ONLY)]);

    Command::cargo_bin("decdev")
        .unwrap()
        .current_dir(dir.path())
        .args(["search", "Sourced", "--source"])
        .arg(&source)
        .assert()
        .success()
        .stdout(predicate::str::contains("Sourced Component"));
}

#[test]
fn show_resolves_a_slug_that_only_exists_in_an_external_source() {
    let dir = tempfile::tempdir().unwrap();
    write_components(dir.path(), &[("a.yaml", MOVEMENT_A)]);
    let source = dir.path().join("external");
    std::fs::create_dir_all(&source).unwrap();
    write_source(&source, &[("sourced.yaml", SOURCE_ONLY)]);

    Command::cargo_bin("decdev")
        .unwrap()
        .current_dir(dir.path())
        .args(["show", "sourced", "--source"])
        .arg(&source)
        .assert()
        .success()
        .stdout(predicate::str::contains("Sourced Component"));
}

#[test]
fn export_includes_components_from_an_external_source() {
    let dir = tempfile::tempdir().unwrap();
    write_components(dir.path(), &[("a.yaml", MOVEMENT_A)]);
    let source = dir.path().join("external");
    std::fs::create_dir_all(&source).unwrap();
    write_source(&source, &[("sourced.yaml", SOURCE_ONLY)]);

    let output = Command::cargo_bin("decdev")
        .unwrap()
        .current_dir(dir.path())
        .arg("export")
        .arg("--source")
        .arg(&source)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let parsed: serde_json::Value =
        serde_json::from_slice(&output).expect("stdout must be valid JSON");
    let array = parsed.as_array().expect("expected a JSON array");
    assert_eq!(array.len(), 2);
}

#[test]
fn validate_reports_files_from_an_external_source() {
    let dir = tempfile::tempdir().unwrap();
    write_components(dir.path(), &[("a.yaml", MOVEMENT_A)]);
    let source = dir.path().join("external");
    std::fs::create_dir_all(&source).unwrap();
    write_source(&source, &[("sourced.yaml", SOURCE_ONLY)]);

    Command::cargo_bin("decdev")
        .unwrap()
        .current_dir(dir.path())
        .arg("validate")
        .arg("--source")
        .arg(&source)
        .assert()
        .success()
        .stdout(predicate::str::contains("OK sourced"));
}

#[test]
fn a_source_works_even_with_no_discoverable_components_directory() {
    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("external");
    std::fs::create_dir_all(&source).unwrap();
    write_source(&source, &[("sourced.yaml", SOURCE_ONLY)]);

    Command::cargo_bin("decdev")
        .unwrap()
        .current_dir(dir.path())
        .arg("list")
        .arg("--source")
        .arg(&source)
        .assert()
        .success()
        .stdout(predicate::str::contains("Sourced Component"));
}

#[test]
fn duplicate_slug_across_components_and_an_external_source_fails_validation() {
    let dir = tempfile::tempdir().unwrap();
    write_components(dir.path(), &[("a.yaml", MOVEMENT_A)]);
    let source = dir.path().join("external");
    std::fs::create_dir_all(&source).unwrap();
    write_source(&source, &[("a.yaml", MOVEMENT_A)]);

    Command::cargo_bin("decdev")
        .unwrap()
        .current_dir(dir.path())
        .arg("validate")
        .arg("--source")
        .arg(&source)
        .assert()
        .failure()
        .stdout(predicate::str::contains("duplicate component slug 'a'"));
}

#[test]
fn export_is_all_or_nothing_when_a_source_duplicates_a_slug() {
    let dir = tempfile::tempdir().unwrap();
    write_components(dir.path(), &[("a.yaml", MOVEMENT_A)]);
    let source = dir.path().join("external");
    std::fs::create_dir_all(&source).unwrap();
    write_source(&source, &[("a.yaml", MOVEMENT_A)]);

    Command::cargo_bin("decdev")
        .unwrap()
        .current_dir(dir.path())
        .arg("export")
        .arg("--source")
        .arg(&source)
        .assert()
        .failure()
        .stdout(predicate::str::is_empty());
}
