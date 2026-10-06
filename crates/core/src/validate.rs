use crate::model::Component;
use std::path::{Path, PathBuf};

const SCHEMA_STR: &str = include_str!("../../../schema/component.schema.json");

const ALLOWED_LICENSES: &[&str] = &[
    "MIT",
    "Apache-2.0",
    "BSD-2-Clause",
    "BSD-3-Clause",
    "CC-BY-4.0",
    "CC0-1.0",
    "GPL-3.0",
    "LGPL-3.0",
];

#[derive(Debug, Clone)]
pub struct ValidationError {
    pub file: PathBuf,
    pub message: String,
}

/// Reads every `*.yaml` file under `dir`, validates each against
/// `schema/component.schema.json` plus the non-schema rules documented in
/// specs/01-capability-spec-format.md (legal_review, derived_from, license
/// allow-list), and returns (valid components, errors) — one bad file does
/// not block the rest. Both lists are ordered by filename for determinism.
pub fn load_and_validate_components(dir: &Path) -> (Vec<Component>, Vec<ValidationError>) {
    let schema_value: serde_json::Value = serde_json::from_str(SCHEMA_STR)
        .expect("embedded schema/component.schema.json must be valid JSON");
    let validator = jsonschema::validator_for(&schema_value)
        .expect("embedded schema/component.schema.json must be a valid JSON Schema");

    let mut valid = Vec::new();
    let mut errors = Vec::new();

    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(e) => {
            errors.push(ValidationError {
                file: dir.to_path_buf(),
                message: format!("could not read directory: {e}"),
            });
            return (valid, errors);
        }
    };

    let mut paths: Vec<PathBuf> = entries
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("yaml"))
        .collect();
    paths.sort();

    for path in paths {
        match validate_one(&path, &validator) {
            Ok(component) => valid.push(component),
            Err(message) => errors.push(ValidationError {
                file: path,
                message,
            }),
        }
    }

    (valid, errors)
}

fn validate_one(path: &Path, validator: &jsonschema::Validator) -> Result<Component, String> {
    let content = std::fs::read_to_string(path).map_err(|e| format!("could not read file: {e}"))?;
    let value: serde_json::Value =
        serde_yaml_ng::from_str(&content).map_err(|e| format!("could not parse YAML: {e}"))?;

    let schema_errors: Vec<String> = validator
        .iter_errors(&value)
        .map(|e| format!("{}: {}", e.instance_path(), e))
        .collect();
    if !schema_errors.is_empty() {
        return Err(schema_errors.join("; "));
    }

    let provenance_type = value.pointer("/provenance/type").and_then(|v| v.as_str());

    if provenance_type == Some("proprietary_analysis") {
        let legal_review = value
            .pointer("/provenance/legal_review")
            .and_then(|v| v.as_bool());
        if legal_review != Some(true) {
            return Err(
                "provenance.legal_review must be true when provenance.type is proprietary_analysis"
                    .to_string(),
            );
        }
    }

    if provenance_type == Some("open_source_derived") {
        let derived_from = value
            .pointer("/provenance/derived_from")
            .and_then(|v| v.as_str());
        if derived_from.map(str::is_empty).unwrap_or(true) {
            return Err(
                "provenance.derived_from must be a non-empty string when provenance.type is open_source_derived"
                    .to_string(),
            );
        }
    }

    if let Some(license) = value.get("license").and_then(|v| v.as_str()) {
        if !ALLOWED_LICENSES.contains(&license) {
            return Err(format!(
                "license '{license}' is not on the allow-list ({})",
                ALLOWED_LICENSES.join(", ")
            ));
        }
    }

    if let Some(implementations) = value.get("implementations").and_then(|v| v.as_array()) {
        for (i, impl_value) in implementations.iter().enumerate() {
            if let Some(license) = impl_value.get("license").and_then(|v| v.as_str()) {
                if !ALLOWED_LICENSES.contains(&license) {
                    return Err(format!(
                        "implementations[{i}].license '{license}' is not on the allow-list ({})",
                        ALLOWED_LICENSES.join(", ")
                    ));
                }
            }
        }
    }

    serde_json::from_value(value)
        .map_err(|e| format!("failed to deserialize into Component after passing validation: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_fixture_dir(files: &[(&str, &str)]) -> tempfile::TempDir {
        let dir = tempfile::tempdir().expect("could not create temp dir");
        for (name, content) in files {
            std::fs::write(dir.path().join(name), content).expect("could not write fixture file");
        }
        dir
    }

    fn validate_single(yaml: &str) -> Result<(), Vec<String>> {
        let dir = write_fixture_dir(&[("component.yaml", yaml)]);
        let (valid, errors) = load_and_validate_components(dir.path());
        if errors.is_empty() {
            assert_eq!(valid.len(), 1);
            Ok(())
        } else {
            Err(errors.into_iter().map(|e| e.message).collect())
        }
    }

    const MINIMAL_VALID: &str = r#"
name: Minimal Component
category: movement
summary: A minimal valid component.
capability:
  inputs: []
  outputs: []
  events: []
  dependencies: []
  determinism: none
implementations:
  - engine: engine-agnostic
    language: C#
    url: https://example.invalid/minimal
    license: MIT
license: MIT
provenance:
  type: original
"#;

    const FULL_EXAMPLE: &str = r#"
name: Quake-style Strafe Movement
category: movement
summary: Air-strafing bunny-hop movement with ground friction and acceleration caps.
description: >
  First-person movement controller modeled on Quake-engine air control:
  separate ground/air acceleration curves, speed gained through strafe-jump
  timing, and a friction model applied only while grounded.
reference_games:
  - Quake
  - Quake III Arena
genre_tags: [fps, arena-shooter]
capability:
  inputs:
    - { name: move_input, type: vector2, description: "Forward/strafe axes, -1..1" }
    - { name: jump_pressed, type: bool }
    - { name: grounded, type: bool }
  outputs:
    - { name: velocity, type: vector3, unit: "units/s" }
  events:
    - { name: Jumped }
    - { name: Landed }
    - { name: StartedFalling }
  dependencies: [PhysicsBody, Input]
  invariants:
    - "Friction is applied only while grounded."
    - "Air acceleration is capped independently of ground acceleration."
  determinism: best-effort
implementations:
  - engine: engine-agnostic
    language: C#
    url: https://example.invalid/REPLACE-WITH-A-REAL-VERIFIED-URL
    license: MIT
    maturity: usable
license: CC-BY-4.0
provenance:
  type: original
  notes: Written from publicly documented Quake movement-physics writeups, no engine source referenced.
version: 0.1.0
"#;

    #[test]
    fn minimal_valid_spec_passes() {
        assert_eq!(validate_single(MINIMAL_VALID), Ok(()));
    }

    #[test]
    fn full_example_from_spec_passes() {
        assert_eq!(validate_single(FULL_EXAMPLE), Ok(()));
    }

    #[test]
    fn missing_name_fails() {
        let yaml = MINIMAL_VALID.replacen("name: Minimal Component\n", "", 1);
        assert!(validate_single(&yaml).is_err());
    }

    #[test]
    fn missing_category_fails() {
        let yaml = MINIMAL_VALID.replacen("category: movement\n", "", 1);
        assert!(validate_single(&yaml).is_err());
    }

    #[test]
    fn missing_summary_fails() {
        let yaml = MINIMAL_VALID.replacen("summary: A minimal valid component.\n", "", 1);
        assert!(validate_single(&yaml).is_err());
    }

    #[test]
    fn missing_capability_fails() {
        let yaml = r#"
name: No Capability
category: movement
summary: Missing the capability block.
implementations:
  - engine: engine-agnostic
    language: C#
    url: https://example.invalid/x
    license: MIT
license: MIT
provenance:
  type: original
"#;
        assert!(validate_single(yaml).is_err());
    }

    #[test]
    fn missing_license_fails() {
        let yaml = r#"
name: No License
category: movement
summary: Missing the top-level license.
capability:
  inputs: []
  outputs: []
  events: []
  dependencies: []
  determinism: none
implementations:
  - engine: engine-agnostic
    language: C#
    url: https://example.invalid/x
    license: MIT
provenance:
  type: original
"#;
        assert!(validate_single(yaml).is_err());
    }

    #[test]
    fn missing_provenance_fails() {
        let yaml = r#"
name: No Provenance
category: movement
summary: Missing provenance.
capability:
  inputs: []
  outputs: []
  events: []
  dependencies: []
  determinism: none
implementations:
  - engine: engine-agnostic
    language: C#
    url: https://example.invalid/x
    license: MIT
license: MIT
"#;
        assert!(validate_single(yaml).is_err());
    }

    #[test]
    fn missing_implementations_fails() {
        let yaml = r#"
name: No Implementations
category: movement
summary: Missing implementations entirely.
capability:
  inputs: []
  outputs: []
  events: []
  dependencies: []
  determinism: none
license: MIT
provenance:
  type: original
"#;
        assert!(validate_single(yaml).is_err());
    }

    #[test]
    fn empty_implementations_array_fails() {
        let yaml = MINIMAL_VALID.replacen(
            "implementations:\n  - engine: engine-agnostic\n    language: C#\n    url: https://example.invalid/minimal\n    license: MIT\n",
            "implementations: []\n",
            1,
        );
        assert!(validate_single(&yaml).is_err());
    }

    #[test]
    fn category_outside_enum_fails() {
        let yaml =
            MINIMAL_VALID.replacen("category: movement\n", "category: not-a-real-category\n", 1);
        assert!(validate_single(&yaml).is_err());
    }

    #[test]
    fn engine_outside_enum_fails() {
        let yaml = MINIMAL_VALID.replacen("engine: engine-agnostic\n", "engine: CryEngine\n", 1);
        assert!(validate_single(&yaml).is_err());
    }

    #[test]
    fn implementation_missing_url_fails() {
        let yaml = MINIMAL_VALID.replacen("    url: https://example.invalid/minimal\n", "", 1);
        assert!(validate_single(&yaml).is_err());
    }

    #[test]
    fn genre_tag_not_lowercase_kebab_fails() {
        let yaml = MINIMAL_VALID.replacen(
            "summary: A minimal valid component.\n",
            "summary: A minimal valid component.\ngenre_tags: [FPS]\n",
            1,
        );
        assert!(validate_single(&yaml).is_err());
    }

    #[test]
    fn determinism_outside_enum_fails() {
        let yaml = MINIMAL_VALID.replacen("determinism: none\n", "determinism: sometimes\n", 1);
        assert!(validate_single(&yaml).is_err());
    }

    #[test]
    fn proprietary_analysis_without_legal_review_fails() {
        let yaml = MINIMAL_VALID.replacen(
            "provenance:\n  type: original\n",
            "provenance:\n  type: proprietary_analysis\n",
            1,
        );
        assert!(validate_single(&yaml).is_err());
    }

    #[test]
    fn proprietary_analysis_with_legal_review_true_passes() {
        let yaml = MINIMAL_VALID.replacen(
            "provenance:\n  type: original\n",
            "provenance:\n  type: proprietary_analysis\n  legal_review: true\n",
            1,
        );
        assert_eq!(validate_single(&yaml), Ok(()));
    }

    #[test]
    fn open_source_derived_without_derived_from_fails() {
        let yaml = MINIMAL_VALID.replacen(
            "provenance:\n  type: original\n",
            "provenance:\n  type: open_source_derived\n",
            1,
        );
        assert!(validate_single(&yaml).is_err());
    }

    #[test]
    fn open_source_derived_with_derived_from_passes() {
        let yaml = MINIMAL_VALID.replacen(
            "provenance:\n  type: original\n",
            "provenance:\n  type: open_source_derived\n  derived_from: https://example.invalid/upstream\n",
            1,
        );
        assert_eq!(validate_single(&yaml), Ok(()));
    }

    #[test]
    fn every_allow_listed_license_passes() {
        for license in ALLOWED_LICENSES {
            let yaml = MINIMAL_VALID.replace("license: MIT", &format!("license: {license}"));
            assert_eq!(
                validate_single(&yaml),
                Ok(()),
                "license {license} should be allowed"
            );
        }
    }

    #[test]
    fn unrecognized_license_fails() {
        let yaml = MINIMAL_VALID.replace("license: MIT", "license: WTFPL");
        assert!(validate_single(&yaml).is_err());
    }

    #[test]
    fn one_bad_file_does_not_block_the_rest() {
        let invalid = MINIMAL_VALID.replacen("category: movement\n", "", 1);
        let dir = write_fixture_dir(&[
            ("a-valid.yaml", MINIMAL_VALID),
            ("b-invalid.yaml", &invalid),
        ]);
        let (valid, errors) = load_and_validate_components(dir.path());
        assert_eq!(valid.len(), 1);
        assert_eq!(errors.len(), 1);
    }
}
