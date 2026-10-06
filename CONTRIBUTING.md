# Contributing a component

DecDev is a git-based catalog — there's no submission form, just a pull
request adding one YAML file.

## Steps

1. Fork the repository.
2. Add a file at `components/<your-slug>.yaml`. `<your-slug>` must be
   kebab-case (`^[a-z0-9]+(-[a-z0-9]+)*$`) and becomes the component's id —
   see [`specs/01-capability-spec-format.md`](specs/01-capability-spec-format.md)
   for the full field-by-field schema and a worked example.
3. Build the CLI once:
   ```sh
   cargo install --path cli
   ```
   (needs a Rust toolchain — `nix develop` in this repo provides one if you
   don't already have `cargo`/`rustc` on `PATH`).
4. Validate locally before opening a PR — this runs the exact check CI runs:
   ```sh
   decdev validate
   ```
5. Open a pull request.

## Hard requirements, checked by CI

- Every `implementations[].url` and `demo_url` must be a **real link you
  have personally verified resolves**. Do not invent one, including by
  copying the placeholder pattern from the spec's own example. This is
  enforced by an automated link-check — see
  [`specs/05-validation-ci.md`](specs/05-validation-ci.md) for why this
  rule exists (it's not hypothetical: an earlier version of this catalog
  shipped with 14 fabricated links before this check existed).
- `license` (top-level) and every `implementations[].license` must be an
  SPDX identifier from the allow-list in
  [`specs/05-validation-ci.md`](specs/05-validation-ci.md). Open an issue
  if your license genuinely isn't on it yet.
- No proprietary game source, ROMs, extracted assets, textures, models, or
  audio — ever, in any form. `reference_games` are named inspirations only.

## Provenance and the human review gate

Your `provenance.type` must honestly describe how the component was
created — `original`, `clean_room`, `open_source_derived` (with a
`derived_from` URL), or `proprietary_analysis`.

If you select `proprietary_analysis`, you must also set
`legal_review: true`. CI only checks that this flag is *present* — it
cannot and does not verify the underlying legal judgment. **A maintainer
will read your notes and confirm before merging** any PR with this
provenance type; green CI alone is not sufficient for that case. This is
not a substitute for an actual IP lawyer's opinion where one is warranted
— see [`docs/architecture.md`](docs/architecture.md) section J.

## Full spec

[`AGENTS.md`](AGENTS.md) and [`specs/`](specs/) are the source of truth for
everything above — read there first if anything here seems to conflict.
