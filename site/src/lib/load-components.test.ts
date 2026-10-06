import { describe, it, expect, afterEach } from "vitest";
import { writeFileSync, mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import {
  loadComponents,
  distinctEngines,
  distinctImplementationLicenses,
  distinctMaturities,
  type Component,
} from "./load-components";

const tempDirs: string[] = [];

function writeFixture(components: Component[]): string {
  const dir = mkdtempSync(path.join(tmpdir(), "decdev-site-test-"));
  tempDirs.push(dir);
  const file = path.join(dir, "components.json");
  writeFileSync(file, JSON.stringify(components));
  return file;
}

afterEach(() => {
  while (tempDirs.length > 0) {
    rmSync(tempDirs.pop()!, { recursive: true, force: true });
  }
});

const BASE: Component = {
  slug: "test-component",
  name: "Test Component",
  category: "movement",
  summary: "A component for testing.",
  reference_games: [],
  genre_tags: [],
  capability: { inputs: [], outputs: [], events: [], dependencies: [], invariants: [], determinism: "none" },
  implementations: [],
  license: "MIT",
  provenance: { type: "original" },
};

describe("loadComponents", () => {
  it("loads and types the fixture shape correctly", () => {
    const file = writeFixture([BASE]);
    const components = loadComponents(file);
    expect(components).toHaveLength(1);
    expect(components[0].name).toBe("Test Component");
    expect(components[0].capability.determinism).toBe("none");
  });
});

describe("distinctEngines", () => {
  it("returns every distinct engine across all implementations, not just the first", () => {
    const component: Component = {
      ...BASE,
      implementations: [
        { engine: "Unity", language: "C#", url: "https://example.invalid/a", license: "MIT" },
        { engine: "Godot", language: "GDScript", url: "https://example.invalid/b", license: "MIT" },
      ],
    };
    expect(distinctEngines(component)).toEqual(["Unity", "Godot"]);
  });
});

describe("distinctImplementationLicenses", () => {
  it("returns every distinct implementation license, deduplicated", () => {
    const component: Component = {
      ...BASE,
      implementations: [
        { engine: "Unity", language: "C#", url: "https://example.invalid/a", license: "MIT" },
        { engine: "Godot", language: "GDScript", url: "https://example.invalid/b", license: "MIT" },
        { engine: "Unreal", language: "C++", url: "https://example.invalid/c", license: "GPL-3.0" },
      ],
    };
    expect(distinctImplementationLicenses(component)).toEqual(["MIT", "GPL-3.0"]);
  });
});

describe("distinctMaturities", () => {
  it("omits implementations with no maturity set", () => {
    const component: Component = {
      ...BASE,
      implementations: [
        { engine: "Unity", language: "C#", url: "https://example.invalid/a", license: "MIT", maturity: "usable" },
        { engine: "Godot", language: "GDScript", url: "https://example.invalid/b", license: "MIT" },
      ],
    };
    expect(distinctMaturities(component)).toEqual(["usable"]);
  });
});
