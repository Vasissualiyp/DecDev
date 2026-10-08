# MCP server spec (alpha)

An [MCP](https://modelcontextprotocol.io) server exposing the same three
read operations the CLI does, so an agent using either surface learns one
vocabulary. Rust crate `mcp/`, package/binary name `decdev-mcp`, a thin
wrapper over `decdev-core` — no new data source. Joins the Cargo workspace
(infra is Rust, never Node/TS — see `specs/00-overview.md`).

## Transport

stdio, newline-delimited JSON-RPC 2.0 (the MCP stdio transport: one
JSON-RPC message per line, no embedded newlines). No HTTP, no sockets, no
extra process. Messages are handled synchronously, one at a time.

Implemented directly against the spec rather than via an SDK: the surface
is small (four methods) and a hand-rolled dispatcher keeps the dependency
set to `serde_json` + `clap`, with no async runtime. The build plan allows
a Rust MCP SDK "if one fits cleanly"; this did not, so the protocol is
implemented directly.

## Startup

Same discovery/validation as the CLI and API: `decdev_core::resolve_sources`
+ `load_and_validate_sources`, plus repeatable `--source <DIR>`. Strict: any
invalid component prints every error to stderr and exits `1` — never serve
a partial catalog. The catalog is loaded once into memory; no refresh.

## Methods

- `initialize` → echoes the client's requested `protocolVersion` (falling
  back to the current MCP version if absent) with
  `capabilities.tools = {}` and `serverInfo {name: "decdev", version}`.
- `notifications/initialized` → notification, no response.
- `ping` → empty result.
- `tools/list` → the three tools below.
- `tools/call` → runs one tool.
- Any other request with an `id` → JSON-RPC error `-32601 method not found`.
  A message with no `id` is a notification and gets no response.

## Tools (names and arguments match the CLI commands)

- `list` — no arguments. Returns every component as `{slug, name,
  category}` records.
- `show` — `{ "slug": string }`. Returns the full component JSON, or an
  error result with `isError: true` and `unknown component: <slug>`.
- `search` — `{ "query": string }`. Returns matching components as the
  same `{slug, name, category}` records, using the shared
  `Component::matches_query` (case-insensitive over
  name/summary/description/reference_games) so it cannot drift from
  `decdev search`.

Tool results use the MCP content shape:
`{"content":[{"type":"text","text":"<json>"}],"isError":bool}`. A bad or
missing argument (`search` without `query`, `show` without `slug`) is a
tool error (`isError: true`), not a protocol error.

## Non-goals

No write tools, no resources/prompts/sampling, no HTTP/SSE transport, no
auth, no database, no `--source` fetching. Voting/bookmarking stays the
separate DB task; `api/` (`specs/08-api.md`) is the HTTP surface.
