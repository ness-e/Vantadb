#!/usr/bin/env node
/**
 * TS-09 — Reproducible JS/WASM bench (insert + search, p50/p95/p99).
 *
 * Scope: Node.js first (in-memory WASM engine; Node has no OPFS — browser
 * persistent backends OPFS/IDB are explicitly out of scope here, see note).
 * Own numbers only — no head-to-head vs other engines (D1).
 *
 * Dataset: deterministic LCG vectors, seed 42 (same shape as
 * `benches/canonical_p99.rs` and `vantadb-node/bench/bench-abi.mjs`).
 * Canonical shape 100k x 1536d x 1k queries is valid with Node >= 22 but
 * takes minutes; defaults are 2000 x 384d x 200q for a fast reproducible run.
 *
 * Usage:
 *   npm run bench                                         # 1 run, defaults
 *   npm run bench -- --records 10000 --dim 1536 --searches 100
 *   npm run bench:3x                                      # 3 runs (median)
 *
 * Output: human-readable table + one machine-readable `JSON:` line on stdout.
 */

import { performance } from "node:perf_hooks";

// Keep the native binding's internal logging off the bench output (the JSON
// line at the end is the machine-readable contract).
process.env.RUST_LOG = process.env.RUST_LOG ?? "warn";

const { VantaDB } = await import("../dist/vantadb.js");

const arg = (name, dflt) => {
  const i = process.argv.indexOf(`--${name}`);
  return i !== -1 && process.argv[i + 1] ? process.argv[i + 1] : dflt;
};
const RECORDS = Number(arg("records", 2000));
const DIM = Number(arg("dim", 384));
const SEARCHES = Number(arg("searches", 200));
const TOP_K = Number(arg("top-k", 10));
const BATCH = Number(arg("batch", 100));

const NS = "bench";
const VEC_SEED = 42;

// Nearest-rank percentile over sorted samples (same as bench-abi.mjs).
const percentile = (sorted, pct) => {
  if (!sorted.length) return 0;
  const idx = Math.min(
    sorted.length - 1,
    Math.max(0, Math.ceil((pct / 100) * sorted.length) - 1),
  );
  return sorted[idx];
};

const summarize = (samples) => {
  if (!samples.length) return { p50: 0, p95: 0, p99: 0, min: 0, max: 0, mean: 0 };
  const sorted = samples.slice().sort((a, b) => a - b);
  const sum = samples.reduce((a, b) => a + b, 0);
  return {
    p50: percentile(sorted, 50),
    p95: percentile(sorted, 95),
    p99: percentile(sorted, 99),
    min: sorted[0],
    max: sorted[sorted.length - 1],
    mean: sum / samples.length,
  };
};

// Deterministic pseudo-random vector in [0,1) — identical data every run.
const vec = (seed) => {
  let s = (seed * 2654435761) % 4294967296;
  const out = new Array(DIM);
  for (let i = 0; i < DIM; i++) {
    s = (s * 1664525 + 1013904223) % 4294967296;
    out[i] = (s / 4294967296) % 1;
  }
  return out;
};

const db = VantaDB.create();
try {
  const batches = [];
  for (let i = 0; i < RECORDS; i += BATCH) {
    const chunk = [];
    for (let j = i; j < Math.min(i + BATCH, RECORDS); j++) {
      chunk.push({
        namespace: NS,
        key: `doc-${j}`,
        payload: `document ${j} about topic ${j % 50} vector memory`,
        vector: vec(j),
      });
    }
    batches.push(chunk);
  }

  db.putBatch(batches[0]); // warmup (excluded from samples)

  const insertSamples = [];
  for (const b of batches) {
    const t = performance.now();
    db.putBatch(b);
    insertSamples.push(performance.now() - t);
  }
  const insert = { ...summarize(insertSamples), ops: RECORDS };

  const q = vec(VEC_SEED);
  const searchVecSamples = [];
  for (let i = 0; i < SEARCHES; i++) {
    const t = performance.now();
    db.search({ namespace: NS, query_vector: q, top_k: TOP_K });
    searchVecSamples.push(performance.now() - t);
  }
  const searchVector = { ...summarize(searchVecSamples), ops: SEARCHES };

  const searchHybSamples = [];
  for (let i = 0; i < SEARCHES; i++) {
    const t = performance.now();
    db.search({ namespace: NS, query_vector: q, text_query: "topic memory", top_k: TOP_K });
    searchHybSamples.push(performance.now() - t);
  }
  const searchHybrid = { ...summarize(searchHybSamples), ops: SEARCHES };

  const fmtMs = (v) => v.toFixed(3);
  console.log(`vantadb-ts WASM bench — records=${RECORDS} dim=${DIM} searches=${SEARCHES} top_k=${TOP_K} batch=${BATCH} vec_seed=${VEC_SEED}`);
  console.log(`note: in-memory WASM engine in Node (no OPFS); browser backends pending.`);
  console.log("");
  for (const [label, s] of [["insert", insert], ["search_vector", searchVector], ["search_hybrid", searchHybrid]]) {
    console.log(`  ${label.padEnd(14)} p50/p95/p99 (ms) ${fmtMs(s.p50)} / ${fmtMs(s.p95)} / ${fmtMs(s.p99)}  mean ${fmtMs(s.mean)}`);
  }
  console.log("");
  console.log("JSON: " + JSON.stringify({ records: RECORDS, dim: DIM, searches: SEARCHES, top_k: TOP_K, batch: BATCH, vec_seed: VEC_SEED, insert, search_vector: searchVector, search_hybrid: searchHybrid }));
} finally {
  db.close();
}
