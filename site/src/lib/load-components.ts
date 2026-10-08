import { readFileSync } from "node:fs";
import path from "node:path";

// Mirrors schema/component.schema.json / crates/core/src/model.rs. This file
// does exactly one thing — read the already-validated export — it does NOT
// parse YAML and does NOT re-implement schema validation (see
// specs/00-overview.md and specs/03-website.md for why).

export interface Field {
  name: string;
  type: string;
  unit?: string;
  description?: string;
}

export interface CapabilityEvent {
  name: string;
  payload?: string;
  description?: string;
}

export interface Capability {
  inputs: Field[];
  outputs: Field[];
  events: CapabilityEvent[];
  dependencies: string[];
  invariants: string[];
  determinism: string;
}

export interface Implementation {
  engine: string;
  language: string;
  url: string;
  license: string;
  maturity?: string;
  demo_url?: string;
  extraction?: Extraction;
}

/** A recipe for pulling the smallest useful subset of a larger upstream
 * project rather than vendoring the whole repository. Only valid when the
 * component's provenance.derived_from is set and provenance.type is not
 * proprietary_analysis — enforced in crates/core, not re-checked here (see
 * specs/00-overview.md on why site/ never re-implements validation). */
export interface Extraction {
  include: string[];
  entry_points: EntryPoint[];
  exclude?: string[];
  external_dependencies?: ExternalDependency[];
  build_requirements?: string;
  notes?: string;
}

export interface EntryPoint {
  path: string;
  symbol?: string;
  description?: string;
}

export interface ExternalDependency {
  name: string;
  version?: string;
  purpose?: string;
}

export interface Provenance {
  type: string;
  notes?: string;
  derived_from?: string;
  legal_review?: boolean;
}

/** One hand-written compatibility assertion, authored on this component and
 * pointing at another by slug. The reverse direction is derived for display,
 * see the detail page. Cross-file reference validity is enforced in
 * crates/core, not re-checked here. */
export interface Compatibility {
  with: string;
  relation?: string;
  note: string;
}

export interface Component {
  slug: string;
  name: string;
  category: string;
  summary: string;
  description?: string;
  reference_games: string[];
  genre_tags: string[];
  capability: Capability;
  implementations: Implementation[];
  compatibility?: Compatibility[];
  license: string;
  provenance: Provenance;
  version?: string;
}

// NOT import.meta.url-relative: Astro/Vite bundles this module into
// dist/.prerender/chunks/ at build time, so a path relative to the
// module's own URL breaks once bundled (it did, the first time this was
// tried). process.cwd() is stable — astro build/dev always run from
// site/, regardless of where the bundler physically puts this module.
const DATA_PATH = path.join(process.cwd(), "src", "data", "components.json");

export function loadComponents(dataPath: string = DATA_PATH): Component[] {
  const raw = readFileSync(dataPath, "utf-8");
  return JSON.parse(raw) as Component[];
}

/** Distinct engine values across ALL of a component's implementations —
 * never just implementations[0]. See the implementations[0]-only bug this
 * guards against, documented in specs/03-website.md and specs/07-testing.md. */
export function distinctEngines(component: Component): string[] {
  return [...new Set(component.implementations.map((i) => i.engine))];
}

/** Distinct implementation license values — NOT the top-level spec license. */
export function distinctImplementationLicenses(component: Component): string[] {
  return [...new Set(component.implementations.map((i) => i.license))];
}

export function distinctMaturities(component: Component): string[] {
  return [
    ...new Set(
      component.implementations
        .map((i) => i.maturity)
        .filter((m): m is string => Boolean(m)),
    ),
  ];
}
