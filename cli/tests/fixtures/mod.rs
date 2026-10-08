//! Shared fixture YAML for the CLI integration tests.
//!
//! Not every test binary (each file under tests/ is compiled separately)
//! uses every constant here — that's expected for a shared fixture module,
//! not dead code.
#![allow(dead_code)]

pub const MOVEMENT_A: &str = r#"
name: Movement A
category: movement
summary: A movement component for testing.
description: Uses Quake-style strafing.
reference_games:
  - Quake
capability:
  inputs: []
  outputs: []
  events: []
  dependencies: []
  determinism: none
implementations:
  - engine: Unity
    language: C#
    url: https://example.invalid/a
    license: MIT
license: MIT
provenance:
  type: original
"#;

/// Camera component whose matching engine/license sit on the SECOND
/// implementations[] entry, not the first — the regression test fixture
/// for the implementations[0]-only bug from the first seed-data incident.
pub const CAMERA_B: &str = r#"
name: Camera B
category: camera
summary: A camera component for testing.
capability:
  inputs: []
  outputs: []
  events: []
  dependencies: []
  determinism: none
implementations:
  - engine: Unity
    language: C#
    url: https://example.invalid/b-unity
    license: MIT
  - engine: Godot
    language: GDScript
    url: https://example.invalid/b-godot
    license: GPL-3.0
license: MIT
provenance:
  type: original
"#;

/// A component whose implementation carries a selective-extraction
/// recipe, for testing `show`'s extraction-recipe rendering.
pub const EXTRACTION_COMPONENT: &str = r#"
name: Extracted Terrain
category: procgen
summary: A procgen component for testing extraction-recipe rendering.
capability:
  inputs: []
  outputs: []
  events: []
  dependencies: []
  determinism: none
implementations:
  - engine: other
    language: Rust
    url: https://example.invalid/upstream-repo
    license: MIT
    extraction:
      include: ["world/src/sim/"]
      entry_points:
        - { path: "world/src/sim/mod.rs", symbol: "WorldSim::generate" }
license: MIT
provenance:
  type: open_source_derived
  derived_from: https://example.invalid/upstream-repo
"#;

pub const INVALID_MISSING_CATEGORY: &str = r#"
name: Invalid Component
summary: Missing the required category field.
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

pub fn write_components(dir: &std::path::Path, files: &[(&str, &str)]) {
    let components = dir.join("components");
    std::fs::create_dir_all(&components).unwrap();
    for (name, content) in files {
        std::fs::write(components.join(name), content).unwrap();
    }
}

/// A component that lives only in an external `--source` directory, with a
/// slug distinct from the in-repo fixtures, for testing external sources.
pub const SOURCE_ONLY: &str = r#"
name: Sourced Component
category: physics
summary: A component provided by an external source directory.
capability:
  inputs: []
  outputs: []
  events: []
  dependencies: []
  determinism: none
implementations:
  - engine: Godot
    language: GDScript
    url: https://example.invalid/sourced
    license: MIT
license: MIT
provenance:
  type: original
"#;

/// Writes component YAML files directly into `dir` — an external source
/// directory for the CLI's `--source` flag — unlike `write_components`,
/// which nests them under a `components/` subdirectory.
pub fn write_source(dir: &std::path::Path, files: &[(&str, &str)]) {
    for (name, content) in files {
        std::fs::write(dir.join(name), content).unwrap();
    }
}

/// A component that asserts compatibility with `a` (the slug of
/// `MOVEMENT_A`), for testing `show`'s own + reciprocal rendering.
pub const COMPAT_OWNER: &str = r#"
name: Compat Owner
category: other
summary: Owns a compatibility assertion for testing.
capability:
  inputs: []
  outputs: []
  events: []
  dependencies: []
  determinism: none
implementations:
  - engine: other
    language: C++
    url: https://example.invalid/compat
    license: MIT
compatibility:
  - with: a
    relation: pairs-with
    note: Pairs with movement A for testing.
license: MIT
provenance:
  type: original
"#;
