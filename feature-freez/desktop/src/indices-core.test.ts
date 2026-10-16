// FEAT-02 ÍNDICES pure logic tests (node --test, no React).
import { test } from "node:test";
import assert from "node:assert/strict";
import {
  CORE_GAPS,
  fmtBytes,
  fmtCount,
  namespaceBars,
  namespaceBarsFromCounts,
  textIndexTiles,
  vectorIndexTiles,
  walTiles,
} from "./components/indices/indices-core.ts";
import type { OperationalMetrics } from "./vanta.ts";

const METRICS: OperationalMetrics = {
  process_rss_bytes: 1_500_000_000,
  records_imported: 120_000,
  import_errors: 3,
  text_lexical_queries: 42_000,
  text_candidates_scored: 1_200_000,
  planner_hybrid_queries: 10,
  planner_text_only_queries: 20,
  planner_vector_only_queries: 30,
  derived_prefix_scans: 5,
  derived_full_scan_fallbacks: 1,
  startup_ms: 12,
  wal_replay_ms: 7,
  wal_records_replayed: 800,
  ann_rebuild_ms: 450,
  derived_rebuild_ms: 0,
  text_index_rebuild_ms: 90,
  text_postings_written: 55_000,
  text_index_repairs: 2,
  text_consistency_audits: 0,
  text_consistency_audit_failures: 0,
  mmap_resident_bytes: null,
  hnsw_logical_bytes: 20_000_000,
  hnsw_nodes_count: 15_000,
};

test("fmtCount / fmtBytes: compact units", () => {
  assert.equal(fmtCount(999), "999");
  assert.equal(fmtCount(1_500), "1.5K");
  assert.equal(fmtCount(1_200_000), "1.2M");
  assert.equal(fmtCount(2_000_000_000), "2.00B");
  assert.equal(fmtBytes(500), "500 B");
  assert.equal(fmtBytes(2_000), "2 KB");
  assert.equal(fmtBytes(2_500_000), "2.5 MB");
  assert.equal(fmtBytes(1_500_000_000), "1.50 GB");
});

test("namespaceBars: sorted desc, widths relative to max", () => {
  const bars = namespaceBars({ a: { count: 10, expiring_soon: 2, expired: 1 }, b: { count: 40, expiring_soon: 0, expired: 0 } });
  assert.deepEqual(bars.map((b) => b.name), ["b", "a"]);
  assert.equal(bars[0].widthPct, 100);
  assert.equal(bars[1].widthPct, 25);
  assert.equal(bars[1].expiringSoon, 2);
  assert.equal(bars[1].expired, 1);
});

test("namespaceBars: empty map and empty max are safe", () => {
  assert.deepEqual(namespaceBars({}), []);
  const bars = namespaceBarsFromCounts({});
  assert.deepEqual(bars, []);
  assert.equal(namespaceBarsFromCounts({ x: 0, y: 0 })[0].widthPct, 0);
});

test("namespaceBarsFromCounts: WASM fallback has no expiry buckets", () => {
  const bars = namespaceBarsFromCounts({ a: 3, b: 1 });
  assert.deepEqual(bars.map((b) => b.name), ["a", "b"]);
  assert.equal(bars[0].count, 3);
  assert.equal(bars[0].expiringSoon, null);
  assert.equal(bars[0].expired, null);
});

test("vectorIndexTiles: real HNSW fields + dims gap", () => {
  const tiles = vectorIndexTiles(METRICS);
  const byKey = Object.fromEntries(tiles.map((t) => [t.key, t]));
  assert.equal(byKey.hnsw_nodes.value, "15.0K");
  assert.equal(byKey.hnsw_bytes.value, "20.0 MB");
  assert.equal(byKey.dims.value, "—");
  assert.equal(byKey.dims.gap, true);
});

test("textIndexTiles / walTiles map real counters", () => {
  const text = Object.fromEntries(textIndexTiles(METRICS).map((t) => [t.key, t]));
  assert.equal(text.postings.value, "55.0K");
  assert.equal(text.queries.value, "42.0K");
  assert.equal(text.repairs.value, "2");
  const wal = Object.fromEntries(walTiles(METRICS).map((t) => [t.key, t]));
  assert.equal(wal.wal_replay.value, "800");
  assert.equal(wal.wal_replay.muted, "7ms");
});

test("CORE_GAPS documents dims + LSM/WAL status (no invented metrics)", () => {
  const labels = CORE_GAPS.map((g) => g.label);
  assert.ok(labels.includes("dims"));
  assert.ok(labels.includes("LSM/WAL status"));
});