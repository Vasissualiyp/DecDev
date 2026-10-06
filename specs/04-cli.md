# CLI spec

Binary crate `cli/`, package/binary name `decdev`, built with `clap`
(derive API), depending on `decdev-core` (`crates/core`) for all
loading/validation logic — the CLI itself contains no YAML parsing or
schema logic, only argument handling and output formatting. Reads
`/components/*.yaml` by walking up from the current working directory to
find a `components/` directory — don't hardcode an absolute path.

Packaging: `cargo install --path cli` installs a `decdev` binary on
`PATH`. For local development, `cargo run -p decdev -- <command>` from the
repo root. CI builds it once (`cargo build --release -p decdev`) and reuses
the binary for both the `validate` CI step and the `export` step feeding
the website build (see `specs/05-validation-ci.md`).

## Commands

### `decdev list [--category <cat>] [--json]`

Lists all **valid** components (optionally filtered by `category`);
components that failed validation are silently excluded here (that's
`validate`'s job to surface, not `list`'s). Default output: one line per
component as `<slug>\t<name>\t<category>`. With `--json`, prints the full
array as JSON.

### `decdev search <query> [--category <cat>] [--engine <engine>] [--license <license>] [--json]`

Case-insensitive substring match of `<query>` against `name`, `summary`,
`description`, and `reference_games`, over valid components only.
`--category` narrows by exact match on the component's top-level
`category`. `--engine` and `--license` narrow by exact match against
**any entry** in `implementations[].engine` / `implementations[].license`
respectively — not the top-level `license` field (that's the
spec/documentation license, a different thing; there is no flag to filter
by it in the MVP CLI since it's rarely what a developer is searching for).
Default output same row format as `list`; exit code `0` even with zero
results (zero results is not an error). `--json` as above.

### `decdev show <slug> [--json]`

Prints the full component spec for one slug, human-readable by default
(every field, labeled, in the order from the example in
`specs/01-capability-spec-format.md`), or raw JSON with `--json`. Exits `1`
with a clear error (`Unknown component: <slug>`) if the slug doesn't exist
— also print the closest-matching slugs (simple Levenshtein or substring
suggestion) to help a typo. Looks up by slug across valid components only.

### `decdev validate [path...]`

Runs `decdev_core::validate::load_and_validate_components()` against
either the given file path(s) or, with no arguments, every file under
`/components`. Prints one line per file: `OK <slug>` or `FAIL <slug>:
<reason>` (reason includes the JSON Schema error path, e.g.
`capability.determinism: must be one of strict, best-effort, none`). Exit
code `1` if any file fails. This is the same check CI runs — a contributor
should be able to run this locally before opening a PR.

### `decdev export`

Always full JSON, no filters, no flags — this command exists for exactly
one consumer: the website's build step (`site/src/data/components.json`
is produced by redirecting this command's stdout, see
`specs/02-repo-layout.md`). Unlike `list`/`search`, which quietly skip
invalid components, `export` is strict: if `load_and_validate_components()`
reports *any* error, `export` prints nothing to stdout, prints every
error to stderr, and exits `1` — the website must never silently build on
a partially-invalid dataset. On success, prints the full valid component
array as JSON to stdout and exits `0`.

## Non-goals

No `install`, `compose`, `adapt`, `test`, `benchmark`, or `publish` commands
in the MVP or alpha CLI — see section F of `docs/architecture.md` for why
each of those implies infrastructure (runtime composition, hosted execution,
automated benchmarking) that doesn't exist yet. Don't stub these as
not-yet-implemented commands either; omit them entirely so the CLI's surface
honestly reflects what the platform can do today.
