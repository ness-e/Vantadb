#!/usr/bin/env node
/**
 * PERF-BENCH-01 — A/B benchmark harness: vantadb-node (native napi) vs
 * vantadb-ts (WASM) on the same operation mix.
 *
 * Scope: REPRODUCIBLE HARNESS ONLY. It prints numbers, it does not judge
 * them — publishing any performance claim is a lead/owner decision
 * (research H-09: "medir primero", Regla 9/11).
 *
 * Fairness caveat (by design, not a bug):
 *   - native backend: `VantaDb.connect(dir)` — persistent fjall storage on disk.
 *   - WASM backend:   `VantaDB.create()`  — in-memory engine (Node has no OPFS;
 *                     `connect(path)` is browser-only in the WASM backend).
 *   So the native side pays fsync/persistence costs the WASM side never sees.
 *
 * Dataset (Regla 9 — medición antes de decidir):
 *   - 100k inserts × 1536d (canónico, mismo shape que `benches/canonical_p99.rs`)
 *   - 1000 search queries (vector + hybrid), top_k=10, seed 42
 *   - Override via --records / --dim / --searches for smoke runs
 *
 * Output: tabla human-readable + un JSON line (machine-readable) en stdout.
 * El JSON incluye p50/p95/p99 por op (insert/search_vector/search_hybrid).
 *
 * Usage:
 *   node bench/bench-abi.mjs                     # both backends, defaults
 *   node bench/bench-abi.mjs --backend native    # one backend only
 *   node bench/bench-abi.mjs --records 1000 --dim 384 --searches 200
 *
 * Prereqs:
 *   - vantadb-node native binding built  (npm run build, or existing *.node)
 *   - vantadb-ts built                    (cd ../vantadb-ts && npm run build)
 *
 * Caveat (sandbox 2026-08-30): el harness corre bajo bun en este ambiente, pero
 * bun no soporta ESM `.wasm` desde paquetes npm sin plugin → la rama `wasm`
 * falla en init (`wasm.vantadb_new undefined`). El comando canónico para la
 * rama WASM es Node ≥22 (donde `vantadb-ts` corre sus tests OK vía vitest con
 * `server.deps.external: [/vantadb-wasm/]` y ESM wasm nativo de Node).
 */

import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { performance } from "node:perf_hooks";

// Keep the native binding's internal logging off the bench output (the JSON
// line at the end is the machine-readable contract; planner DEBUG chatter
// would pollute it on stdout).
process.env.RUST_LOG = process.env.RUST_LOG ?? "warn";

import { VantaDb as NativeVantaDb } from "../index.js";
import { VantaDB as WasmVantaDB } from "../../vantadb-ts/dist/vantadb.js";

const arg = (name, dflt) => {
  const i = process.argv.indexOf(`--${name}`);
  return i !== -1 && process.argv[i + 1] ? process.argv[i + 1] : dflt;
};
const RECORDS = Number(arg("records", 500));
const DIM = Number(arg("dim", 64));
const SEARCHES = Number(arg("searches", 100));
const BACKENDS = arg("backend", "native,wasm").split(",");

const NS = "bench";

// Deterministic seed (Regla 9 — dataset determinístico seed 42).
const VEC_SEED = 42;

// Percentile — copiado de `.agents/skills/impeccable/scripts/detector/profile/profiler.mjs:95-102`
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

// Deterministic pseudo-random vector in [0,1) so both backends see identical data.
const vec = (seed) => {
  let s = seed * 2654435761 % 4294967296;
  const out = new Array(DIM);
  for (let i = 0; i < DIM; i++) {
    s = (s * 1664525 + 1013904223) % 4294967296;
    out[i] = (s / 4294967296) % 1;
  }
  return out;
};

const seedRecords = () =>
  Array.from({ length: RECORDS }, (_, i) => ({
    namespace: NS,
    key: `doc-${i}`,
    payload: `document ${i} about topic ${i % 50} vector memory`,
    vector: vec(i),
  }));

async function benchNative() {
  const dir = mkdtempSync(join(tmpdir(), "vantadb-bench-native-"));
  const db = await NativeVantaDb.connect(dir);
  const results = {};
  try {
    const seed = seedRecords();
    const batches = [];
    for (let i = 0; i < seed.length; i += 100) batches.push(seed.slice(i, i + 100));

    // warmup
    await db.putBatch(batches[0]);

    // Per-op insert timings: time each putBatch individually so we can derive p50/p95/p99.
    const insertSamples = [];
    for (const b of batches) {
      const t = performance.now();
      await db.putBatch(b);
      insertSamples.push(performance.now() - t);
    }
    results.insert = summarize(insertSamples);
    results.insert_ops = seed.length;

    const q = vec(VEC_SEED);
    const searchVecSamples = [];
    for (let i = 0; i < SEARCHES; i++) {
      const t = performance.now();
      await db.search({ namespace: NS, query_vector: q, top_k: 10 });
      searchVecSamples.push(performance.now() - t);
    }
    results.search_vector = summarize(searchVecSamples);
    results.search_vector_ops = SEARCHES;

    const searchHybSamples = [];
    for (let i = 0; i < SEARCHES; i++) {
      const t = performance.now();
      await db.search({ namespace: NS, query_vector: q, text_query: "topic memory", top_k: 10 });
      searchHybSamples.push(performance.now() - t);
    }
    results.search_hybrid = summarize(searchHybSamples);
    results.search_hybrid_ops = SEARCHES;
  } finally {
    await db.close();
    rmSync(dir, { recursive: true, force: true });
  }
  return results;
}

function benchWasm() {
  const db = WasmVantaDB.create();
  const results = {};
  try {
    const seed = seedRecords();
    const batches = [];
    for (let i = 0; i < seed.length; i += 100) batches.push(seed.slice(i, i + 100));

    db.putBatch(batches[0]); // warmup

    // Per-op insert timings (WASM is synchronous — no await).
    const insertSamples = [];
    for (const b of batches) {
      const t = performance.now();
      db.putBatch(b);
      insertSamples.push(performance.now() - t);
    }
    results.insert = summarize(insertSamples);
    results.insert_ops = seed.length;

    const q = vec(VEC_SEED);
    const searchVecSamples = [];
    for (let i = 0; i < SEARCHES; i++) {
      const t = performance.now();
      db.search({ namespace: NS, query_vector: q, top_k: 10 });
      searchVecSamples.push(performance.now() - t);
    }
    results.search_vector = summarize(searchVecSamples);
    results.search_vector_ops = SEARCHES;

    const searchHybSamples = [];
    for (let i = 0; i < SEARCHES; i++) {
      const t = performance.now();
      db.search({ namespace: NS, query_vector: q, text_query: "topic memory", top_k: 10 });
      searchHybSamples.push(performance.now() - t);
    }
    results.search_hybrid = summarize(searchHybSamples);
    results.search_hybrid_ops = SEARCHES;
  } finally {
    db.close();
  }
  return results;
}

const fmtMs = (v) => (v ? v.toFixed(3) : "n/a");
const fmtRate = (ops, totalMs) => (totalMs ? Math.round((ops / totalMs) * 1000) : 0);

const fmt = (r) => {
  const insertTotal = (r.insert?.mean || 0) * (r.insert_ops || 0);
  const searchVecTotal = (r.search_vector?.mean || 0) * (r.search_vector_ops || 0);
  const searchHybTotal = (r.search_hybrid?.mean || 0) * (r.search_hybrid_ops || 0);
  return [
    ["insert p50/p95/p99 (ms)", `${fmtMs(r.insert?.p50)} / ${fmtMs(r.insert?.p95)} / ${fmtMs(r.insert?.p99)}`],
    ["insert mean (ms/op)", fmtMs(r.insert?.mean)],
    ["insert throughput (rec/s)", fmtRate(r.insert_ops || 0, insertTotal)],
    ["search_vector p50/p95/p99 (ms)", `${fmtMs(r.search_vector?.p50)} / ${fmtMs(r.search_vector?.p95)} / ${fmtMs(r.search_vector?.p99)}`],
    ["search_vector mean (ms/op)", fmtMs(r.search_vector?.mean)],
    ["search_vector throughput (ops/s)", fmtRate(r.search_vector_ops || 0, searchVecTotal)],
    ["search_hybrid p50/p95/p99 (ms)", `${fmtMs(r.search_hybrid?.p50)} / ${fmtMs(r.search_hybrid?.p95)} / ${fmtMs(r.search_hybrid?.p99)}`],
    ["search_hybrid mean (ms/op)", fmtMs(r.search_hybrid?.mean)],
    ["search_hybrid throughput (ops/s)", fmtRate(r.search_hybrid_ops || 0, searchHybTotal)],
  ];
};

console.log(`vantadb-node vs vantadb-ts A/B bench — records=${RECORDS} dim=${DIM} searches=${SEARCHES} vec_seed=${VEC_SEED}`);
console.log(`note: native=persistent fjall (fsync), wasm=in-memory (no OPFS in Node). WASM init falls back to in-memory on Node.`);
console.log("");

const rows = [];
for (const backend of BACKENDS) {
  const res = backend === "native" ? await benchNative() : backend === "wasm" ? benchWasm() : null;
  if (!res) throw new Error(`unknown backend '${backend}' (use native|wasm|native,wasm)`);
  rows.push({ backend, ...res });
  console.log(`── ${backend} ──`);
  for (const [label, value] of fmt(res)) console.log(`  ${label.padEnd(24)} ${value}`);
  console.log("");
}

console.log("JSON: " + JSON.stringify({
  records: RECORDS,
  dim: DIM,
  searches: SEARCHES,
  vec_seed: VEC_SEED,
  rows,
}));