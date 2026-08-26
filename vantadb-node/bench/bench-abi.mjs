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
 * Usage:
 *   node bench/bench-abi.mjs                     # both backends, defaults
 *   node bench/bench-abi.mjs --backend native    # one backend only
 *   node bench/bench-abi.mjs --records 1000 --dim 384 --searches 200
 *
 * Prereqs:
 *   - vantadb-node native binding built  (npm run build, or existing *.node)
 *   - vantadb-ts built                    (cd ../vantadb-ts && npm run build)
 *
 * Output: table + one JSON line (machine-readable) on stdout.
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

    let t = performance.now();
    for (const b of batches) await db.putBatch(b);
    results.put_batch_ops = seed.length;
    results.put_batch_ms = performance.now() - t;

    const q = vec(1);
    t = performance.now();
    for (let i = 0; i < SEARCHES; i++) {
      await db.search({ namespace: NS, query_vector: q, top_k: 10 });
    }
    results.search_vector_ops = SEARCHES;
    results.search_vector_ms = performance.now() - t;

    t = performance.now();
    for (let i = 0; i < SEARCHES; i++) {
      await db.search({ namespace: NS, query_vector: q, text_query: "topic memory", top_k: 10 });
    }
    results.search_hybrid_ops = SEARCHES;
    results.search_hybrid_ms = performance.now() - t;
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

    let t = performance.now();
    for (const b of batches) db.putBatch(b);
    results.put_batch_ops = seed.length;
    results.put_batch_ms = performance.now() - t;

    const q = vec(1);
    t = performance.now();
    for (let i = 0; i < SEARCHES; i++) {
      db.search({ namespace: NS, query_vector: q, top_k: 10 });
    }
    results.search_vector_ops = SEARCHES;
    results.search_vector_ms = performance.now() - t;

    t = performance.now();
    for (let i = 0; i < SEARCHES; i++) {
      db.search({ namespace: NS, query_vector: q, text_query: "topic memory", top_k: 10 });
    }
    results.search_hybrid_ops = SEARCHES;
    results.search_hybrid_ms = performance.now() - t;
  } finally {
    db.close();
  }
  return results;
}

const rate = (r) => Math.round((r.put_batch_ops / r.put_batch_ms) * 1000);
const rateMs = (r) => r.put_batch_ms / r.put_batch_ops;

const fmt = (r) => [
  ["put_batch (records/s)", rate(r)],
  ["put_batch (ms/op)", rateMs(r).toFixed(3)],
  ["search_vector (ops/s)", Math.round((r.search_vector_ops / r.search_vector_ms) * 1000)],
  ["search_hybrid (ops/s)", Math.round((r.search_hybrid_ops / r.search_hybrid_ms) * 1000)],
];

console.log(`vantadb-node vs vantadb-ts A/B bench — records=${RECORDS} dim=${DIM} searches=${SEARCHES}`);
console.log(`note: native=persistent fjall (fsync), wasm=in-memory (no OPFS in Node)`);
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

console.log("JSON: " + JSON.stringify({ records: RECORDS, dim: DIM, searches: SEARCHES, rows }));