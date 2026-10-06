# Validation & CI spec

## `crates/core` (`decdev-core`)

The only place schema/YAML logic lives, in Rust. Depended on by `cli/` via
a path dependency — never by `site/`, which only ever reads
`site/src/data/components.json` (see `specs/00-overview.md` and
`specs/02-repo-layout.md` for why).

Public API (`src/validate.rs`):

- `pub fn load_and_validate_components(dir: &Path) -> (Vec<Component>, Vec<ValidationError>)`
  — reads every `*.yaml` under `dir`, parses with `serde_yaml`, validates
  each against `schema/component.schema.json` (loaded once, compiled with
  the `jsonschema` crate) plus the two rules not expressible in JSON
  Schema (both from `specs/01-capability-spec-format.md`): if
  `provenance.type == "proprietary_analysis"`, require
  `provenance.legal_review == true`; if
  `provenance.type == "open_source_derived"`, require a non-empty
  `provenance.derived_from`.
- `ValidationError { file: PathBuf, message: String }` where `message` is
  a human-readable JSON Schema error path + reason (the `jsonschema`
  crate's error type formats this directly).

Do not duplicate this logic anywhere else — `cli/`'s commands all go
through this one function.

## License allow-list check

Part of the same validation pass in `crates/core`: `license` (top-level,
the spec's own license) and every `implementations[].license` must match
a short allow-list of recognizable SPDX identifiers (start with: `MIT`,
`Apache-2.0`, `BSD-2-Clause`, `BSD-3-Clause`, `CC-BY-4.0`, `CC0-1.0`,
`GPL-3.0`, `LGPL-3.0`). Anything else fails validation with a message
telling the contributor to use an SPDX identifier or open an issue if
their license genuinely isn't on the list yet — don't silently accept
arbitrary strings.

## `.github/workflows/ci.yml`

Triggers: `pull_request` and `push` to the default branch.

Steps, in order — each gates the next; a failure stops the pipeline (no
later step masks an earlier failure):

1. Checkout; setup Rust via `rust-toolchain.toml`; cache the Cargo
   registry and `target/`.
2. `cargo build --workspace`
3. `cargo test --workspace` (per `specs/07-testing.md`)
4. `cargo clippy --workspace -- -D warnings`
5. `cargo fmt --check`
6. `cargo run --release -p cli -- validate` with no arguments, i.e.
   against the real `/components` directory — this is a *data* check,
   distinct from the unit/integration tests in step 3 which run against
   fixtures. Fail the job on any validation error, printing every failing
   file's errors (not just the first).
7. Setup Node (version pinned in `site/package.json`'s `engines`, or LTS
   if unset); `npm ci --prefix site`.
8. `cargo run --release -p cli -- export > site/src/data/components.json`
   — if this fails (per `specs/04-cli.md`'s strict behavior), stop here;
   don't let the site build proceed on missing/partial data.
9. `npm run build --prefix site` (runs `astro build` + the Pagefind
   post-build step) — this is also where a component with a valid schema
   but a template-breaking edge case would surface, before merge.

## Required human gate (not automatable by CI)

CI can mechanically check that `legal_review: true` is present when
`provenance.type: proprietary_analysis` — it cannot and must not try to
determine whether that self-attestation is actually true. Document this
explicitly in `CONTRIBUTING.md`: PRs with `provenance.type:
proprietary_analysis` require a maintainer to actually read the notes and
confirm before merge, not just see the CI check go green.
