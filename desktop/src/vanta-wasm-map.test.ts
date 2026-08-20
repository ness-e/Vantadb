// WASM-02 mapper tests — pure logic, no DOM/WASM (node --test, like
// vanta-http-map.test.ts). Exercises the DTO adaptation (WASM wire ↔ desktop
// DTO) with a fake `VantaDB` handle and the coverage of every `vanta_*`
// command in vanta.ts (deep_link_take is Tauri-only, guarded in vanta.ts).
import { test } from "node:test";
import assert from "node:assert/strict";
import {
  getWasmMapping,
  wasmMappedCommands,
  wasmUnsupportedCommands,
} from "./vanta-wasm-map.ts";
import type { VantaDB } from "../../vantadb-wasm/pkg/vantadb_wasm.js";
import type { HealthReport, MemoryRecord, OperationalMetrics, SearchResult } from "./vanta.ts";

const sdkRecord = {
  namespace: "ns1",
  key: "k1",
  payload: "hello",
  metadata: { kind: { String: "doc" }, n: { Int: 3 } },
  created_at_ms: 100,
  updated_at_ms: 200,
  version: 1,
  node_id: "42",
  vector: null,
  sparse_vector: null,
  expires_at_ms: null,
};

const db = (m: Record<string, unknown>) => m as unknown as VantaDB;

test("vanta_get: WASM record → desktop MemoryRecord (key→id, payload→text, tagged metadata untagged)", () => {
  const out = getWasmMapping("vanta_get").run(
    db({ get: () => sdkRecord }),
    { namespace: "ns1", key: "k1" },
  ) as MemoryRecord;
  assert.equal(out.id, "k1");
  assert.equal(out.namespace, "ns1");
  assert.equal(out.text, "hello");
  assert.deepEqual(out.metadata, { kind: "doc", n: 3 });
  assert.equal(out.node_id, "42");
});

test("vanta_get: missing record → descriptive rejection", () => {
  assert.throws(
    () => getWasmMapping("vanta_get").run(db({ get: () => null }), { namespace: "ns1", key: "missing" }),
    /record not found: ns1\/missing/,
  );
});

test("vanta_put: metadata tagged for the wire, ttl computed, record adapted", () => {
  let captured: unknown;
  const expires = Date.now() + 5000;
  const out = getWasmMapping("vanta_put").run(
    db({
      put: (input: unknown) => {
        captured = input;
        return sdkRecord;
      },
    }),
    { namespace: "ns1", key: "k1", payload: "hello", metadata: { kind: "doc", n: 3 }, expires_at_ms: expires },
  ) as MemoryRecord;
  const input = captured as Record<string, unknown>;
  assert.equal(input.key, "k1");
  assert.equal(input.payload, "hello");
  assert.deepEqual(input.metadata, { kind: { String: "doc" }, n: { Int: 3 } });
  assert.ok((input.ttl_ms as number) > 0 && (input.ttl_ms as number) <= 5000);
  assert.equal(out.id, "k1");
});

test("vanta_search: WASM hits array → desktop SearchResult[]", () => {
  const hits = [{ record: sdkRecord, score: 0.9, explanation: null }];
  const [r] = getWasmMapping("vanta_search").run(
    db({ search: () => hits }),
    { query: { query: "hi", namespace: "ns1" } },
  ) as SearchResult[];
  assert.equal(r.id, "k1");
  assert.equal(r.text, "hello");
  assert.equal(r.score, 0.9);
  assert.deepEqual(r.metadata, { kind: "doc", n: 3 });
});

test("vanta_get: WASM wire strings (u64 timestamps) and Float32Array vector coerced to the numeric desktop surface", () => {
  const out = getWasmMapping("vanta_get").run(
    db({
      get: () => ({
        namespace: "ns1",
        key: "k1",
        payload: "hello",
        metadata: {},
        created_at_ms: "100",
        updated_at_ms: "200",
        version: "1",
        node_id: "42",
        vector: new Float32Array([0.5, 0.25]),
        sparse_vector: null,
        expires_at_ms: null,
      }),
    }),
    { namespace: "ns1", key: "k1" },
  ) as MemoryRecord;
  assert.equal(out.created_at_ms, 100);
  assert.equal(out.updated_at_ms, 200);
  assert.equal(out.version, 1);
  assert.deepEqual(out.vector, [0.5, 0.25]);
});

test("vanta_query: rejected — WASM IQL reads resolve empty against the engine (not wired)", () => {
  assert.throws(() => getWasmMapping("vanta_query"), /vanta.*query/);
});

test("vanta_list: options forwarded, WASM page → desktop ListPage", () => {
  let captured: unknown;
  const out = getWasmMapping("vanta_list").run(
    db({
      list: (_ns: string, opts: unknown) => {
        captured = opts;
        return { records: [sdkRecord], next_cursor: 2 };
      },
    }),
    { namespace: "ns1", limit: 5, cursor: 1 },
  ) as { records: MemoryRecord[]; next_cursor: number | null };
  assert.deepEqual(captured, { limit: 5, cursor: 1 });
  assert.equal(out.records[0].id, "k1");
  assert.equal(out.next_cursor, 2);
});

test("vanta_ingest: IngestItem[] → tagged inputs → returned keys[] (genId for missing ids)", () => {
  let captured: unknown;
  const ids = getWasmMapping("vanta_ingest").run(
    db({
      put_batch: (inputs: unknown) => {
        captured = inputs;
        return [{ key: "a" }, { key: "b" }];
      },
    }),
    { records: [{ id: "a", text: "x", namespace: "ns1" }, { text: "y", namespace: "ns1" }] },
  ) as string[];
  assert.deepEqual(ids, ["a", "b"]);
  const inputs = captured as Record<string, unknown>[];
  assert.equal(inputs[0].key, "a");
  assert.equal(inputs[0].payload, "x");
  assert.match(inputs[1].key as string, /^rec_\d+_\d+$/);
});

test("vanta_delete_by_filter: filter tagged, bigint count coerced to number", () => {
  let captured: unknown;
  const n = getWasmMapping("vanta_delete_by_filter").run(
    db({
      delete_by_filter: (ns: string, filter: unknown) => {
        captured = filter;
        return 7n;
      },
    }),
    { namespace: "ns1", filter: [{ field: "kind", op: "Eq", value: "doc" }] },
  );
  assert.equal(n, 7);
  assert.deepEqual(captured, [{ field: "kind", op: "Eq", value: { String: "doc" } }]);
});

test("vanta_metrics: stringified u64 → numeric desktop interface (mmap null passthrough)", () => {
  const m = getWasmMapping("vanta_metrics").run(
    db({ operational_metrics: () => ({ hnsw_nodes_count: "7", process_rss_bytes: "123", mmap_resident_bytes: null }) }),
    {},
  ) as OperationalMetrics;
  assert.equal(m.hnsw_nodes_count, 7);
  assert.equal(m.process_rss_bytes, 123);
  assert.equal(m.mmap_resident_bytes, null);
});

test("vanta_health: engine probe → healthy wasm report", () => {
  const h = getWasmMapping("vanta_health").run(
    db({ operational_metrics: () => ({}) }),
    {},
  ) as HealthReport;
  assert.equal(h.status, "healthy");
  assert.equal(h.backend, "wasm");
  assert.equal(typeof h.checked_at_ms, "number");
});

test("mutating commands persist (OPFS/IDB save after run); read commands do not", () => {
  const persist = new Set(["vanta_put", "vanta_ingest", "vanta_ingest_batch", "vanta_delete", "vanta_delete_by_filter"]);
  for (const cmd of wasmMappedCommands()) {
    assert.equal(getWasmMapping(cmd).persist === true, persist.has(cmd), `${cmd} persist flag`);
  }
});

test("coverage: every vanta_* command has a mapping entry (real or documented unsupported)", () => {
  const allCommands = [
    "vanta_health",
    "vanta_connect",
    "vanta_disconnect",
    "vanta_list_connections",
    "vanta_set_active",
    "vanta_ingest",
    "vanta_ingest_batch",
    "vanta_search",
    "vanta_get",
    "vanta_get_version",
    "vanta_versions",
    "vanta_delete",
    "vanta_put",
    "vanta_list",
    "vanta_query",
    "vanta_iql_autocomplete",
    "vanta_export_namespace",
    "vanta_delete_by_filter",
    "vanta_graph_bfs",
    "vanta_graph_dfs",
    "vanta_graph_degree",
    "vanta_metrics",
    "vanta_namespace_stats",
    "vanta_audit_events",
  ];
  const known = new Set([...wasmMappedCommands(), ...wasmUnsupportedCommands()]);
  for (const cmd of allCommands) {
    assert.ok(known.has(cmd), `${cmd} has no entry in vanta-wasm-map.ts`);
  }
});

test("unsupported commands reject with a descriptive reason", () => {
  for (const cmd of wasmUnsupportedCommands()) {
    assert.throws(() => getWasmMapping(cmd), new RegExp(`vanta.*${cmd.replace("vanta_", "")}`));
  }
});