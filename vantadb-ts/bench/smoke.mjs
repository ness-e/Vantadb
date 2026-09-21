#!/usr/bin/env node
/**
 * TS-09 smoke: runs the bench harness tiny (50 x 8d x 5q) and asserts the
 * machine-readable `JSON:` line parses with p50/p95/p99 per op.
 * Usage: node bench/smoke.mjs   (from vantadb-ts/)
 */
import { execFileSync } from "node:child_process";

const out = execFileSync(
  process.execPath,
  ["bench/bench.mjs", "--records", "50", "--dim", "8", "--searches", "5", "--batch", "25"],
  { encoding: "utf-8", env: { ...process.env, RUST_LOG: "warn" } },
);
const line = out.split("\n").find((l) => l.startsWith("JSON: "));
if (!line) throw new Error("smoke: no JSON: line in bench output");
const res = JSON.parse(line.slice("JSON: ".length));
for (const op of ["insert", "search_vector", "search_hybrid"]) {
  for (const p of ["p50", "p95", "p99", "mean"]) {
    if (typeof res[op]?.[p] !== "number" || Number.isNaN(res[op][p])) {
      throw new Error(`smoke: ${op}.${p} missing or not a number`);
    }
  }
}
if (res.vec_seed !== 42) throw new Error("smoke: vec_seed must be 42 (deterministic dataset)");
console.log("smoke OK: bench emits p50/p95/p99 per op (insert, search_vector, search_hybrid)");
