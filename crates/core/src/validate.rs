use crate::model::Component;
use std::collections::BTreeMap;
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
    "GPL-3.0-or-later",
    "LGPL-3.0",
    "LGPL-3.0-or-later",
];

#[derive(Debug, Clone)]
pub struct ValidationError {
    pub file: PathBuf,
    pub message: String,
}

fn schema_validator() -> jsonschema::Validator {
    let schema_value: serde_json::Value = serde_json::from_str(SCHEMA_STR)
        .expect("embedded schema/component.schema.json must be valid JSON");
    jsonschema::validator_for(&schema_value)
        .expect("embedded schema/component.schema.json must be a valid JSON Schema")
}

/// Reads every `*.yaml` file under `dir`, validates each against
/// `schema/component.schema.json` plus the non-schema rules documented in
/// specs/01-capability-spec-format.md (legal_review, derived_from, license
/// allow-list), and returns (valid components, errors) — one bad file does
/// not block the rest. Both lists are ordered by filename for determinism.
/// Thin wrapper over `load_and_validate_sources` for the common
/// single-source case (the default `components/` directory).
pub fn load_and_validate_components(dir: &Path) -> (Vec<Component>, Vec<ValidationError>) {
    let dir = dir.to_path_buf();
    load_and_validate_sources(std::slice::from_ref(&dir))
}

/// Reads every `*.yaml` file from each source directory in order, validates
/// each with the same rules as `load_and_validate_components`, and merges
/// the results. This is what backs the CLI's repeatable `--source` flag:
/// users can point DecDev at their own additional directories of component
/// specs (their own recipes, a local fork, a third-party "plugin" repo)
/// without editing the central registry. The central repo's CI never passes
/// `--source`, so it stays hermetic — see `specs/04-cli.md`.
///
/// A duplicate slug across sources is a validation error rather than a
/// silent last-one-wins, since the "plugin" mechanism must not let one
/// source quietly shadow another's component. One bad file does not block
/// the rest.
pub fn load_and_validate_sources(sources: &[PathBuf]) -> (Vec<Component>, Vec<ValidationError>) {
    let validator = schema_validator();
    let mut valid = Vec::new();
    let mut errors = Vec::new();
    let mut seen: BTreeMap<String, PathBuf> = BTreeMap::new();

    for dir in sources {
        let entries = match std::fs::read_dir(dir) {
            Ok(entries) => entries,
            Err(e) => {
                errors.push(ValidationError {
                    file: dir.clone(),
                    message: format!("could not read directory: {e}"),
                });
                continue;
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
                Ok(component) => match seen.get(&component.slug) {
                    Some(previous) => errors.push(ValidationError {
                        file: path,
                        message: format!(
                            "duplicate component slug '{}' — already provided by {}",
                            component.slug,
                            previous.display()
                        ),
                    }),
                    None => {
                        seen.insert(component.slug.clone(), path.clone());
                        valid.push(component);
                    }
                },
                Err(message) => errors.push(ValidationError {
                    file: path,
                    message,
                }),
            }
        }
    }

    (valid, errors)
}

/// Validates a single file against the schema and the non-schema rules,
/// independent of any directory scan. Used by `decdev validate <path>` for
/// explicit file arguments. Recompiles the schema validator on each call —
/// fine at CLI-invocation scale, not meant for hot loops.
pub fn validate_file(path: &Path) -> Result<Component, String> {
    let validator = schema_validator();
    validate_one(path, &validator)
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

    let derived_from_present = value
        .pointer("/provenance/derived_from")
        .and_then(|v| v.as_str())
        .is_some_and(|s| !s.is_empty());

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

            // Extraction recipes are deliberately NOT tied to the literal
            // "open_source_derived" string — any provenance that isn't
            // proprietary_analysis and names a derived_from source
            // qualifies, so a future additional legitimate provenance
            // category needs no changes here. See
            // specs/01-capability-spec-format.md.
            if impl_value.get("extraction").is_some() {
                if provenance_type == Some("proprietary_analysis") {
                    return Err(format!(
                        "implementations[{i}].extraction is not allowed when provenance.type is proprietary_analysis — extraction recipes may only reference legitimately-licensed sources"
                    ));
                }
                if !derived_from_present {
                    return Err(format!(
                        "implementations[{i}].extraction requires a non-empty provenance.derived_from naming the upstream source"
                    ));
                }
            }
        }
    }

    let mut component: Component = serde_json::from_value(value).map_err(|e| {
        format!("failed to deserialize into Component after passing validation: {e}")
    })?;
    component.slug = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or_default()
        .to_string();
    Ok(component)
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

    const EXTRACTION_BLOCK: &str = "\n    extraction:\n      include: [\"src/physics/vehicle.cpp\"]\n      entry_points:\n        - { path: \"src/physics/vehicle.cpp\", symbol: \"VehicleStep\" }\n";

    fn with_extraction(provenance_block: &str) -> String {
        let with_impl = MINIMAL_VALID.replacen(
            "    license: MIT\n",
            &format!("    license: MIT{EXTRACTION_BLOCK}"),
            1,
        );
        with_impl.replacen("provenance:\n  type: original\n", provenance_block, 1)
    }

    #[test]
    fn extraction_without_derived_from_fails() {
        // provenance.type: original never sets derived_from, regardless of
        // extraction being present.
        let yaml = with_extraction("provenance:\n  type: original\n");
        assert!(validate_single(&yaml).is_err());
    }

    #[test]
    fn extraction_on_proprietary_analysis_fails_even_with_legal_review_and_derived_from() {
        // Extraction must never be reachable via the proprietary_analysis
        // path, no matter what else is set — this is the hard legal
        // boundary, not just a missing-field check.
        let yaml = with_extraction(
            "provenance:\n  type: proprietary_analysis\n  legal_review: true\n  derived_from: https://example.invalid/upstream\n",
        );
        assert!(validate_single(&yaml).is_err());
    }

    #[test]
    fn extraction_with_open_source_derived_and_derived_from_passes() {
        let yaml = with_extraction(
            "provenance:\n  type: open_source_derived\n  derived_from: https://example.invalid/upstream\n",
        );
        assert_eq!(validate_single(&yaml), Ok(()));
    }

    #[test]
    fn extraction_with_a_different_non_proprietary_provenance_type_also_passes() {
        // The deliberate genericity test: this is clean_room, NOT
        // open_source_derived, and extraction still passes because the
        // rule keys off derived_from + "not proprietary_analysis", never
        // the literal string "open_source_derived". A future additional
        // legitimate provenance category would pass this same way.
        let yaml = with_extraction(
            "provenance:\n  type: clean_room\n  derived_from: https://example.invalid/upstream\n",
        );
        assert_eq!(validate_single(&yaml), Ok(()));
    }

    #[test]
    fn extraction_missing_entry_points_fails_schema_validation() {
        let yaml = MINIMAL_VALID.replacen(
            "    license: MIT\n",
            "    license: MIT\n    extraction:\n      include: [\"src/a.cpp\"]\n",
            1,
        );
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

    #[test]
    fn sources_from_multiple_directories_are_merged() {
        let first = write_fixture_dir(&[("a.yaml", MINIMAL_VALID)]);
        let second = write_fixture_dir(&[("b.yaml", MINIMAL_VALID)]);
        let (valid, errors) =
            load_and_validate_sources(&[first.path().to_path_buf(), second.path().to_path_buf()]);
        assert!(errors.is_empty());
        assert_eq!(valid.len(), 2);
    }

    #[test]
    fn duplicate_slug_across_sources_is_an_error_not_a_silent_override() {
        let first = write_fixture_dir(&[("shared.yaml", MINIMAL_VALID)]);
        let second = write_fixture_dir(&[("shared.yaml", MINIMAL_VALID)]);
        let (valid, errors) =
            load_and_validate_sources(&[first.path().to_path_buf(), second.path().to_path_buf()]);
        assert_eq!(valid.len(), 1);
        assert_eq!(errors.len(), 1);
        assert!(
            errors[0]
                .message
                .contains("duplicate component slug 'shared'"),
            "unexpected message: {}",
            errors[0].message
        );
    }

    #[test]
    fn an_unreadable_source_does_not_block_the_other_sources() {
        let first = write_fixture_dir(&[("a.yaml", MINIMAL_VALID)]);
        let missing = first.path().join("does-not-exist");
        let (valid, errors) = load_and_validate_sources(&[first.path().to_path_buf(), missing]);
        assert_eq!(valid.len(), 1);
        assert_eq!(errors.len(), 1);
    }
}
