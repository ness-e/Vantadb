/**
 * Native (N-API) search-latency probe — FIND-668.
 *
 * `vantadb-node/bench/bench-abi.mjs` measures both backends, but it imports
 * `../../vantadb-ts/dist/vantadb.js` at module load, so it cannot run at all
 * unless the WASM/TS workspace is built (`SyntaxError: The requested module ...
 * does not provide an export named 'VantaDB'`). That is why the pathological
 * `search_hybrid` number in `docs/user/operations/BENCHMARKS.md` §10/§15/§16 had
 * never been reproduced against a current build.
 *
 * This probe loads ONLY the prebuilt native addon (`vantadb-node/index.cjs` ->
 * `vantadb_native.<platform>.node`): no Rust rebuild, no WASM dist.
 *
 * It times the three operations over the SAME corpus and the SAME top_k, which
 * is the comparison the finding claims (hybrid vs vector on 2K-10K records):
 *
 *   insert          putBatch of `records` documents
 *   search_vector   vector only
 *   search_hybrid   vector + text_query (BM25 + HNSW + RRF fusion)
 *
 * Usage:
 *   node evals/node_abi_search_probe.mjs --records 2000 --searches 30 --json out.json
 *   node evals/node_abi_search_probe.mjs --sweep 500,2000,10000
 */

import { createRequire } from "node:module";
import { mkdtempSync, rmSync, writeFileSync, mkdirSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const require = createRequire(import.meta.url);
const HERE = dirname(fileURLToPath(import.meta.url));
const REPO = join(HERE, "..");
const { VantaDb: NativeVantaDb } = require(join(REPO, "vantadb-node", "index.cjs"));

const arg = (name, dflt) => {
  const i = process.argv.indexOf(`--${name}`);
  return i === -1 || i + 1 >= process.argv.length ? dflt : process.argv[i + 1];
};

const DIM = Number(arg("dim", 64));
const TOP_K = Number(arg("top_k", 10));
const SEARCHES = Number(arg("searches", 30));
const BATCH = Number(arg("batch", 100));
const NS = "abi_probe";

const vec = (seed) => {
  // Deterministic LCG: same vector for the same seed on every run/machine.
  let s = (seed * 2654435761) >>> 0;
  const out = new Array(DIM);
  let norm = 0;
  for (let i = 0; i < DIM; i++) {
    s = (s * 1664525 + 1013904223) >>> 0;
    const v = s / 0xffffffff - 0.5;
    out[i] = v;
    norm += v * v;
  }
  norm = Math.sqrt(norm) || 1;
  return out.map((v) => v / norm);
};

const percentile = (sorted, pct) => {
  const idx = Math.min(sorted.length - 1, Math.max(0, Math.ceil((pct / 100) * sorted.length) - 1));
  return Number(sorted[idx].toFixed(3));
};

const summarize = (samples) => {
  const s = samples.slice().sort((a, b) => a - b);
  const sum = s.reduce((a, b) => a + b, 0);
  return {
    n: s.length,
    p50: percentile(s, 50),
    p95: percentile(s, 95),
    p99: percentile(s, 99),
    mean: Number((sum / s.length).toFixed(3)),
  };
};

async function measure(records) {
  const dir = mkdtempSync(join(tmpdir(), "vantadb-abi-probe-"));
  const db = await NativeVantaDb.connect(dir);
  const out = { records, dim: DIM, top_k: TOP_K, searches: SEARCHES };
  try {
    const docs = Array.from({ length: records }, (_, i) => ({
      namespace: NS,
      key: `doc-${i}`,
      payload: `document ${i} about topic ${i % 50} vector memory`,
      vector: vec(i + 1),
    }));

    const t0 = performance.now();
    for (let i = 0; i < docs.length; i += BATCH) {
      await db.putBatch(docs.slice(i, i + BATCH));
    }
    out.insert_ms = Number((performance.now() - t0).toFixed(2));

    const q = vec(42);
    const vecSamples = [];
    for (let i = 0; i < SEARCHES; i++) {
      const t = performance.now();
      await db.search({ namespace: NS, query_vector: q, top_k: TOP_K });
      vecSamples.push(performance.now() - t);
    }
    out.search_vector = summarize(vecSamples);

    const hybSamples = [];
    for (let i = 0; i < SEARCHES; i++) {
      const t = performance.now();
      await db.search({
        namespace: NS,
        query_vector: q,
        text_query: "topic memory",
        top_k: TOP_K,
      });
      hybSamples.push(performance.now() - t);
    }
    out.search_hybrid = summarize(hybSamples);

    out.hybrid_over_vector_p50 =
      out.search_vector.p50 > 0
        ? Number((out.search_hybrid.p50 / out.search_vector.p50).toFixed(2))
        : null;
  } finally {
    await db.close();
    rmSync(dir, { recursive: true, force: true });
  }
  return out;
}

const main = async () => {
  const sweepArg = arg("sweep", "");
  const sizes = sweepArg
    ? sweepArg.split(",").map((s) => Number(s.trim()))
    : [Number(arg("records", 2000))];

  console.log("VantaDB native (N-API) search probe — FIND-668");
  console.log(`  dim=${DIM} top_k=${TOP_K} searches=${SEARCHES}`);
  console.log("");
  console.log("records | insert (ms) | vec p50 | vec p95 | hyb p50 | hyb p95 | hyb/vec");
  console.log("--------+-------------+---------+---------+---------+---------+--------");

  const results = [];
  for (const n of sizes) {
    const r = await measure(n);
    results.push(r);
    console.log(
      `${String(n).padStart(7)} | ${String(r.insert_ms).padStart(11)} | ` +
        `${String(r.search_vector.p50).padStart(7)} | ${String(r.search_vector.p95).padStart(
          7,
        )} | ` +
        `${String(r.search_hybrid.p50).padStart(7)} | ${String(r.search_hybrid.p95).padStart(
          7,
        )} | ` +
        `${String(r.hybrid_over_vector_p50).padStart(6)}x`,
    );
  }

  const jsonPath = arg("json", "");
  if (jsonPath) {
    const report = {
      benchmark: "node_abi_search_probe",
      finding: "FIND-668",
      timestamp: new Date().toISOString(),
      environment: {
        node: process.version,
        platform: process.platform,
        arch: process.arch,
        dim: DIM,
        top_k: TOP_K,
        searches: SEARCHES,
        addon: "vantadb-node/index.cjs (prebuilt napi addon)",
      },
      results,
    };
    mkdirSync(dirname(jsonPath), { recursive: true });
    writeFileSync(jsonPath, JSON.stringify(report, null, 2));
    console.log(`\n  report: ${jsonPath}`);
  }
};

main().catch((e) => {
  console.error("FAILED:", e);
  process.exit(1);
});
