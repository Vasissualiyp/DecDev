import { readFileSync } from "node:fs";
import path from "node:path";

// Reads enum lists directly from schema/component.schema.json for building
// facet option lists — this is reading the schema's own authoritative
// vocabulary for a UI, not re-implementing or re-running validation (that
// stays entirely in crates/core; see specs/00-overview.md).
//
// Resolved via process.cwd(), not import.meta.url: Astro/Vite bundles this
// module into dist/.prerender/chunks/ at build time, which breaks a path
// relative to the module's own URL (see the identical fix and longer note
// in load-components.ts — this bit that file for real before it was fixed
// here too). astro build/dev always run from site/, so cwd -> .. -> repo
// root is stable regardless of where the bundler puts this module.
const SCHEMA_PATH = path.join(process.cwd(), "..", "schema", "component.schema.json");

interface JsonSchema {
  properties: {
    category: { enum: string[] };
    implementations: {
      items: {
        properties: {
          engine: { enum: string[] };
          maturity: { enum: string[] };
        };
      };
    };
  };
}

function readSchema(schemaPath: string = SCHEMA_PATH): JsonSchema {
  return JSON.parse(readFileSync(schemaPath, "utf-8")) as JsonSchema;
}

export function categoryEnum(schemaPath?: string): string[] {
  return readSchema(schemaPath).properties.category.enum;
}

export function engineEnum(schemaPath?: string): string[] {
  return readSchema(schemaPath).properties.implementations.items.properties.engine.enum;
}

export function maturityEnum(schemaPath?: string): string[] {
  return readSchema(schemaPath).properties.implementations.items.properties.maturity.enum;
}
