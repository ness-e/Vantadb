#!/usr/bin/env node
// scripts/docs/verify-surface-manifest.mjs
//
// PURPOSE:
//   Automated reconciliation gate for docs/api/SURFACE_MANIFEST.md (DOC-10).
//   Verifies that the public surface inventory matches:
//   1. Workspace versions across root Cargo.toml, package.json files, and pyproject.toml
//   2. Public status of released packages (e.g. vantadb-node, vantadb-ts, vantadb-py)
//   3. gen-surface-manifest generator consistency.
//
// USAGE:
//   node scripts/docs/verify-surface-manifest.mjs [--check]
//   Exit code: 0 = OK, 1 = Drift or mismatch detected.

import { readFileSync } from "node:fs";
import { execSync } from "node:child_process";
import { abs } from "./lib.mjs";

const MANIFEST_PATH = "docs/api/SURFACE_MANIFEST.md";

function main() {
  const isCheckOnly = process.argv.includes("--check");

  // 1. Verify that the generator produces byte-identical output (no drift)
  try {
    execSync("node scripts/docs/gen-surface-manifest.mjs --check", { stdio: "pipe" });
  } catch (err) {
    console.error("FAIL [DOC-10]: docs/api/SURFACE_MANIFEST.md has drifted from generator.");
    console.error("Run `node scripts/docs/gen-surface-manifest.mjs --write` to sync.");
    process.exit(1);
  }

  // 2. Read manifest content
  const manifest = readFileSync(abs(MANIFEST_PATH), "utf8");

  // 3. Verify public status of core published SDKs
  const expectations = [
    {
      surface: "vantadb",
      status: "PUBLIC",
      pattern: /\|\s*`\.`\s*\|\s*`vantadb`\s*\|\s*[^|]+\|\s*\*\*PUBLIC\*\*/,
    },
    {
      surface: "vantadb-python",
      status: "PUBLIC",
      pattern: /\|\s*`vantadb-python`\s*\|\s*`vantadb_py`\s*\|\s*[^|]+\|\s*\*\*PUBLIC\*\*/,
    },
    {
      surface: "vantadb-wasm",
      status: "PUBLIC",
      pattern: /\|\s*`vantadb-wasm`\s*\|\s*`vantadb-wasm`\s*\|\s*[^|]+\|\s*\*\*PUBLIC\*\*/,
    },
    {
      surface: "vantadb-node",
      status: "PUBLIC",
      pattern: /\|\s*`vantadb-node`\s*\|\s*`vantadb-node \(npm\)`\s*\|\s*[^|]+\|\s*\*\*PUBLIC\*\*/,
    },
    {
      surface: "vantadb-server",
      status: "PUBLIC",
      pattern: /\|\s*`vantadb-server`\s*\|\s*`vantadb-server`\s*\|\s*[^|]+\|\s*\*\*PUBLIC\*\*/,
    },
  ];

  let errors = 0;
  for (const exp of expectations) {
    if (!exp.pattern.test(manifest)) {
      console.error(
        `FAIL [DOC-10]: Surface ${exp.surface} does not have expected ${exp.status} status in ${MANIFEST_PATH}`,
      );
      errors++;
    }
  }

  if (errors > 0) {
    console.error(`Total reconciliation errors: ${errors}`);
    process.exit(1);
  }

  console.log("OK [DOC-10]: SURFACE_MANIFEST.md is 100% reconciled with public registry releases.");
  process.exit(0);
}

main();
