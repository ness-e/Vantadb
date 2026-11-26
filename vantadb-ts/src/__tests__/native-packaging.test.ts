import { describe, it, expect } from "vitest";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

/**
 * EGO-17: `dist/native.js` (from `src/native.ts`) does a runtime
 * `import("vantadb-node")` (`NativeVantaDB.connect`), so `vantadb-node`
 * must be a production dependency — devDependencies are pruned by
 * `npm prune --production`/asar packaging and the import would fail
 * at runtime by layout accident.
 */
const pkg = JSON.parse(
  readFileSync(join(dirname(fileURLToPath(import.meta.url)), "..", "..", "package.json"), "utf8"),
) as { dependencies?: Record<string, string>; devDependencies?: Record<string, string> };

describe("native packaging (EGO-17)", () => {
  it("declares vantadb-node as a production dependency", () => {
    expect(pkg.dependencies?.["vantadb-node"]).toBeTruthy();
  });

  it("does not rely on devDependencies for the native runtime", () => {
    expect(pkg.devDependencies?.["vantadb-node"]).toBeUndefined();
  });
});
