# REST API spec (alpha)

Read-only HTTP API over the same catalog the CLI and site serve. Rust
crate `api/`, package/binary name `decdev-api`, built on `axum` + `tokio`.
Thin wrapper over `decdev-core` (see `specs/05-validation-ci.md`) — no new
data source, no database, no auth, no write endpoints. Joins the Cargo
workspace (infra is Rust, never Node/TS — see `specs/00-overview.md`).

## Startup

- Discover the `components/` directory by walking up from the working
  directory, plus optional repeatable `--source <DIR>` extras, via
  `decdev_core::resolve_sources` / `load_and_validate_sources` — identical
  discovery and validation semantics to the CLI (`specs/04-cli.md`).
- **Strict, like `decdev export`**: if any component fails validation,
  print every error to stderr and exit `1`. The API never serves a
  partially-invalid dataset.
- Load once into memory (`Arc<Vec<Component>>`); the catalog is immutable
  for the process lifetime. Refresh means restart — no hot reload, no DB
  (see `docs/architecture.md` section H).
- Binds `127.0.0.1:<port>` (default `8080`, overridable with `--port`).

## Endpoints

- `GET /components` → `200` JSON array of every component (same shape the
  CLI's `export` and the site's `components.json` use).
- `GET /components/{slug}` → `200` component JSON; `404` with
  `{"error":"unknown component: <slug>"}` when the slug doesn't exist.
- `GET /search?q=<query>` → `200` JSON array of components whose
  `name`/`summary`/`description`/`reference_games` contain `<query>`
  case-insensitively. The matching rule is `Component::matches_query` in
  `crates/core` — one definition shared with `decdev search`, so the CLI
  and API cannot drift. A missing or empty `q` is `400`
  `{"error":"missing required query parameter: q"}`. Zero results is `200`
  with `[]`, not an error.

Error responses are always `{"error": "<message>"}` alongside the
appropriate status code.

## Non-goals

No write endpoints, no auth/accounts, no database, no pagination or search
index (the whole catalog fits in memory at this scale), no hot reload, and
no fetching/cloning of `--source` directories. Accounts, votes, bookmarks
and any per-user state are explicitly **not planned** — DecDev is a
read-only, PyPI-like take-from registry (`docs/architecture.md` sections
A/H/K). The MCP server is its own crate (Tier 3 task 11).
