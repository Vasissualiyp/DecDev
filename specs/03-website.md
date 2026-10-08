# Website spec

Statically generated (Astro, `output: 'static'`). No server-rendered routes,
no API calls at runtime for core browsing/search — everything is baked in
at build time from `site/src/data/components.json`.

TypeScript/JS is the right tool here — this is the one part of the repo
where that's true by design, not by default (see `specs/00-overview.md`).
What's *not* true here: `site/` does not parse YAML and does not
re-implement schema validation. `load-components.ts` does exactly one
thing — `JSON.parse(readFileSync('src/data/components.json'))`, typed with
a hand-written TS interface matching the schema in
`specs/01-capability-spec-format.md` — and trusts the result is valid,
because `decdev export` (Rust, `crates/core`) already refused to emit it
otherwise. If a component is ever malformed by the time the site sees it,
that's a bug in the Rust side or the CI sequencing in
`specs/05-validation-ci.md`, not something `site/` should defend against
with its own re-validation.

## Pages

### `/` — browse/search

- Loads all components at build time via `load-components.ts` (i.e. from
  the generated `components.json`, not from `/components/*.yaml` — the
  website never touches the YAML files directly).
- Renders a filterable list: each row shows `name`, `category`, `summary`,
  `reference_games`, and a badge per **distinct** `engine` value across
  *all* of that component's `implementations` (not just `implementations[0]`
  — a component with a Unity and a Godot implementation must show both
  badges, since filtering by either engine must surface this row). Same for
  `license` badges if a component's implementations carry more than one
  distinct license (the top-level `license` field is the spec/doc license
  and is shown separately, labeled as such, not conflated with
  implementation licenses).
- Filter facets, each a simple checkbox/select group, computed at build time
  from the actual data (don't hardcode the option lists — derive them from
  what's present in `/components`, falling back to the schema's `enum` for
  `category` and `engine`):
  - capability (`category`)
  - genre (`genre_tags`)
  - engine (any match across `implementations[].engine`, not just the first)
  - license (any match across `implementations[].license`)
  - maturity (any match across `implementations[].maturity`)
- Render every row server-side (at build time) with its facet values as
  `data-*` attributes (e.g. `data-category`, `data-engines="unity godot"`,
  `data-licenses="mit apache-2.0"`). Facet filtering is a small client-side
  script that shows/hides rows by checking those attributes — no separate
  JSON endpoint or client-side data fetch needed; the data the script
  filters is already in the DOM. Keep this script dependency-free (plain
  DOM APIs), it doesn't need a framework.
- Free-text search box wired to Pagefind (see below) — this is the only
  piece that needs its own index; facet filtering above works directly on
  the rendered DOM.

### `/components/[slug]` — component detail

One static page per file in `/components`, generated via Astro's
`getStaticPaths`. Renders every field from the spec in the order given in
`specs/01-capability-spec-format.md`'s example: name, summary, description,
reference games, capability contract (inputs/outputs/events/dependencies/
invariants/determinism), implementations (each as a card with engine,
language, license, maturity, link to repo, link to demo if present),
compatibility, license, provenance, version.

The compatibility section lists the component's own
`compatibility` entries (linked to the referenced component's page) and,
derived, any other component whose `compatibility` points at this one
("Referenced by …") — so a relationship authored once shows on both
pages. An extraction recipe (`implementations[].extraction`) renders its
include paths, entry points, exclusions, dependencies and build
requirements when present.

Each implementation's `url` and `demo_url` render as plain outbound links.
Do not embed, iframe, or proxy anything from the linked repo — DecDev hosts
metadata, not the implementation (section C/I of `docs/architecture.md`).

### `/submit` — static instructions page

Not a form. Static page explaining: fork, add a YAML file under
`/components` following the template in `specs/01-capability-spec-format.md`,
run `decdev validate` locally (requires building the Rust CLI once — link
to the one-line `cargo install --path cli` setup), open a PR. Links to
`CONTRIBUTING.md`.

## Search (Pagefind)

Run `npx pagefind --site dist` as a post-build step (wire into the site's
`package.json` as `"postbuild": "pagefind --site dist"`). Pagefind indexes
the rendered static HTML, so no separate index-building code is needed —
just make sure searchable text (name, summary, description, reference_games,
genre_tags) actually renders as visible text on the pages above.

## Explicitly not in MVP

No accounts, no voting/bookmarking, no server-side search API, no dynamic
"compatible with" computation. These are alpha-tier (section L of
`docs/architecture.md`) and need a database that the MVP deliberately
doesn't have.
