# Testing spec

Every crate/package in this repo (`crates/core`, `cli/`, `site/`, and the
alpha-tier `api`/MCP server when they exist) ships with tests. No task in
`specs/06-build-plan.md` is done when it merely "works when I tried it by
hand" — it's done when its behavior is pinned down by a test that fails if
the behavior regresses. "Extensively tested" below is deliberately
specific, not a vibe: each bullet is a concrete case a reviewer can check
for.

## `crates/core` (schema validation) — the highest-value tests in the repo

Rust's built-in `#[cfg(test)] mod tests` + `cargo test`. Everything else
(CLI, CI, the website's data) depends on this crate being correct, so it
gets the most thorough coverage. Use fixture YAML as `const` string
literals in the test module, or files under `crates/core/tests/fixtures/`
— not the real contents of `/components`, so these tests don't break every
time someone adds a component.

- A minimal valid spec (every required field, nothing else) passes.
- The full example from `specs/01-capability-spec-format.md` passes.
- One test per *required* top-level field, asserting validation fails with
  that field omitted (`name`, `category`, `summary`, `capability`,
  `license`, `provenance`, `implementations`).
- `category` rejects a value outside the enum.
- `implementations` rejects an empty array (`minItems: 1`).
- Each `implementations[]` entry rejects missing `engine`/`language`/`url`/
  `license`, and `engine` rejects a value outside its enum.
- `genre_tags` rejects an entry that isn't lowercase-kebab (e.g. `"FPS"`).
- `capability.determinism` rejects a value outside
  `strict`/`best-effort`/`none`.
- The non-schema rules (all three from `specs/01-capability-spec-format.md`):
  `provenance.type: proprietary_analysis` with `legal_review` absent or
  `false` fails; the same with `legal_review: true` passes.
  `provenance.type: open_source_derived` with `derived_from` absent or
  empty fails; the same with a non-empty `derived_from` passes.
  `implementations[].extraction` present with no `provenance.derived_from`
  fails; present with `provenance.type: proprietary_analysis` fails *even
  with* `legal_review: true` and a `derived_from` set (this is the hard
  legal boundary, not a missing-field check — test it explicitly, not just
  the missing-field case); present with `provenance.type:
  open_source_derived` and a `derived_from` set passes; present with some
  *other* non-`proprietary_analysis` provenance type (e.g. `clean_room`)
  and a `derived_from` set also passes — this last case is the one that
  actually proves the rule is generic and doesn't key off the literal
  string `open_source_derived`, don't skip it.
- `extraction.entry_points` missing (schema-required) fails.
- License allow-list check: one passing case per allow-listed license, one
  failing case for an arbitrary unrecognized string.
- `load_and_validate_components()` against a fixture directory containing
  one valid and one invalid file returns both the valid component *and*
  the error for the invalid one — i.e. one bad file must not silently
  swallow or block loading of the rest.
- `load_and_validate_sources()` (the `--source` backing function): two
  source directories each containing a valid component are merged into one
  result; the *same top-level slug* in two sources is a hard error naming
  the duplicate (never a silent last-one-wins override); a missing/
  unreadable source directory is reported as an error but does not block
  the other sources from loading.

## `cli/`

Integration tests in `cli/tests/`, using `assert_cmd` (spawns the compiled
`decdev` binary — fast enough in Rust that this is the standard approach,
unlike spawning a subprocess per test in Node) + `predicates` for
assertions on stdout/stderr/exit code, run with the process's working
directory set to a temp dir containing a fixture `components/` directory
(the loader walks up from the cwd, so `current_dir` is how tests point it
at fixtures), and/or an external fixture directory passed via `--source`:

- `list`: returns all fixture components; `--category` filters correctly;
  `--json` output is valid JSON matching the fixture data; a component
  that fails validation is silently excluded from the output.
- `search`: query matches on each of `name`/`summary`/`description`/
  `reference_games` individually (one test per field, so a future refactor
  that narrows the match fields is caught); a query matching nothing
  returns an empty result and exit code `0`; combining `--category` with a
  query narrows correctly; `--engine`/`--license` match a fixture component
  whose matching value is on a *non-first* `implementations[]` entry (this
  is the regression test for the implementations[0]-only bug — don't only
  test against fixtures with one implementation).
- `show`: known slug prints the full spec; unknown slug exits `1` and
  includes a suggestion for a close typo (test with a one-character-off
  slug against a fixture set); a fixture component with an
  `implementations[].extraction` block prints its `include` paths and
  `entry_points` in the output.
- `validate`: a fixture with one valid and one invalid file reports both
  correctly, exits `1`; an all-valid fixture set exits `0`; explicit path
  arguments validate only those files and ignore `--source`.
- `export`: an all-valid fixture set prints the full JSON array to stdout
  and exits `0`; a fixture set containing even one invalid file prints
  **nothing** to stdout, prints the error(s) to stderr, and exits `1` —
  this is the invariant the website build depends on, test it explicitly.
- `--source` (the external-component-sources / "plugin" mechanism): each
  of `list`, `search`, `show`, `export` includes a component that exists
  only in the external source directory; `validate` reports that source's
  files too; `--source` works even when no `components/` directory is
  discoverable from the cwd; a slug duplicated between the discovered
  `components/` and a `--source` makes `validate` fail and makes `export`
  all-or-nothing (nothing on stdout, exit `1`).

## `site/`

No validation logic to test here — that's `crates/core`'s job, and by the
time `site/` sees data it's already passed through `decdev export`'s
strict all-or-nothing gate (see the `export` test above). `site/`'s tests
are scoped to rendering only:

- `load-components.ts` unit tests (plain `vitest`, or even a plain Node
  script if that's simpler for this little code): given a fixture
  `components.json`, confirms the loaded/typed shape matches, and that any
  client-side filter logic re-implemented here (if any — prefer deriving
  facets at build time over duplicating CLI search logic) doesn't silently
  diverge from the CLI's semantics (e.g. the engine/license any-match-
  across-implementations rule from `specs/03-website.md`).
- Integration tests as a **build-and-inspect-the-output** test: run
  `astro build` against a fixture `components.json` (point the test's
  working copy of `src/data/components.json` at the fixture before
  building), then read the generated files under `dist/` with plain `fs`
  and assert on their HTML content with a string/regex check or a
  lightweight HTML parser (e.g. `node-html-parser`). At least one such
  test per page (`/`, `/components/[slug]`, `/submit`), e.g. asserting
  `dist/components/<fixture-slug>/index.html` contains the fixture
  component's `name` and every `implementations[].url` as an `href`.
- One escaping test: a fixture component whose `name` or `description`
  contains HTML-significant characters (e.g. `<script>`) renders as
  escaped text in the output HTML, not as live markup — Astro's `{expr}`
  interpolation escapes by default, but this is cheap insurance against a
  future refactor that uses `set:html` or similar and silently reopens an
  XSS hole on community-submitted content.
- A fixture component whose implementation carries an `extraction` block
  renders its `include` paths and `entry_points` on the detail page.

## Coverage expectation

Not a specific percentage threshold — percentage targets invite testing
trivial code to hit a number. Instead: every *branch* in validation logic
(each schema rule, each CLI flag combination, each error path) has at least
one test exercising it, per the bullets above. If you add a new required
field, a new CLI flag, or a new error condition and don't add a
corresponding test case in this file's lists, update this spec file in the
same PR so the list stays the actual coverage contract, not aspirational
documentation.

## CI enforcement

`.github/workflows/ci.yml` (per `specs/05-validation-ci.md`) runs
`cargo test --workspace` as its own required step before the site build
even starts, and (once `site/` exists) its own test command for that
package — a PR that breaks a test fails CI the same way one that adds an
invalid component spec does.
