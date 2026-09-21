#!/usr/bin/env node
// build-wasm-types.mjs
//
// Replace the .d.ts that wasm-pack generates in `pkg/vantadb_wasm.d.ts` with
// a hand-written one from `vantadb-wasm/src/vantadb_wasm.d.ts`.
//
// Background: wasm-bindgen 0.2 + wasm-pack 0.15 generate a .d.ts that types
// most cross-boundary values as `any` (the macro cannot infer JsValue
// shapes). For npm consumers that breaks TypeScript autocomplete and forces
// every caller to wrap calls. The hand-written .d.ts preserves the runtime
// contract exactly while replacing every `any` with a concrete interface
// (and a few `unknown` slots on the FFI glue that consumers never touch).
//
// Build wiring: invoked by `.github/workflows/release-npm-61.yml` right
// after `wasm-pack build --release`.
//
// Usage:
//   node dev-tools/build-wasm-types.mjs          # apply (overwrite pkg/vantadb_wasm.d.ts)
//   node dev-tools/build-wasm-types.mjs --check  # fail if pkg/ would change (CI gate)
//
// Exit codes:
//   0  merge applied (or already in sync)
//   1  hand-written source not found
//   2  generated pkg/vantadb_wasm.d.ts not found
//   3  --check mode and pkg/ would change (CI gate)
//
// Source of truth: `vantadb-wasm/src/lib.rs` (Rust). When the Rust
// signatures change, update the hand-written .d.ts in the same PR.

import { readFileSync, writeFileSync, existsSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join, resolve } from "node:path";

const __dirname = dirname(fileURLToPath(import.meta.url));
const repoRoot = resolve(__dirname, "..");

const HAND_WRITTEN = join(repoRoot, "vantadb-wasm", "src", "vantadb_wasm.d.ts");
const TARGET = join(repoRoot, "vantadb-wasm", "pkg", "vantadb_wasm.d.ts");

const checkMode = process.argv.includes("--check");

if (!existsSync(HAND_WRITTEN)) {
    console.error(`build-wasm-types: hand-written source not found: ${HAND_WRITTEN}`);
    process.exit(1);
}

const handWritten = readFileSync(HAND_WRITTEN, "utf8");

if (checkMode) {
    if (!existsSync(TARGET)) {
        console.error(`build-wasm-types: --check FAIL (target missing): ${TARGET}`);
        process.exit(3);
    }
    const current = readFileSync(TARGET, "utf8");
    if (current === handWritten) {
        console.log("build-wasm-types: --check OK (pkg/vantadb_wasm.d.ts already in sync)");
        process.exit(0);
    }
    console.error(
        "build-wasm-types: --check FAIL (pkg/vantadb_wasm.d.ts would change — " +
            "did you forget to run this script after editing the hand-written source?)",
    );
    process.exit(3);
}

writeFileSync(TARGET, handWritten, "utf8");
console.log("build-wasm-types: wrote hand-written .d.ts to pkg/vantadb_wasm.d.ts");
