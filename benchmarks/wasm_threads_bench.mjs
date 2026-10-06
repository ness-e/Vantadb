#!/usr/bin/env node
/**
 * STRAT-04 — WASM threads lock-free kernel benchmark (Node harness).
 *
 * Measures the SAME lock-free scoring kernel (batch cosine similarity with an
 * atomic chunk cursor) running in N worker_threads that all import ONE shared
 * `WebAssembly.Memory`. `--workers 1` IS the single-thread baseline (the
 * declared fallback path — same kernel, no parallelism).
 *
 * Prerequisites:
 *   pwsh vantadb-wasm/threads-kernel/build.ps1     # builds the .wasm
 *
 * Usage:
 *   node benchmarks/wasm_threads_bench.mjs [options]
 *
 * Options:
 *   --n-vectors  Vectors in the arena            [default: 100000]
 *   --dims       Vector dimensions               [default: 128]
 *   --workers    Comma-separated worker counts   [default: 1,2,4,8,12]
 *   --repeats    Runs per worker count           [default: 5]
 *   --chunk      Vectors claimed per fetch_add   [default: 256]
 *   --seed       Dataset seed (deterministic)    [default: 42]
 *   --wasm       Path to the .wasm module        [default: threads-kernel release]
 *   --output     Output JSON path                [default: benchmarks/wasm_threads_results.json]
 *
 * Exit code != 0 on any correctness failure (missing scores, reference drift,
 * kernel status != 0) — a green run is the correctness gate, the numbers are
 * the benchmark.
 */

import { Worker } from 'node:worker_threads';
import { readFile } from 'node:fs/promises';
import path from 'node:path';
import fs from 'node:fs';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, '..');
const DEFAULT_WASM = path.join(
  ROOT,
  'vantadb-wasm',
  'threads-kernel',
  'target',
  'wasm32-unknown-unknown',
  'release',
  'vantadb_wasm_threads_kernel.wasm',
);
const WORKER_PATH = path.join(__dirname, 'wasm_threads_worker.mjs');

// ── CLI ────────────────────────────────────────────────────────────────────

const args = {};
for (let i = 2; i < process.argv.length; i++) {
  const k = process.argv[i].replace(/^--/, '');
  if (i + 1 < process.argv.length && !process.argv[i + 1].startsWith('--')) {
    args[k] = process.argv[++i];
  } else {
    args[k] = true;
  }
}

const N_VECTORS = parseInt(args['n-vectors'] || '100000', 10);
const DIMS = parseInt(args.dims || '128', 10);
const WORKER_COUNTS = (args.workers || '1,2,4,8,12')
  .split(',')
  .map((n) => parseInt(n, 10));
const REPEATS = parseInt(args.repeats || '5', 10);
const CHUNK = parseInt(args.chunk || '256', 10);
const SEED = parseInt(args.seed || '42', 10);
const WASM_PATH = args.wasm || DEFAULT_WASM;
const OUTPUT = args.output || path.join(__dirname, 'wasm_threads_results.json');

// ── Memory layout (host-owned) ─────────────────────────────────────────────
//
// [0, MODULE_STATIC)                 module static region (--initial-memory=2MiB:
//                                    stack-first 1MiB + data) — never used by workers
// [MODULE_STATIC, +N*STACK_SIZE)     one stack region per worker instance
// [arenaBase, ...)                   vectors | query | scores | cursor
//
// The module is linked with `--export=__stack_pointer`, so each worker points
// its instance at its own stack top (wasm stacks grow downwards).

const PAGE = 65536;
const MODULE_STATIC = 2 * 1024 * 1024; // keep in sync with .cargo/config.toml
const STACK_SIZE = 1024 * 1024; // per-worker stack region (kernel frames are tiny)
const MAX_WORKERS = Math.max(...WORKER_COUNTS);

const vectorsBytes = N_VECTORS * DIMS * 4;
const queryBytes = DIMS * 4;
const scoresBytes = N_VECTORS * 4;

const stackBase = MODULE_STATIC;
const arenaBase = stackBase + MAX_WORKERS * STACK_SIZE;
const vectorsPtr = arenaBase;
const queryPtr = vectorsPtr + vectorsBytes;
const scoresPtr = queryPtr + queryBytes;
const cursorPtr = (scoresPtr + scoresBytes + 7) & ~7; // 8-byte aligned
const totalBytes = cursorPtr + 8;
const initialPages = Math.ceil(totalBytes / PAGE);
const MAX_PAGES = 16384; // --max-memory=1GiB in .cargo/config.toml

// ── Dataset (deterministic, seeded) ────────────────────────────────────────

function mulberry32(seed) {
  let a = seed >>> 0;
  return () => {
    a |= 0;
    a = (a + 0x6d2b79f5) | 0;
    let t = Math.imul(a ^ (a >>> 15), 1 | a);
    t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

function referenceCosine(vectors, query, index, dims) {
  let dot = 0;
  let na = 0;
  let nb = 0;
  const base = index * dims;
  for (let d = 0; d < dims; d++) {
    const v = vectors[base + d];
    const q = query[d];
    dot += v * q;
    na += v * v;
    nb += q * q;
  }
  const denom = Math.sqrt(na) * Math.sqrt(nb);
  return denom > 0 ? dot / denom : 0;
}

// ── Harness ────────────────────────────────────────────────────────────────

function once(worker, type) {
  return new Promise((resolve, reject) => {
    const onMessage = (msg) => {
      if (msg?.type !== type) return;
      worker.off('message', onMessage);
      worker.off('error', onError);
      resolve(msg);
    };
    const onError = (err) => {
      worker.off('message', onMessage);
      reject(err);
    };
    worker.on('message', onMessage);
    worker.on('error', onError);
  });
}

async function runOnce(workers) {
  const buffer = workers[0]._memory.buffer;
  // Reset shared state for this run.
  new Uint32Array(buffer, cursorPtr, 1)[0] = 0;
  new Float32Array(buffer, scoresPtr, N_VECTORS).fill(NaN);

  const t0 = performance.now();
  for (const w of workers) w.postMessage({ type: 'go' });
  const results = await Promise.all(workers.map((w) => once(w, 'done')));
  const elapsedMs = performance.now() - t0;

  for (const r of results) {
    if (r.status !== 0) throw new Error(`kernel returned status ${r.status}`);
  }

  // Correctness gate: every index scored exactly once (NaN would mean a
  // skipped chunk) and scores match an f64 reference within f32 tolerance.
  const scores = new Float32Array(buffer, scoresPtr, N_VECTORS);
  let nanCount = 0;
  for (let i = 0; i < N_VECTORS; i++) if (Number.isNaN(scores[i])) nanCount++;
  if (nanCount > 0) {
    throw new Error(`${nanCount} of ${N_VECTORS} indices were never scored`);
  }

  const vectors = new Float32Array(buffer, vectorsPtr, N_VECTORS * DIMS);
  const query = new Float32Array(buffer, queryPtr, DIMS);
  let maxAbsErr = 0;
  const SPOT_CHECKS = 16;
  for (let k = 0; k < SPOT_CHECKS; k++) {
    const i = Math.floor(((k + 0.5) * N_VECTORS) / SPOT_CHECKS);
    const ref = referenceCosine(vectors, query, i, DIMS);
    maxAbsErr = Math.max(maxAbsErr, Math.abs(scores[i] - ref));
  }
  if (maxAbsErr > 5e-4) {
    throw new Error(`scores drifted from reference: max abs err ${maxAbsErr}`);
  }

  return { elapsedMs, maxAbsErr, workerMs: results.map((r) => r.kernelMs) };
}

async function spawnWorkers(memory, workerCount) {
  const workers = [];
  for (let i = 0; i < workerCount; i++) {
    const w = new Worker(WORKER_PATH, {
      workerData: {
        wasmPath: WASM_PATH,
        memory,
        stackTop: stackBase + (i + 1) * STACK_SIZE,
        layout: {
          vectorsPtr,
          count: N_VECTORS,
          dims: DIMS,
          queryPtr,
          scoresPtr,
          cursorPtr,
        },
        chunk: CHUNK,
      },
    });
    w._memory = memory;
    workers.push(w);
  }
  await Promise.all(workers.map((w) => once(w, 'ready')));
  return workers;
}

function median(values) {
  const sorted = [...values].sort((a, b) => a - b);
  const mid = Math.floor(sorted.length / 2);
  return sorted.length % 2 ? sorted[mid] : (sorted[mid - 1] + sorted[mid]) / 2;
}

async function main() {
  if (!fs.existsSync(WASM_PATH)) {
    console.error(`wasm module not found: ${WASM_PATH}`);
    console.error('Build it first: pwsh vantadb-wasm/threads-kernel/build.ps1');
    process.exit(2);
  }

  console.log('='.repeat(72));
  console.log('  STRAT-04 — WASM threads lock-free kernel benchmark');
  console.log('='.repeat(72));
  console.log(`  module:      ${WASM_PATH}`);
  console.log(`  dims:        ${DIMS}`);
  console.log(`  n-vectors:   ${N_VECTORS}`);
  console.log(`  chunk:       ${CHUNK}`);
  console.log(`  workers:     ${WORKER_COUNTS.join(', ')}`);
  console.log(`  repeats:     ${REPEATS}`);
  console.log(`  arena bytes: ${vectorsBytes + queryBytes + scoresBytes} (~${((vectorsBytes + queryBytes + scoresBytes) / 1048576).toFixed(1)} MiB)`);
  console.log('='.repeat(72));

  // One shared memory for all runs/workers.
  const memory = new WebAssembly.Memory({
    initial: initialPages,
    maximum: MAX_PAGES,
    shared: true,
  });

  // Deterministic dataset.
  const buffer = memory.buffer;
  const rand = mulberry32(SEED);
  const vectors = new Float32Array(buffer, vectorsPtr, N_VECTORS * DIMS);
  for (let i = 0; i < vectors.length; i++) vectors[i] = rand() * 2 - 1;
  const query = new Float32Array(buffer, queryPtr, DIMS);
  for (let d = 0; d < DIMS; d++) query[d] = rand() * 2 - 1;

  const rows = [];
  let baselineMs = null;
  let baselineComputeMs = null;
  const WARMUP_RUNS = 3;

  for (const workerCount of WORKER_COUNTS) {
    const workers = await spawnWorkers(memory, workerCount);
    try {
      // Warm-up: the FIRST call per worker runs in V8's Liftoff baseline
      // (~7x slower); timed runs must measure steady-state TurboFan code.
      for (let w = 0; w < WARMUP_RUNS; w++) await runOnce(workers);

      const times = [];
      const computeTimes = [];
      let maxAbsErr = 0;
      let lastWorkerMs = [];
      for (let r = 0; r < REPEATS; r++) {
        const res = await runOnce(workers);
        times.push(res.elapsedMs);
        computeTimes.push(Math.max(...res.workerMs)); // slowest worker = parallel compute phase
        maxAbsErr = Math.max(maxAbsErr, res.maxAbsErr);
        lastWorkerMs = res.workerMs;
      }
      const med = median(times);
      const medCompute = median(computeTimes);
      if (workerCount === 1) baselineMs = med;
      const speedup = baselineMs ? baselineMs / med : 1;
      const computeSpeedup = baselineComputeMs ? baselineComputeMs / medCompute : 1;
      const vectorsPerSec = (N_VECTORS / med) * 1000;
      rows.push({
        workers: workerCount,
        median_ms: +med.toFixed(2),
        compute_ms: +medCompute.toFixed(2),
        min_ms: +Math.min(...times).toFixed(2),
        max_ms: +Math.max(...times).toFixed(2),
        speedup_vs_1: +speedup.toFixed(2),
        compute_speedup_vs_1: +computeSpeedup.toFixed(2),
        vectors_per_s: Math.round(vectorsPerSec),
        max_abs_err: maxAbsErr,
        worker_kernel_ms: lastWorkerMs.map((m) => +m.toFixed(1)),
        correct: true,
      });
      if (workerCount === 1) baselineComputeMs = medCompute;
      console.log(
        `  workers=${String(workerCount).padStart(2)}  wall ${med.toFixed(2).padStart(8)} ms (${speedup.toFixed(2)}x)  ` +
          `compute ${medCompute.toFixed(2).padStart(7)} ms (${computeSpeedup.toFixed(2)}x)  ` +
          `${Math.round(vectorsPerSec).toLocaleString('en-US')} vec/s  maxerr ${maxAbsErr.toExponential(2)}`,
      );
    } finally {
      await Promise.all(workers.map((w) => w.terminate()));
    }
  }

  const result = {
    timestamp: new Date().toISOString(),
    node: process.version,
    platform: `${process.platform}-${process.arch}`,
    cpu_cores: (await import('node:os')).cpus().length,
    dims: DIMS,
    n_vectors: N_VECTORS,
    chunk: CHUNK,
    seed: SEED,
    repeats: REPEATS,
    rows,
  };
  fs.writeFileSync(OUTPUT, JSON.stringify(result, null, 2));
  console.log('='.repeat(72));
  console.log(`  results written to: ${OUTPUT}`);
}

main().catch((e) => {
  console.error(`FAILED: ${e.message}`);
  process.exit(1);
});
