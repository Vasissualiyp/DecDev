# Specs overview

These specs implement the MVP decided in [`docs/architecture.md`](../docs/architecture.md).
They're written to be mechanical: follow them in order, don't improvise
architecture — if something seems to need a judgment call that isn't covered
here, stop and check against `docs/architecture.md` rather than guessing.

Read in this order:

1. [`01-capability-spec-format.md`](01-capability-spec-format.md) — the YAML
   schema every component must satisfy. Build this first; everything else
   reads these files.
2. [`02-repo-layout.md`](02-repo-layout.md) — where files live.
3. [`03-website.md`](03-website.md) — the static catalog site.
4. [`04-cli.md`](04-cli.md) — the CLI tool.
5. [`05-validation-ci.md`](05-validation-ci.md) — CI checks on every PR.
6. [`06-build-plan.md`](06-build-plan.md) — ordered task checklist with
   acceptance criteria, split into evening / week / alpha tiers.
7. [`07-testing.md`](07-testing.md) — test requirements for every
   crate/package. Not optional reading: `06-build-plan.md`'s acceptance
   criteria assume the test coverage described here, they don't restate
   it per task.

## Stack (decided, not a menu)

There are two implementation languages in this repo, split by what the code
*is*, not by convenience:

- **CLI and infra: Rust.** The shared schema-validation/loading library
  (`crates/core`), the CLI (`cli/`) and the alpha-tier REST API (`api/`)
  are Rust, compiled to native binaries. This is deliberate, not a
  default: the CLI is meant to be a tool people install and run the way
  they'd install `ripgrep` or `fd` — a single binary, no runtime
  dependency — not an npm package requiring a Node install. The MCP server
  (`specs/06-build-plan.md` Tier 3) is also Rust, for the same reason:
  it's infra, not presentation.
- **Website: TypeScript/JavaScript is fine, and expected.** `site/` is
  ordinary web tooling (Astro, Pagefind) — that's normal territory for
  JS/TS and there's no reason to fight it. This is the one explicit
  exception to the Rust-only rule above, scoped to `site/` only.

The two sides don't share code or a build system — they integrate through
one generated file: `decdev export` (a `cli/` command, see
`specs/04-cli.md`) writes the full validated component list as JSON, and
`site/` treats that JSON as its only data source. The website never parses
YAML or re-implements validation in TypeScript — validation has exactly one
implementation, in Rust, and the website just renders its output. This also
means `site/`'s tests don't need to cover validation at all (see
`specs/07-testing.md`) — by the time `site/` sees data, it's already valid.

Concretely:

- **Workspace (Rust side)**: a Cargo workspace — root `Cargo.toml` with
  `members = ["crates/core", "cli", "api"]`. See `specs/02-repo-layout.md`.
- **Schema validation**: JSON Schema (draft 2020-12), checked with the
  `jsonschema` crate; component files are YAML, parsed with `serde_yaml`
  into `serde`-derived structs. Lives in `crates/core`, the only place
  this logic exists.
- **CLI**: `clap` (derive API). Built and distributed as a single compiled
  binary (`cargo install --path cli`, or a prebuilt release binary via
  GitHub Releases).
- **Website (independent npm project, not part of the Cargo workspace)**:
  Astro, statically generated (`output: 'static'`, no SSR adapter) —
  zero-JS-by-default output, build-time data loading from the exported
  JSON. Pagefind for client-side full-text search, run as a post-build
  step. One small hand-authored `filter.js` (plain JS, no bundler, no
  TypeScript needed for ~50 lines of DOM show/hide) for facet filtering —
  see `specs/03-website.md`.
- **Hosting**: any static host with a free tier (GitHub Pages, Netlify,
  Cloudflare Pages). No server process, no database, no secrets required
  for the MVP.
- **CI**: GitHub Actions. Rust steps (`cargo build`, `cargo test`,
  `cargo clippy -- -D warnings`, `cargo fmt --check`) run first; the site
  build runs only after `decdev export` succeeds. See
  `specs/05-validation-ci.md`.
- **Testing**: `cargo test` plus `assert_cmd` (spawns the compiled `decdev`
  binary and asserts on stdout/exit codes) for the Rust side; a lightweight
  JS test runner (`vitest`, or even plain Node scripts run as a CI step —
  this is presentation-layer, don't overthink it) for `site/`, scoped to
  rendering only. See `specs/07-testing.md`.

Do not add a database, a backend server, a GC-language CLI, or a Node
dependency anywhere outside `site/` to the MVP. Section H of
`docs/architecture.md` explains the no-database part; this file explains
the language split.
