import { describe, it, expect, beforeAll, afterAll } from "vitest";
import { execSync } from "node:child_process";
import { readFileSync, writeFileSync, existsSync } from "node:fs";
import path from "node:path";
import type { Component } from "../src/lib/load-components";

// Build-and-inspect-the-output integration tests (per specs/07-testing.md):
// write fixture data into the real src/data/components.json location, run
// the Astro build DIRECTLY (not via `npm run build`, which would trigger
// the prebuild hook and re-run `decdev export`, clobbering our fixture),
// then assert on the generated HTML. The original file is restored
// afterward regardless of test outcome.

const SITE_ROOT = path.resolve(__dirname, "..");
const DATA_PATH = path.join(SITE_ROOT, "src", "data", "components.json");
const DIST_DIR = path.join(SITE_ROOT, "dist");
const ASTRO_BIN = path.join(SITE_ROOT, "node_modules", ".bin", "astro");

const originalDataExisted = existsSync(DATA_PATH);
const originalData = originalDataExisted ? readFileSync(DATA_PATH, "utf-8") : null;

const XSS_NAME = '<script>alert(1)</script>';

const FIXTURES: Component[] = [
  {
    slug: "test-movement",
    name: "Test Movement Component",
    category: "movement",
    summary: "A fixture component for build-output tests.",
    reference_games: ["Test Game"],
    genre_tags: ["fps"],
    capability: {
      inputs: [],
      outputs: [],
      events: [],
      dependencies: [],
      invariants: [],
      determinism: "none",
    },
    implementations: [
      {
        engine: "Unity",
        language: "C#",
        url: "https://example.invalid/test-movement",
        license: "MIT",
      },
    ],
    license: "MIT",
    provenance: { type: "original" },
  },
  {
    slug: "xss-test",
    name: XSS_NAME,
    category: "camera",
    summary: "Escaping regression test fixture.",
    description: XSS_NAME,
    reference_games: [],
    genre_tags: [],
    capability: {
      inputs: [],
      outputs: [],
      events: [],
      dependencies: [],
      invariants: [],
      determinism: "none",
    },
    implementations: [
      {
        engine: "engine-agnostic",
        language: "C#",
        url: "https://example.invalid/xss-test",
        license: "MIT",
      },
    ],
    license: "MIT",
    provenance: { type: "original" },
  },
];

beforeAll(() => {
  writeFileSync(DATA_PATH, JSON.stringify(FIXTURES));
  execSync(`"${ASTRO_BIN}" build`, { cwd: SITE_ROOT, stdio: "pipe" });
}, 60_000);

afterAll(() => {
  if (originalDataExisted && originalData !== null) {
    writeFileSync(DATA_PATH, originalData);
  }
});

function readDist(relativePath: string): string {
  return readFileSync(path.join(DIST_DIR, relativePath), "utf-8");
}

describe("/ (index page)", () => {
  it("renders the fixture component's name and badges", () => {
    const html = readDist("index.html");
    expect(html).toContain("Test Movement Component");
    expect(html).toContain("Unity");
  });
});

describe("/components/[slug]", () => {
  it("renders the component's name and every implementation URL as a link", () => {
    const html = readDist(path.join("components", "test-movement", "index.html"));
    expect(html).toContain("Test Movement Component");
    expect(html).toContain('href="https://example.invalid/test-movement"');
  });
});

describe("/submit", () => {
  it("renders the contributor instructions", () => {
    const html = readDist(path.join("submit", "index.html"));
    expect(html).toContain("decdev validate");
  });
});

describe("HTML escaping", () => {
  it("renders a component name/description containing script tags as escaped text, not live markup", () => {
    const html = readDist(path.join("components", "xss-test", "index.html"));
    expect(html).not.toContain(XSS_NAME);
    expect(html).toContain("&lt;script&gt;");
  });

  it("escapes the same component on the index page listing too", () => {
    const html = readDist("index.html");
    expect(html).not.toContain(XSS_NAME);
  });
});
