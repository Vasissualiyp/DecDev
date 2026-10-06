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
