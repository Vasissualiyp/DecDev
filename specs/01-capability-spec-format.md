# Capability spec format (Game Capability IR v0)

Descriptive metadata only — not executable. See `docs/architecture.md`
section B for why. One YAML file per component, validated against the JSON
Schema below.

## File location and naming

`/components/<slug>.yaml`, where `<slug>` is kebab-case, globally unique,
matching `^[a-z0-9]+(-[a-z0-9]+)*$`. The filename (minus extension) IS the
component id — don't also put an `id` field that could drift out of sync.

## JSON Schema

Save as `/schema/component.schema.json`.

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://decdev.example/schema/component.schema.json",
  "title": "DecDev Component Spec",
  "type": "object",
  "required": ["name", "category", "summary", "capability", "license", "provenance", "implementations"],
  "additionalProperties": false,
  "properties": {
    "name": { "type": "string", "minLength": 1 },
    "category": {
      "type": "string",
      "enum": ["movement", "camera", "combat", "inventory", "dialogue", "quest",
        "procgen", "vehicle", "character-controller", "enemy-ai", "animation",
        "physics", "rendering", "networking", "other"]
    },
    "summary": { "type": "string", "minLength": 1, "maxLength": 280 },
    "description": { "type": "string" },
    "reference_games": {
      "type": "array",
      "items": { "type": "string", "minLength": 1 },
      "description": "Named inspirations only. Never a pointer to proprietary source/assets."
    },
    "genre_tags": {
      "type": "array",
      "items": { "type": "string", "pattern": "^[a-z0-9]+(-[a-z0-9]+)*$" },
      "description": "Lowercase kebab-case only (e.g. 'fps', 'arena-shooter') so facets on the site don't fragment into near-duplicate casings."
    },
    "capability": {
      "type": "object",
      "required": ["inputs", "outputs", "events", "dependencies", "determinism"],
      "additionalProperties": false,
      "properties": {
        "inputs": { "type": "array", "items": { "$ref": "#/$defs/field" } },
        "outputs": { "type": "array", "items": { "$ref": "#/$defs/field" } },
        "events": { "type": "array", "items": { "$ref": "#/$defs/event" } },
        "dependencies": {
          "type": "array",
          "items": { "type": "string" },
          "description": "Other capability categories or primitives this assumes (e.g. 'PhysicsBody', 'Input')."
        },
        "invariants": {
          "type": "array",
          "items": { "type": "string" },
          "description": "Plain-language behavioral guarantees, e.g. 'velocity is zero on the vertical axis while climbing'."
        },
        "determinism": { "type": "string", "enum": ["strict", "best-effort", "none"] }
      }
    },
    "implementations": {
      "type": "array",
      "minItems": 1,
      "items": {
        "type": "object",
        "required": ["engine", "language", "url", "license"],
        "additionalProperties": false,
        "properties": {
          "engine": {
            "type": "string",
            "enum": ["Unity", "Unreal", "Godot", "engine-agnostic", "other"],
            "description": "Fixed vocabulary, not free text — an 'engine' facet with inconsistent casing/spelling across contributors ('unity' vs 'Unity' vs 'Unity3D') is useless as a filter. Use 'other' plus a note in description if the real engine isn't listed yet, rather than inventing a new string."
          },
          "language": { "type": "string" },
          "url": { "type": "string", "format": "uri" },
          "license": { "type": "string" },
          "maturity": { "type": "string", "enum": ["prototype", "usable", "production-tested"] },
          "demo_url": { "type": "string", "format": "uri" }
        }
      }
    },
    "license": {
      "type": "string",
      "description": "License of the spec/documentation itself. Use an SPDX identifier, e.g. 'MIT', 'CC-BY-4.0'."
    },
    "provenance": {
      "type": "object",
      "required": ["type"],
      "additionalProperties": false,
      "properties": {
        "type": {
          "type": "string",
          "enum": ["original", "clean_room", "open_source_derived", "proprietary_analysis"]
        },
        "notes": { "type": "string" },
        "derived_from": { "type": "string", "description": "URL, required if type is open_source_derived." },
        "legal_review": {
          "type": "boolean",
          "description": "Required and must be true if provenance.type is 'proprietary_analysis'. See docs/architecture.md section J."
        }
      }
    },
    "version": {
      "type": "string",
      "pattern": "^\\d+\\.\\d+\\.\\d+$",
      "description": "Optional; treat an absent version as 0.1.0 by convention. Note: ajv does not apply JSON Schema 'default' values unless instantiated with useDefaults: true, and this spec does not require that — don't rely on a default magically appearing on the parsed object."
    }
  },
  "$defs": {
    "field": {
      "type": "object",
      "required": ["name", "type"],
      "additionalProperties": false,
      "properties": {
        "name": { "type": "string" },
        "type": { "type": "string" },
        "unit": { "type": "string" },
        "description": { "type": "string" }
      }
    },
    "event": {
      "type": "object",
      "required": ["name"],
      "additionalProperties": false,
      "properties": {
        "name": { "type": "string" },
        "payload": { "type": "string" },
        "description": { "type": "string" }
      }
    }
  }
}
```

Two validation rules are not expressible in plain JSON Schema (both enforced
in `scripts/lib/load.ts`, see `05-validation-ci.md` — not optional, both are
covered by required test cases in `specs/07-testing.md`):

1. If `provenance.type == "proprietary_analysis"`, then
   `provenance.legal_review` must be present and `true`, else the file
   fails validation.
2. If `provenance.type == "open_source_derived"`, then
   `provenance.derived_from` must be present (non-empty string), else the
   file fails validation — an "open source derived" component with no
   pointer to what it was derived from is not meaningfully attributed.

## Example: `/components/quake-strafe-movement.yaml`

```yaml
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
    url: https://github.com/example/quake-movement-csharp
    license: MIT
    maturity: usable
license: CC-BY-4.0
provenance:
  type: original
  notes: Written from publicly documented Quake movement-physics writeups, no engine source referenced.
version: 0.1.0
```
