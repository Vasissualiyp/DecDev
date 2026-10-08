# DecDev — Architecture Proposal

Answers [prompt.md](../prompt.md) sections A–M. This is the design-of-record: the
specs in [`/specs`](../specs) are a mechanical translation of the decisions made
here. If you change a decision, update this file first, then the specs.

Working name for the project throughout: **DecDev**.

---

## M. The critical question first

**Is a general-purpose "LEGO for arbitrary game systems" feasible?**

No, not as literal plug-and-play binary/source composition across arbitrary
genres. The coupling the prompt itself names — shared physics timestep,
animation-driven root motion, camera/input/movement feedback loops, networking
authority models, save/load of cross-system state — is not incidental
sloppiness that better interfaces fix. It's load-bearing: a Souls-like lock-on
camera and a Quake movement controller disagree about what "forward" means
relative to input, about whether movement speed is a function of animation or
the reverse, and about who owns the authoritative position for netcode. Those
are semantic disagreements, not type mismatches. An adapter can reconcile
units and coordinate frames; it cannot invent a shared opinion about which
system is authoritative when two reference games made opposite choices on
purpose.

AI agents do not change this calculus today. They're good at inferring a
plausible interface from one codebase and writing plausible adapter glue code.
They cannot *verify* that the glue preserves the original behavior — that
needs human playtesting or a very good automated test oracle, and "does this
feel like Dark Souls combat" is not something current automated testing can
certify. So the automated pipeline in section D (source → IR → decoupled
implementation → published component) is honest as a *target*, dishonest as
something to promise for the MVP.

**Narrowest abstraction that's still compelling:** a catalog + declarative
spec format that gives developers and AI coding agents a standard vocabulary
for describing a capability's inputs, outputs, events, and invariants — used
as **documentation and code-generation scaffolding that a human or agent
writes implementation code against**, not as an executable composition
engine. The value is real even if automatic binary composition never ships:
search ("find a movement implementation with high accel, low air control"),
comparison across implementations, a stable naming convention for the thing
you're about to hand-build, and a spec an AI coding agent can use as a
contract while generating code. This is scoped to **narrow capability
families where coupling is genuinely shallow** (inventory grids, dialogue
trees, third-person cameras, basic lock-on targeting) — not an implicit
promise that FPS movement and Souls combat will one day snap together without
human engineering.

Everything below is designed so this honest, narrower product is what ships,
while leaving room (but making no promises) for deeper composition later.

---

## A. Product architecture

- **MVP**: a git-based catalog of declarative component *specs* (YAML/JSON,
  no runtime code required) + a statically generated search/browse website +
  a CLI that reads the same data. No backend service, no database, no code
  execution. Submissions arrive as pull requests.
- **Intermediate** (alpha, 1–3 months): read-only REST API and an MCP server
  wrapping the same static index; CI that validates schema + a
  provenance/license gate; a handful of real multi-engine implementations per
  capability to pressure-test that the schema isn't accidentally
  engine-biased. The catalog stays read-only and consumption-oriented: no
  accounts, votes, bookmarks, or any per-user state (see section H).
- **Long-term**: executable capability contracts (behavior-level, not just
  metadata), adapter generation tooling, and — only if the narrow-families bet
  pays off — a composition/build pipeline for the families where it actually
  works. This is explicitly *not* a monolithic engine; it's closer to a
  package manager with opinionated interface contracts for a handful of
  capability families.
- **What NOT to build initially**: automated decompilation/analysis pipeline,
  any hosted execution of untrusted code, runtime composition/adapter
  generation, accounts/payments, benchmarking infrastructure, a custom engine
  or IR runtime.

DecDev is a **PyPI-like take-from registry**: the product is a place to
discover and pull capability specs and their linked implementations, not a
community or social network. That framing is load-bearing — it is *why*
there is no database, no auth, and no per-user state anywhere in the plan.

**External component sources (community "plugins").** The CLI accepts
repeatable `--source <DIR>` directories of additional component specs,
validated with the same rules and merged with the central `components/`.
This is deliberately data-only: a source is a directory of YAML, never
executable code, so it adds no code-execution surface (section I) and no
hosted-fetch step (section C). The central registry's CI never passes
`--source`, so it stays hermetic and a pull request cannot make it load
external content; a duplicate slug across sources is a hard error rather
than a silent override. It gives contributors a sanctioned way to run their
own recipe sets (mostly FOSS) without forking DecDev or waiting on the
central catalog, while the project's own legal gate (section J) still
applies to everything DecDev itself loads and publishes. It is not, and
will not become, a mechanism for DecDev to host or endorse unauthorized
sources — it does not change section J.

## B. Game Capability IR (v0 — descriptive, not executable)

v0 is deliberately just a structured contract *description*, not a
behavioral/executable spec. It's the thing a human or AI agent reads before
writing code, not something a compiler lowers. Concrete schema and example
are in [`specs/01-capability-spec-format.md`](../specs/01-capability-spec-format.md).

Fields cover: capability name/category, inputs/outputs/events (name + type +
units), dependencies on other capabilities or primitives, invariants (plain
language — "velocity is zero-g while climbing"), determinism requirement
(`strict` / `best-effort` / `none`), version, reference games (inspiration
only, see section J), implementations (external links, not hosted code), and
license/provenance.

v1+ (not in scope now) would add actual behavioral semantics — e.g. a state
machine or timed-event sequence per capability — strong enough that two
implementations of the same v1 spec could be swapped with higher confidence.
That's a research-grade problem; v0 does not pretend to solve it.

One addition within v0's own descriptive scope: an `implementations[]`
entry can carry an `extraction` recipe — which files/entry-points/transitive
dependencies to pull out of a larger upstream project, rather than the
whole repository — for the case where the only real implementation of a
capability lives inside a much bigger permissively-licensed codebase. See
`specs/01-capability-spec-format.md`'s "Selective extraction" section and
section J below for why this is scoped hard to legitimately-licensed
sources only.

## C. Component model

A valid component = one spec file conforming to the v0 schema, with a
non-empty `license`, a non-empty `provenance.type`, and at least one
`implementations` entry (an external link is sufficient — DecDev does not
host implementation code in the MVP).

On the coupling list in the prompt (global state, update-order assumptions,
engine-specific APIs, hidden dependencies, etc.): the MVP spec format does not
attempt to solve these. It requires the *contributor* to have already done
the decoupling work by hand when they write the spec and implementation —
DecDev records the result (what the interface is) and is explicit that it
does not verify the claim beyond schema shape. This is a trust/curation
problem (section I/K), not a technical one the schema can paper over.

## D. AI decomposition pipeline

Realistic today: an AI agent can read an existing (human-owned,
appropriately-licensed) codebase and draft a first-pass spec for a module a
human has already identified as a capability boundary, and can draft
documentation of its inputs/outputs. Not realistic today, and not attempted in
MVP or alpha: automatically discovering capability boundaries in a coupled
codebase, automatically severing hidden dependencies, or verifying behavioral
equivalence after decoupling. Every stage past "draft a spec for a
human-identified module" needs a human in the loop. DecDev's MVP sidesteps
this entirely: components are contributed by humans who build or adapt the
implementation themselves, using the reference game as inspiration, not by
feeding a pipeline a decompiled binary.

## E. Composition/adaptation

Out of scope for MVP and alpha. When/if pursued, scope it to the narrow
capability families identified in section M first, and expect adapters to be
hand-written-with-AI-assistance per pair of implementations, not generated
automatically from the v0 metadata — v0 doesn't carry enough semantic
information to generate a correct adapter, only enough to tell a human which
adapters are worth attempting.

Extraction recipes (section B) make the *scoping* half of this — "which
files, out of a project with thousands, actually matter" — tractable for
agents, but don't change this section's conclusion: the recipe's
`entry_points` tell an agent what to adapt, writing the adapter itself is
still hand-written-with-AI-assistance work against the target component's
`capability` interface. Resolving multiple components' recipes together
(the "Minecraft worldgen + X + Y + Z" composed-request case) — detecting
conflicts, ordering extraction, merging build requirements — is explicitly
not attempted; each recipe resolves independently today.

## F. Agent interface

MVP: CLI only (`search`, `show`, `list`, `validate`) reading the static index
— see [`specs/04-cli.md`](../specs/04-cli.md). Alpha: add a read-only REST API
and an MCP server that are thin wrappers over the same index — no new
capability, just another transport. Explicitly deferred: `compose`, `adapt`,
`test`, `benchmark`, `publish`-as-automation — these imply capabilities
(runtime composition, hosted execution, automated benchmarking) that sections
A/E/I rule out for now. An agent in the alpha era composes components the way
a human does: reads specs, writes code, by hand or with its own coding tools.

## G. Runtime strategy

| Option | MVP fit | Long-term fit |
|---|---|---|
| 1. Custom minimal runtime | No — massive scope, contradicts "not another engine" | Only as a far-future research prototype, if ever |
| 2. Engine-agnostic, target existing engines | **Yes** — matches "spec + link out" model, zero runtime to maintain | Stays the default; most developers are already committed to an engine |
| 3. Intermediate runtime/ABI, multiple backends | No — nothing to compose yet | Plausible once v1 IR has real semantics, for the narrow families only |

Recommendation: option 2 for MVP and alpha, unconditionally. Revisit option 3
only after the narrow-families composition bet in section M has evidence
behind it.

## H. Website/database

MVP: **no database**. Each component is a YAML file in `/components` in this
repo, validated against a JSON Schema in CI. A build script compiles those
files into a static search index; the website is statically generated from
that index (no server-rendered dynamic queries needed at MVP traffic). This
is the "cheap and simple" instruction taken literally — hosting cost is
effectively $0, and git history *is* the provenance/audit trail for free.
Concrete layout: [`specs/02-repo-layout.md`](../specs/02-repo-layout.md) and
[`specs/03-website.md`](../specs/03-website.md).

Alpha: still **no database**. The alpha tier adds read-only HTTP/MCP
surfaces over the same files-in-git index, never mutable server state.
Accounts, votes, bookmarks and other per-user data were considered and
explicitly **rejected** — DecDev is a PyPI-like take-from registry, not a
social platform (section K), and building them would trade away the
zero-infra, no-secrets model for a nice-to-have signal. GitHub stars/forks
and pull requests remain the only social layer. If a *future* feature ever
genuinely needs mutable state, that is a deliberate architecture change to be
argued here first — not a default, and never justified by "a real site has a
database."

## I. Security and trust

MVP has no code execution surface at all — it's metadata plus outbound links,
which eliminates most of the listed risks (sandboxing, arbitrary code
execution, dependency attacks, malicious components-as-code) by construction.
What remains: license-violation risk (mitigated by a required `license` field
and a CI check against an allow-list of recognizable licenses) and
link-rot/malicious-link risk (mitigated by review-before-merge on every PR;
no auto-merge). Revisit sandboxing seriously only if/when DecDev ever runs
`install`/build steps on a contributor's behalf — not in MVP or alpha.

## J. Legal/IP architecture

Hard rule, non-negotiable in the schema: DecDev never hosts proprietary
source, ROMs, extracted assets, or binaries. `reference_games` is a list of
named inspirations only (free-text game names), never a pointer to any
proprietary material.

`provenance.type` is a required enum distinguishing: `original` (written from
scratch, inspired by public behavior only), `clean_room` (built to a written
behavioral spec without looking at original source), `open_source_derived`
(adapted from an existing OSS implementation — must also carry that
project's license), and `proprietary_analysis` (contributor examined
proprietary source/binaries as *research reference* under a permissive
reading of applicable law). Any spec with `provenance.type:
proprietary_analysis` is blocked in CI pending a `legal_review: true` field —
i.e. a human (ultimately a lawyer, not DecDev) has to affirmatively flag it
before it can merge. DecDev does not and cannot determine on its own whether
reverse-engineering-derived behavior is safe to publish in a given
jurisdiction; this gate exists to force the question to a human rather than
let CI silently wave it through. **This is not legal advice** — a real
contributor pipeline handling `proprietary_analysis` entries should involve
an actual IP lawyer before launch, not just this CI gate.

**On selective extraction specifically** (section B/E): the temptation
this project has already run into once is "the registry only stores
*metadata* — which files to pull from a larger project, not the project
itself — so proprietary sources are fine as extraction targets." They are
not. A precise, structured guide to extracting the functional subsystems
of a decompiled, unauthorized reproduction of a current commercial game is
the operationally useful artifact of that infringement, one layer of
indirection removed from hosting the bytes directly — it doesn't become
legitimate because DecDev itself never stores the source. The validation
rule (`provenance.derived_from` set, `provenance.type` not
`proprietary_analysis`) is deliberately generic — it doesn't hardcode the
string `open_source_derived` — specifically so future legitimate
provenance categories don't require a schema change, but "legitimate" is
load-bearing: it means a project whose license actually permits cloning,
reading, and reusing the code, not "the user claims to be authorized" for
a decompiled AAA title where no realistic authorization exists. If this
gets relitigated again, the answer is still no for proprietary sources,
regardless of how the storage layer is framed.

## K. Business/ecosystem strategy

MVP leans entirely on GitHub's existing social infrastructure instead of
building one: PRs are the contribution mechanism, git blame/history is
attribution, stars/forks are the only "rating" signal needed at this scale.
Developers contribute for the usual OSS reasons (solving their own game's
need and publishing the byproduct, discovery/attribution, portfolio value) —
no monetization, badges, or "verified/benchmarked" status in MVP or alpha;
those require infrastructure (sections A/I) deliberately deferred.

This is a deliberate product shape, not an unfinished one: DecDev is a
PyPI-like *take-from* registry. People come to discover, compare and pull
capability specs; contribution happens through git pull requests, and GitHub
already provides every social primitive (stars, forks, issues, blame) this
project needs. DecDev itself does not plan accounts, votes, bookmarks, or any
per-user state — see section H.

## L. Development plan

Concrete, file-level task lists: [`specs/06-build-plan.md`](../specs/06-build-plan.md).
Summary:

1. **One evening**: schema + ~10 hand-written component specs + a generated
   Markdown index. No website, no CLI beyond a one-off script.
2. **One week**: JSON Schema CI validation, statically generated searchable
   website (filter by capability/genre/engine/license), working CLI
   (`search`/`show`/`list`/`validate`), CONTRIBUTING guide + spec template,
   30–50 seeded components.
3. **1–3 month alpha**: read-only REST API + MCP server over the same index,
   manually-asserted (not computed) compatibility tags between a few
   component pairs, IR nudged toward v0.1 based on what the first 50+ real
   specs revealed was missing.
