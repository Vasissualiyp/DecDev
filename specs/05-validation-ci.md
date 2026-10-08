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
- `pub fn load_and_validate_sources(sources: &[PathBuf]) -> (Vec<Component>, Vec<ValidationError>)`
  — the multi-source form behind the CLI's and API's `--source`: loads and
  validates every source dir in order, merges the results, and treats a
  duplicate slug across sources as a hard error rather than a silent
  override. `load_and_validate_components(dir)` delegates to it.
- `pub fn find_components_dir()` / `resolve_sources(extra)` (in
  `src/source.rs`) — the shared discovery helpers so the CLI and API pick
  up the same catalog. `Component::matches_query` (in `src/model.rs`) is
  the one definition of search matching, shared by `decdev search` and the
  API's `/search` (`specs/08-api.md`).

Do not duplicate this logic anywhere else — `cli/`'s commands, `api/`'s
handlers and `mcp/`'s tools all go through these shared functions.

## License allow-list check

Part of the same validation pass in `crates/core`: `license` (top-level,
the spec's own license) and every `implementations[].license` must match
a short allow-list of recognizable SPDX identifiers (start with: `MIT`,
`Apache-2.0`, `BSD-2-Clause`, `BSD-3-Clause`, `CC-BY-4.0`, `CC0-1.0`,
`GPL-3.0`, `GPL-3.0-or-later`, `LGPL-3.0`, `LGPL-3.0-or-later`). Anything
else fails validation with a message
telling the contributor to use an SPDX identifier or open an issue if
their license genuinely isn't on the list yet — don't silently accept
arbitrary strings.

## Link check

Every `implementations[].url` and `implementations[].demo_url` in
`/components` must resolve. This exists specifically because an AI
coding agent once filled an entire seed set with plausible-looking but
entirely fabricated `github.com/...` URLs (see git history/incident
notes around the first real seed components) — "don't invent links" as
a written rule alone did not prevent that, so it's also a mechanical CI
check, not optional.

Implementation: a small script (`scripts/check-links.sh` or a `cli/`
dev-only subcommand — either is fine, it does NOT need to go through
`decdev-core`'s validation pipeline since this is a network check, not a
schema check) that collects every `url`/`demo_url` from
`load_and_validate_components()`'s output and sends an HTTP
HEAD request to each (follow redirects; treat anything outside 2xx/3xx,
including timeouts, as a failure after one retry). Runs over the real
`/components` directory in CI, never as part of `cargo test` — network
calls don't belong in the unit/integration test suite (flaky, slow,
and not what `cargo test --workspace` is for). Fail the CI job on any
non-resolving link, printing the offending file and URL.

This check only catches a link being dead *right now* — it doesn't
verify the content behind it actually matches the spec (that's still a
human PR-review judgment call), and it can't run usefully offline. Both
are acceptable limits; "the link exists" is the floor, not the whole bar.

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
6. `cargo run --release -p decdev -- validate` with no arguments, i.e.
   against the real `/components` directory — this is a *data* check,
   distinct from the unit/integration tests in step 3 which run against
   fixtures. Fail the job on any validation error, printing every failing
   file's errors (not just the first).
7. Link check (see above) against the real `/components` directory. Fail
   the job on any non-resolving `url`/`demo_url`.
8. Setup Node (version pinned in `site/package.json`'s `engines`, or LTS
   if unset); `npm ci --prefix site`.
9. `cargo run --release -p decdev -- export > site/src/data/components.json`
   — if this fails (per `specs/04-cli.md`'s strict behavior), stop here;
   don't let the site build proceed on missing/partial data.
10. `npm run build --prefix site` (runs `astro build` + the Pagefind
    post-build step) — this is also where a component with a valid
    schema but a template-breaking edge case would surface, before merge.

## Required human gate (not automatable by CI)

CI can mechanically check that `legal_review: true` is present when
`provenance.type: proprietary_analysis` — it cannot and must not try to
determine whether that self-attestation is actually true. Document this
explicitly in `CONTRIBUTING.md`: PRs with `provenance.type:
proprietary_analysis` require a maintainer to actually read the notes and
confirm before merge, not just see the CI check go green.
