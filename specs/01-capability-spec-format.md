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
          "demo_url": { "type": "string", "format": "uri" },
          "extraction": { "$ref": "#/$defs/extraction" }
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
    },
    "compatibility": {
      "type": "array",
      "description": "Manually-asserted relationships to other components, by slug. NOT computed — a maintainer adds an entry by hand after actually trying the pair together (see specs/01-capability-spec-format.md). Each entry is displayed on this component's page and, in the reciprocal direction, on the referenced component's page.",
      "items": {
        "type": "object",
        "required": ["with", "note"],
        "additionalProperties": false,
        "properties": {
          "with": { "type": "string", "minLength": 1, "description": "Slug of the other component. Must name a component that exists; a dangling reference fails validation." },
          "relation": {
            "type": "string",
            "enum": ["pairs-with", "conflicts-with", "supersedes"],
            "description": "Optional. Absent means 'pairs-with' by convention."
          },
          "note": { "type": "string", "minLength": 1, "description": "Plain-language note: what was actually tried and what happened." }
        }
      }
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
    },
    "extraction": {
      "type": "object",
      "required": ["include", "entry_points"],
      "additionalProperties": false,
      "description": "Present only when this implementation is a selectively-extracted subset of a larger upstream project, rather than a dedicated repo for this exact capability. Only valid when the component's provenance.derived_from is set and provenance.type is not proprietary_analysis. This describes the smallest useful boundary, not the whole upstream repository.",
      "properties": {
        "include": {
          "type": "array",
          "minItems": 1,
          "items": { "type": "string", "minLength": 1 },
          "description": "File/directory paths or globs, relative to the upstream repo root, that must be pulled — the full resolved set, including transitive internal dependencies within the upstream repo. This is the authoritative set of what to copy."
        },
        "entry_points": {
          "type": "array",
          "minItems": 1,
          "items": {
            "type": "object",
            "required": ["path"],
            "additionalProperties": false,
            "properties": {
              "path": { "type": "string", "minLength": 1 },
              "symbol": { "type": "string" },
              "description": { "type": "string" }
            }
          },
          "description": "The API surface within `include` that an adapter should call into to satisfy this component's own capability interface (the component's top-level `capability` block IS the standardized interface the adapter targets — there is no separate interface schema to define)."
        },
        "exclude": {
          "type": "array",
          "items": { "type": "string" },
          "description": "Sibling subsystems or paths explicitly NOT needed, named for contributor/reviewer clarity. Not mechanically enforced — `include` is already the authoritative set; this just documents what was deliberately left out and why (pair with `notes`)."
        },
        "external_dependencies": {
          "type": "array",
          "items": {
            "type": "object",
            "required": ["name"],
            "additionalProperties": false,
            "properties": {
              "name": { "type": "string" },
              "version": { "type": "string" },
              "purpose": { "type": "string" }
            }
          },
          "description": "Third-party libraries the extracted code needs that are not themselves part of the upstream repo."
        },
        "build_requirements": {
          "type": "string",
          "description": "Free-text build/runtime requirements: compiler/language standard, platform, engine version, etc."
        },
        "notes": { "type": "string" }
      }
    }
  }
}
```

Four validation rules are not expressible in plain JSON Schema (all
enforced in `crates/core/src/validate.rs`, see `05-validation-ci.md` — not
optional, all covered by required test cases in `specs/07-testing.md`):

1. If `provenance.type == "proprietary_analysis"`, then
   `provenance.legal_review` must be present and `true`, else the file
   fails validation.
2. If `provenance.type == "open_source_derived"`, then
   `provenance.derived_from` must be present (non-empty string), else the
   file fails validation — an "open source derived" component with no
   pointer to what it was derived from is not meaningfully attributed.
3. If any `implementations[].extraction` is present, then
   `provenance.derived_from` must be present AND `provenance.type` must
   NOT be `proprietary_analysis`. Deliberately generic — this does not
   check for the literal string `open_source_derived`, so any future
   additional legitimate provenance category that sets `derived_from`
   automatically supports extraction with no validator changes. See
   "Selective extraction" below for why this exists and what it does not
   permit.
4. Every `compatibility[].with` must name a known component's slug and
   must not be the component's own slug. Unlike rules 1–3 this is a
   **cross-file** rule — it can only be checked once the whole catalog is
   loaded, so it lives in `load_and_validate_sources` (not `validate_one`),
   and a single-file `decdev validate <path>` deliberately does not apply
   it. See "Compatibility assertions" below.

## `implementations[].url` and `demo_url` must be real — no exceptions

Every link in a component spec must point to something that actually
exists and that the contributor has personally verified resolves (a
real `curl`/browser check, not "it looks like a plausible URL"). This
includes not pattern-matching this very document's example below — its
URL uses the reserved `.invalid` TLD and says "REPLACE WITH A REAL
VERIFIED URL" specifically so it can never be mistaken for something to
copy. Inventing a plausible-looking `github.com/...` URL instead of
sourcing (or just leaving unfilled) a real one is fabrication, full stop.

This is enforced mechanically, not just by convention: CI runs a
link-check step (HEAD-request every `url`/`demo_url` in
`/components`, fail on anything that doesn't 2xx/3xx — see
`specs/05-validation-ci.md`), because "please don't invent links" is
not a control, it's a hope.

## Selective extraction from a larger upstream project

Most implementations are either a dedicated repo for exactly this
capability, or a whole small repo worth vendoring in full. Sometimes
neither is true: the only real-world implementation of a capability lives
inside a much larger project, and vendoring that entire project just to
get one subsystem is wasteful and makes the dependency unclear. The
`extraction` field on an `implementations[]` entry (see `$defs/extraction`
above) describes a recipe for pulling the **smallest useful boundary**
instead of the whole repository: which files to include (the full
resolved set, transitive internal dependencies already folded in), the
entry points an adapter should call into, what's explicitly excluded and
why, third-party dependencies the extracted code needs, and build
requirements.

**Hard legal boundary, not a style preference: `extraction` may only
reference legitimately-licensed sources.** The non-schema rule above
enforces this mechanically (`provenance.derived_from` required,
`provenance.type` must not be `proprietary_analysis`), but the rule exists
*because* of a specific, deliberate design decision: this mechanism must
never become a way to describe "which files to pull out of a decompiled
proprietary game." Analyzing an unauthorized decompilation of a current
commercial title and publishing a precise guide to extracting pieces of it
is not meaningfully different from redistributing the source itself — it's
the operationally useful artifact of the same infringement, one layer of
indirection removed. `extraction` is for permissively-licensed open-source
projects where cloning the repo and reading the code is already 100%
legitimate; it never gates on "the user says they're authorized" for
something that has no realistic authorization path (see `docs/architecture.md`
section J and the real example below, which was rejected precisely because
its only named use cases were current commercial AAA titles).

The adapter an agent writes to satisfy the component's `capability`
interface is still hand-written-with-AI-assistance code, same as any other
composition work (`docs/architecture.md` section E) — `extraction` makes
the *scoping* of what to pull tractable, it does not generate the adapter
and does not automate dependency resolution across multiple components'
recipes together. That remains explicitly future work.

Agent workflow using this field: search the registry → select a capability
→ inspect `provenance`/`license` (and confirm `extraction`, if present,
only names a legitimately-licensed `derived_from`) → read the recipe →
clone the upstream repo and pull `include` → write an adapter from
`entry_points` to the component's `capability` block → integrate. DecDev's
own role stops at describing the recipe — it never clones, fetches,
executes, or hosts any of the upstream source itself (same principle as
every other implementation link in the registry, see `docs/architecture.md`
sections C and I).

### Example: an extraction recipe (`/components/voxel-terrain-generation.yaml`, abridged)

```yaml
implementations:
  - engine: other
    language: Rust
    url: https://github.com/veloren/veloren
    license: GPL-3.0-or-later
    extraction:
      include:
        - "world/src/sim/"
        - "world/src/lib.rs"
      entry_points:
        - { path: "world/src/sim/mod.rs", symbol: "WorldSim::generate" }
      exclude:
        - "voxygen/"   # renderer — separate concern
        - "server/"    # networking — separate concern
      external_dependencies:
        - { name: noise, purpose: "Procedural noise functions." }
provenance:
  type: open_source_derived
  derived_from: https://github.com/veloren/veloren
```

See the real, full version of this file for the complete recipe —
every path and symbol in it was verified against the actual public
repository before being written down, not guessed at (the same standard
every link in this registry is held to, see above).

## Compatibility assertions

`compatibility` is an optional top-level array of hand-written
relationships to other components, by slug. It is explicitly **not
computed** — a maintainer adds an entry after actually trying the pair, so
the catalog carries human judgment that CI cannot derive (see
`docs/architecture.md` section E on why composition stays out of scope for
automation).

Each entry:

- `with` (required): the slug of the other component. Must name a
  component that exists and must not be the component itself — dangling
  and self references fail validation (rule 4 above).
- `relation` (optional): `pairs-with`, `conflicts-with`, or `supersedes`.
  An absent `relation` means `pairs-with`.
- `note` (required): plain language — what was tried, and what the
  integrator should watch for. Interface-level observations ("both drive
  horizontal velocity from the same move-input contract; the wall-run
  state must take over while active") are the expected form; a seed note
  is not a claim that the two have been run together.

A relationship is authored **once**, on one side of the pair. Both
`decdev show` and the component detail page derive the reciprocal
direction (`decdev_core::inbound_compatibility`) so the note appears on
both components' pages without being duplicated in the YAML. If a
reference is removed on its authored side, the reciprocal view disappears
with it — there is a single source of truth.

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
    url: https://example.invalid/REPLACE-WITH-A-REAL-VERIFIED-URL
    license: MIT
    maturity: usable
license: CC-BY-4.0
provenance:
  type: original
  notes: Written from publicly documented Quake movement-physics writeups, no engine source referenced.
version: 0.1.0
```
