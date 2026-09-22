#!/usr/bin/env node
// fix-wasm-pkg-files.mjs
//
// Ensure the wasm-pack generated `pkg/package.json` ships every runtime file
// the glue code imports. wasm-pack's default `files` array omits `snippets/`
// (generated for `#[wasm_bindgen(inline_js = ...)]`), so the published tarball
// misses `snippets/<hash>/inline0.js` while `vantadb_wasm.js` imports it —
// any `import "vantadb-wasm"` then dies with ERR_MODULE_NOT_FOUND
// (caught by TS-07 smoke-pack, 2026-09-22, vantadb-wasm@0.6.0 broken).
//
// Build wiring: invoked by `.github/workflows/release-npm-61.yml` in the
// publish-wasm job, right after `wasm-pack build --release` (+ d.ts override).
//
// Usage:
//   node dev-tools/fix-wasm-pkg-files.mjs          # patch pkg/package.json in place
//
// Exit codes: 0 patched or already ok; 1 pkg/package.json not found.

import { existsSync, readFileSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join, resolve } from "node:path";

const __dirname = dirname(fileURLToPath(import.meta.url));
const repoRoot = resolve(__dirname, "..");

const MANIFEST = join(repoRoot, "vantadb-wasm", "pkg", "package.json");
const SNIPPETS_DIR = join(repoRoot, "vantadb-wasm", "pkg", "snippets");

if (!existsSync(MANIFEST)) {
    console.error(`fix-wasm-pkg-files: manifest not found: ${MANIFEST}`);
    process.exit(1);
}

const manifest = JSON.parse(readFileSync(MANIFEST, "utf8"));
if (!Array.isArray(manifest.files)) {
    manifest.files = [];
}

let changed = false;
if (existsSync(SNIPPETS_DIR) && !manifest.files.some((f) => f.startsWith("snippets"))) {
    manifest.files.push("snippets/");
    changed = true;
}

if (changed) {
    writeFileSync(MANIFEST, JSON.stringify(manifest, null, 2) + "\n", "utf8");
    console.log("fix-wasm-pkg-files: added snippets/ to pkg/package.json files");
} else {
    console.log("fix-wasm-pkg-files: pkg/package.json already ok (no change)");
}
