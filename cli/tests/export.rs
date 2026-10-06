use assert_cmd::Command;
use predicates::prelude::*;

const VALID_A: &str = r#"
name: Component A
category: movement
summary: A valid component.
capability:
  inputs: []
  outputs: []
  events: []
  dependencies: []
  determinism: none
implementations:
  - engine: engine-agnostic
    language: C#
    url: https://example.invalid/a
    license: MIT
license: MIT
provenance:
  type: original
"#;

const VALID_B: &str = r#"
name: Component B
category: camera
summary: Another valid component.
capability:
  inputs: []
  outputs: []
  events: []
  dependencies: []
  determinism: none
implementations:
  - engine: engine-agnostic
    language: C#
    url: https://example.invalid/b
    license: MIT
license: MIT
provenance:
  type: original
"#;

const INVALID: &str = r#"
name: Missing Category
summary: This file is missing the required category field.
capability:
  inputs: []
  outputs: []
  events: []
  dependencies: []
  determinism: none
implementations:
  - engine: engine-agnostic
    language: C#
    url: https://example.invalid/c
    license: MIT
license: MIT
provenance:
  type: original
"#;

fn write_components(dir: &std::path::Path, files: &[(&str, &str)]) {
    let components = dir.join("components");
    std::fs::create_dir_all(&components).unwrap();
    for (name, content) in files {
        std::fs::write(components.join(name), content).unwrap();
    }
}

#[test]
fn export_all_valid_prints_json_array_and_exits_zero() {
    let dir = tempfile::tempdir().unwrap();
    write_components(dir.path(), &[("a.yaml", VALID_A), ("b.yaml", VALID_B)]);

    Command::cargo_bin("decdev")
        .unwrap()
        .current_dir(dir.path())
        .arg("export")
        .assert()
        .success()
        .stdout(predicate::str::starts_with("[").and(predicate::str::contains("Component A")));
}

#[test]
fn export_with_one_invalid_file_prints_nothing_and_exits_nonzero() {
    let dir = tempfile::tempdir().unwrap();
    write_components(
        dir.path(),
        &[("a.yaml", VALID_A), ("c-invalid.yaml", INVALID)],
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
