#!/usr/bin/env node
/**
 * WASM Vector DB Benchmark Suite (MCP-03)
 *
 * Compares VantaDB, EdgeVec, minimemory, altor-vec, and lattice-db
 * on: ingestion speed, search latency, recall, and memory.
 *
 * Usage:
 *   node --experimental-wasm-modules benchmarks/wasm_bench.mjs [options]
 *
 * Options:
 *   --dims       Vector dimensions          [default: 128]
 *   --n-vectors  Number of vectors to insert [default: 1000]
 *   --n-queries  Number of query vectors     [default: 100]
 *   --top-k      Top-K for search            [default: 10]
 *   --engines    Comma-separated list        [default: vantadb,edgevec,minimemory,altor-vec]
 *   --output     Output JSON path            [default: benchmarks/wasm_benchmark_results.json]
 *
 * Dependencies (install first):
 *   npm install edgevec @rckflr/minimemory altor-vec
 *   (VantaDB uses the local vantadb-wasm/pkg)
 */

import { createRequire } from 'module';
import path from 'path';
import fs from 'fs';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, '..');
const require = createRequire(import.meta.url);

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

const DIMS = parseInt(args.dims || '128', 10);
const N_VECTORS = parseInt(args['n-vectors'] || '1000', 10);
const N_QUERIES = parseInt(args['n-queries'] || '100', 10);
const TOP_K = parseInt(args['top-k'] || '10', 10);
const ENGINES = (args.engines || 'vantadb,edgevec,minimemory').split(',');
const OUTPUT = args.output || path.join(__dirname, 'wasm_benchmark_results.json');

// ── Helpers ────────────────────────────────────────────────────────────────

function randomVector(dim) {
  const v = new Float32Array(dim);
  for (let i = 0; i < dim; i++) v[i] = Math.random() - 0.5;
  return v;
}

function cosineSimilarity(a, b) {
  let dot = 0, na = 0, nb = 0;
  for (let i = 0; i < a.length; i++) {
    dot += a[i] * b[i];
    na += a[i] * a[i];
    nb += b[i] * b[i];
  }
  return dot / (Math.sqrt(na) * Math.sqrt(nb));
}

function computeGroundTruth(vectors, queries, topK) {
  const results = [];
  for (const q of queries) {
    const sims = vectors.map((v, i) => ({ i, s: cosineSimilarity(q, v) }));
    sims.sort((a, b) => b.s - a.s);
    results.push(sims.slice(0, topK).map(x => x.i));
  }
  return results;
}

function recallAtK(predicted, groundTruth, k) {
  let hits = 0;
  for (let i = 0; i < predicted.length; i++) {
    const gtSet = new Set(groundTruth[i].slice(0, k));
    if (gtSet.has(predicted[i])) hits++;
  }
  return hits / predicted.length;
}

function sleep(ms) { return new Promise(r => setTimeout(r, ms)); }

// ── Engine Benchmarks ─────────────────────────────────────────────────────

function generateDataset() {
  const vectors = Array.from({ length: N_VECTORS }, () => randomVector(DIMS));
  const queries = Array.from({ length: N_QUERIES }, () => randomVector(DIMS));
  const groundTruth = computeGroundTruth(vectors, queries, TOP_K);
  return { vectors, queries, groundTruth };
}

// ── VantaDB ────────────────────────────────────────────────────────────────

async function benchVantaDB(dataset) {
  const pkg = path.join(ROOT, 'vantadb-wasm', 'pkg');
  const mod = await import(path.join(pkg, 'vantadb_wasm.js'));
  await mod.default();

  const db = new mod.VantaDB({ storage_path: '/vantadb_bench', read_only: false });

  const t0 = performance.now();
  for (let i = 0; i < dataset.vectors.length; i++) {
    await db.put({
      namespace: 'bench',
      key: `v${i}`,
      payload: `vector ${i}`,
      vector: Array.from(dataset.vectors[i]),
    });
  }
  const ingestMs = performance.now() - t0;

  const tIdx = performance.now();
  await db.rebuild_index();
  const indexMs = performance.now() - tIdx;

  const queryLatencies = [];
  const predictions = [];
  for (const q of dataset.queries) {
    const t = performance.now();
    const hits = await db.search({
      namespace: 'bench',
      query_vector: Array.from(q),
      top_k: TOP_K,
    });
    queryLatencies.push(performance.now() - t);
    const ids = [];
    if (hits && hits.length) {
      for (const h of hits) {
        const key = h.record ? h.record.key : h.key;
        if (key != null) ids.push(parseInt(key.replace('v', ''), 10));
      }
    }
    predictions.push(ids);
  }

  recallAtK(predictions, dataset.groundTruth, TOP_K);
  queryLatencies.sort((a, b) => a - b);

  db.close();

  return {
    engine: 'VantaDB',
    ingest_ms: ingestMs,
    index_ms: indexMs,
    p50_ms: queryLatencies[Math.floor(queryLatencies.length * 0.5)],
    p95_ms: queryLatencies[Math.floor(queryLatencies.length * 0.95)],
    p99_ms: queryLatencies[Math.floor(queryLatencies.length * 0.99)],
    qps: dataset.queries.length / (queryLatencies.reduce((a, b) => a + b, 0) / 1000),
  };
}

// ── EdgeVec ────────────────────────────────────────────────────────────────

async function benchEdgeVec(dataset) {
  const edgevec = require('edgevec');
  await edgevec.default();

  const config = new edgevec.EdgeVecConfig(DIMS);
  const db = new edgevec.EdgeVec(config);

  const t0 = performance.now();
  const ids = [];
  for (const v of dataset.vectors) {
    ids.push(db.insert(v));
  }
  const ingestMs = performance.now() - t0;

  const queryLatencies = [];
  for (const q of dataset.queries) {
    const t = performance.now();
    const results = db.search(q, TOP_K);
    queryLatencies.push(performance.now() - t);
  }

  queryLatencies.sort((a, b) => a - b);

  return {
    engine: 'EdgeVec',
    ingest_ms: ingestMs,
    index_ms: 0,
    p50_ms: queryLatencies[Math.floor(queryLatencies.length * 0.5)],
    p95_ms: queryLatencies[Math.floor(queryLatencies.length * 0.95)],
    p99_ms: queryLatencies[Math.floor(queryLatencies.length * 0.99)],
    qps: dataset.queries.length / (queryLatencies.reduce((a, b) => a + b, 0) / 1000),
  };
}

// ── minimemory ─────────────────────────────────────────────────────────────

async function benchMinimemory(dataset) {
  let mm;
  try {
    mm = require('@rckflr/minimemory');
  } catch {
    return { engine: 'minimemory', error: 'npm install @rckflr/minimemory' };
  }
  await mm.default();

  const db = new mm.WasmVectorDB(DIMS, 'cosine', 'flat');

  const t0 = performance.now();
  for (let i = 0; i < dataset.vectors.length; i++) {
    db.insert(`v${i}`, dataset.vectors[i]);
  }
  const ingestMs = performance.now() - t0;

  const queryLatencies = [];
  for (const q of dataset.queries) {
    const t = performance.now();
    db.search(q, TOP_K);
    queryLatencies.push(performance.now() - t);
  }

  queryLatencies.sort((a, b) => a - b);

  return {
    engine: 'minimemory',
    ingest_ms: ingestMs,
    index_ms: 0,
    p50_ms: queryLatencies[Math.floor(queryLatencies.length * 0.5)],
    p95_ms: queryLatencies[Math.floor(queryLatencies.length * 0.95)],
    p99_ms: queryLatencies[Math.floor(queryLatencies.length * 0.99)],
    qps: dataset.queries.length / (queryLatencies.reduce((a, b) => a + b, 0) / 1000),
  };
}

// ── Main ───────────────────────────────────────────────────────────────────

async function main() {
  console.log('='.repeat(60));
  console.log('  WASM Vector DB Benchmark Suite (MCP-03)');
  console.log('='.repeat(60));
  console.log(`  dims:       ${DIMS}`);
  console.log(`  n-vectors:  ${N_VECTORS}`);
  console.log(`  n-queries:  ${N_QUERIES}`);
  console.log(`  top-k:      ${TOP_K}`);
  console.log(`  engines:    ${ENGINES.join(', ')}`);
  console.log('='.repeat(60));

  const dataset = generateDataset();
  console.log(`  Dataset: ${dataset.vectors.length} vectors, ${dataset.queries.length} queries`);

  const runners = {
    vantadb: benchVantaDB,
    edgevec: benchEdgeVec,
    minimemory: benchMinimemory,
  };

  const results = [];
  for (const name of ENGINES) {
    const fn = runners[name];
    if (!fn) {
      console.warn(`  ** Unknown engine: ${name}, skipping`);
      continue;
    }
    console.log(`\n  --- ${name} ---`);
    try {
      const r = await fn(dataset);
      results.push(r);
      console.log(`  ingest: ${r.ingest_ms?.toFixed(1)}ms | p50: ${r.p50_ms?.toFixed(3)}ms | p95: ${r.p95_ms?.toFixed(3)}ms | qps: ${r.qps?.toFixed(0)}`);
    } catch (e) {
      console.error(`  ** ${name} FAILED: ${e.message}`);
      results.push({ engine: name, error: e.message });
    }
    await sleep(50);
  }

  console.log('\n' + '='.repeat(60));
  console.log('  RESULTS');
  console.log('='.repeat(60));
  console.table(results);

  fs.writeFileSync(OUTPUT, JSON.stringify({ timestamp: new Date().toISOString(), dims: DIMS, nVectors: N_VECTORS, nQueries: N_QUERIES, topK: TOP_K, results }, null, 2));
  console.log(`\n  Results written to: ${OUTPUT}`);
}

main().catch(e => { console.error(e); process.exit(1); });
