#!/usr/bin/env node
// DIST-17 — Cross-language conformance comparator.
//
// Reads the per-binding artifacts emitted by the three producer tests
// (`vantadb-python/tests/test_cross_language_parity.py`,
// `vantadb-ts/src/__tests__/parity.test.ts`, `vantadb-node/tests/parity.test.ts`)
// from `target/bindings-parity/`, canonicalizes them under the rules below,
// prints a canonical SHA-256 per step (across the bindings that ran it), and
// exits non-zero on any divergence. Per-step hashes stay comparable when a
// binding declares an exclusion (the excluded step is simply not in its set).
//
// Canonicalization (documented — docs/api/BINDINGS_NAMESPACES.md §Conformance):
//   - put        rows sorted by key; `node_id` compared as decimal string (exact)
//   - search     rows sorted by (score desc, id asc); scores quantized to 6
//                decimal places (≈5e-7 tolerance). Ties in the core's stable
//                sort are HashMap-order dependent, so order among equal scores
//                is not semantic; rank for distinct scores is score-derived.
//   - graph_bfs  sequence compared as-is (the core sorts each BFS level —
//                src/graph.rs::bfs_traverse)
//   - iql        `write_node_id` exact; `read_ids` sorted asc
//   - Wall-clock fields (created_at_ms, updated_at_ms, valid_at_ms,
//     last_accessed) are NOT part of the projection.
//
// Exclusions: a binding may declare steps it does not run (`excluded`), but
// every step must still be covered by at least two bindings — a step silently
// covered by one binding is a harness error, not a pass.
//
// Usage: node dev-tools/parity-compare.mjs
// Exit:  0 = all covered steps identical; 1 = divergence / missing artifact /
//        bad coverage (fail loud, never a vacuous pass).

import { createHash } from "node:crypto";
import { existsSync, readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = fileURLToPath(new URL("..", import.meta.url));
const ARTIFACT_DIR = path.join(ROOT, "target", "bindings-parity");
const FIXTURE = path.join(ROOT, "tests", "parity", "scenario.json");
const BINDINGS = ["python", "wasm", "node"];
const STEPS = ["put", "search", "graph_bfs", "iql"];
const MIN_STEP_COVERAGE = 2; // a step run by a single binding proves nothing

function fail(message) {
  console.error(`PARITY FAIL: ${message}`);
  process.exit(1);
}

/** Deterministic stringify (sorted object keys) — the hash input. */
function stableStringify(value) {
  if (Array.isArray(value)) {
    return `[${value.map(stableStringify).join(",")}]`;
  }
  if (value !== null && typeof value === "object") {
    const body = Object.keys(value)
      .sort()
      .map((key) => `${JSON.stringify(key)}:${stableStringify(value[key])}`)
      .join(",");
    return `{${body}}`;
  }
  return JSON.stringify(value);
}

/** First diverging path between two canonical values, or null when equal. */
function firstDifference(a, b, at = "$") {
  if (Array.isArray(a) && Array.isArray(b)) {
    if (a.length !== b.length) {
      return `${at}.length: ${a.length} != ${b.length}`;
    }
    for (let i = 0; i < a.length; i += 1) {
      const diff = firstDifference(a[i], b[i], `${at}[${i}]`);
      if (diff) return diff;
    }
    return null;
  }
  if (a !== null && b !== null && typeof a === "object" && typeof b === "object") {
    const keys = [...new Set([...Object.keys(a), ...Object.keys(b)])].sort();
    for (const key of keys) {
      const diff = firstDifference(a[key], b[key], `${at}.${key}`);
      if (diff) return diff;
    }
    return null;
  }
  return Object.is(a, b) ? null : `${at}: ${JSON.stringify(a)} != ${JSON.stringify(b)}`;
}

function loadArtifacts() {
  const artifacts = new Map();
  for (const binding of BINDINGS) {
    const file = path.join(ARTIFACT_DIR, `${binding}.json`);
    if (!existsSync(file)) {
      fail(`missing artifact ${path.relative(ROOT, file)} — run the ${binding} producer test first`);
    }
    let artifact;
    try {
      artifact = JSON.parse(readFileSync(file, "utf8"));
    } catch (error) {
      fail(`unreadable artifact ${file}: ${error.message}`);
    }
    if (artifact.binding !== binding) {
      fail(`${file}: binding=${JSON.stringify(artifact.binding)}, expected ${JSON.stringify(binding)}`);
    }
    if (!artifact.steps || typeof artifact.steps !== "object") {
      fail(`${file}: missing "steps" object`);
    }
    if (!Array.isArray(artifact.excluded)) {
      fail(`${file}: missing "excluded" array (use [] when nothing is excluded)`);
    }
    artifacts.set(binding, artifact);
  }

  const scenario = artifacts.get(BINDINGS[0]).scenario;
  for (const [binding, artifact] of artifacts) {
    if (artifact.scenario !== scenario) {
      fail(`scenario mismatch: ${binding}=${JSON.stringify(artifact.scenario)}, expected ${JSON.stringify(scenario)}`);
    }
  }

  // Guard against stale local artifacts: the artifacts must reference the
  // scenario the fixture currently declares (in CI the producers always run
  // fresh, but a stale `target/` could otherwise compare yesterday's runs).
  let fixture;
  try {
    fixture = JSON.parse(readFileSync(FIXTURE, "utf8"));
  } catch (error) {
    fail(`unreadable fixture ${path.relative(ROOT, FIXTURE)}: ${error.message}`);
  }
  if (fixture.scenario !== scenario) {
    fail(`artifacts use scenario ${JSON.stringify(scenario)} but the fixture declares ${JSON.stringify(fixture.scenario)} — re-run the producers`);
  }
  return { artifacts, scenario };
}

function quantizeScore(score) {
  if (typeof score !== "number" || !Number.isFinite(score)) {
    fail(`search score must be a finite number, got ${JSON.stringify(score)}`);
  }
  return Math.round(score * 1e6) / 1e6;
}

function canonicalizeStep(step, value, binding) {
  switch (step) {
    case "put": {
      if (!Array.isArray(value) || value.length === 0) {
        fail(`${binding}.put: expected a non-empty array`);
      }
      return [...value]
        .sort((a, b) => String(a.key).localeCompare(String(b.key)))
        .map((row) => ({ key: String(row.key), node_id: String(row.node_id) }));
    }
    case "search": {
      if (!Array.isArray(value) || value.length === 0) {
        fail(`${binding}.search: expected a non-empty array`);
      }
      return [...value]
        .map((row) => ({ id: String(row.id), score: quantizeScore(row.score) }))
        .sort((a, b) => b.score - a.score || a.id.localeCompare(b.id));
    }
    case "graph_bfs": {
      if (!Array.isArray(value) || value.length === 0) {
        fail(`${binding}.graph_bfs: expected a non-empty array`);
      }
      return value.map(String);
    }
    case "iql": {
      if (value === null || typeof value !== "object") {
        fail(`${binding}.iql: expected an object`);
      }
      if (!Array.isArray(value.read_ids)) {
        fail(`${binding}.iql.read_ids: expected an array`);
      }
      return {
        write_node_id: String(value.write_node_id),
        read_ids: [...value.read_ids].map(String).sort(),
      };
    }
    default:
      fail(`unknown step ${JSON.stringify(step)}`);
  }
}

function canonicalize(binding, artifact) {
  const excluded = new Set(artifact.excluded);
  for (const step of excluded) {
    if (!STEPS.includes(step)) {
      fail(`${binding}: unknown excluded step ${JSON.stringify(step)} (known: ${STEPS.join(", ")})`);
    }
  }
  const steps = {};
  const missing = [];
  for (const step of STEPS) {
    if (excluded.has(step)) continue;
    if (!(step in artifact.steps)) {
      missing.push(step);
      continue;
    }
    steps[step] = canonicalizeStep(step, artifact.steps[step], binding);
  }
  if (missing.length > 0) {
    fail(`${binding}: no exclusion declared for missing step(s) ${missing.join(", ")}`);
  }
  return { excluded: [...excluded].sort(), steps };
}

function main() {
  const { artifacts, scenario } = loadArtifacts();
  const canonical = new Map();
  for (const [binding, artifact] of artifacts) {
    canonical.set(binding, canonicalize(binding, artifact));
  }

  // Coverage: every step must be exercised by at least MIN_STEP_COVERAGE
  // bindings — one binding alone is not a cross-language comparison.
  for (const step of STEPS) {
    const covered = [...canonical.entries()].filter(([, c]) => step in c.steps).map(([b]) => b);
    if (covered.length < MIN_STEP_COVERAGE) {
      fail(`step ${step} covered by ${covered.length} binding(s) (${covered.join(", ") || "none"}) — need >= ${MIN_STEP_COVERAGE}`);
    }
  }

  // Compare every step across the bindings that ran it, against the first
  // binding as reference, and hash the canonical projection per step.
  // Per-step hashes are the headline evidence: a binding that excludes a step
  // still produces comparable hashes for the steps it does run.
  const stepHashes = new Map(); // step -> Map(binding -> sha256)
  for (const step of STEPS) {
    const runners = [...canonical.entries()].filter(([, c]) => step in c.steps);
    const [referenceBinding, reference] = runners[0];
    const hashes = new Map();
    for (const [binding, c] of runners) {
      hashes.set(binding, createHash("sha256").update(stableStringify(c.steps[step])).digest("hex"));
    }
    for (const [binding, c] of runners.slice(1)) {
      const diff = firstDifference(reference.steps[step], c.steps[step], `${step}`);
      if (diff) {
        fail(`${referenceBinding} vs ${binding} diverge at ${diff}`);
      }
    }
    stepHashes.set(step, hashes);
  }

  console.log(`PARITY OK — scenario ${scenario} · ${canonical.size}/${BINDINGS.length} bindings`);
  for (const step of STEPS) {
    const hashes = stepHashes.get(step);
    const [digest] = hashes.values();
    const runners = [...hashes.keys()].join(", ");
    const excluded = [...canonical.entries()]
      .filter(([, c]) => c.excluded.includes(step))
      .map(([b]) => b);
    const excludedNote = excluded.length > 0 ? `; excluded: ${excluded.join(", ")}` : "";
    console.log(`  ${step.padEnd(10)} sha256:${digest}  (${runners}${excludedNote})`);
  }
}

main();
