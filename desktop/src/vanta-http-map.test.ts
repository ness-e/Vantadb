// WEB-04 mapper tests — pure logic, no DOM/IPC (node --test, like
// vanta-deep-link.test.ts). Exercises the wire adaptation (SDK record ↔
// desktop DTO) and the coverage of every `vanta_*` command in vanta.ts.
import { test } from "node:test";
import assert from "node:assert/strict";
import {
  getHttpMapping,
  mappedCommands,
  unsupportedCommands,
} from "./vanta-http-map.ts";
import type { MemoryRecord, SearchResult, VantaQueryResult } from "./vanta.ts";

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

test("vanta_get: SDK record → desktop MemoryRecord (key→id, payload→text)", () => {
  const out = getHttpMapping("vanta_get").transform?.(sdkRecord) as MemoryRecord;
  assert.equal(out.id, "k1");
  assert.equal(out.namespace, "ns1");
  assert.equal(out.text, "hello");
  assert.equal(out.node_id, "42");
  assert.equal(out.version, 1);
  // VantaValue-tagged metadata is untagged on the desktop DTO.
  assert.deepEqual(out.metadata, { kind: "doc", n: 3 });
});

test("vanta_search: SDK hit → desktop SearchResult", () => {
  const hits = [{ record: sdkRecord, score: 0.9, explanation: null }];
  const [r] = getHttpMapping("vanta_search").transform?.(hits) as SearchResult[];
  assert.equal(r.id, "k1");
  assert.equal(r.text, "hello");
  assert.equal(r.score, 0.9);
  assert.deepEqual(r.metadata, { kind: "doc", n: 3 });
});

test("vanta_query Read: NodeDTO → MemoryRecord (node_id string, text from relational)", () => {
  const resp = {
    success: true,
    data: "Read 1 nodes.",
    node_id: null,
    nodes: [
      {
        id: 999888777666,
        semantic_cluster: 0,
        relational: {
          __vanta_payload: "hi",
          __vanta_namespace: "ns1",
          kind: "doc",
        },
        hits: 1,
        confidence_score: 0.9,
      },
    ],
  };
  const q = getHttpMapping("vanta_query").transform?.(resp) as VantaQueryResult;
  assert.ok("Read" in q);
  assert.equal(q.Read[0].node_id, "999888777666");
  assert.equal(q.Read[0].text, "hi");
  assert.equal(q.Read[0].namespace, "ns1");
  assert.deepEqual(q.Read[0].metadata, { kind: "doc" });
});

test("vanta_query Write: 'Mutated N nodes: msg' → {Write}", () => {
  const q = getHttpMapping("vanta_query").transform?.({
    success: true,
    data: "Mutated 3 nodes: created k1",
    node_id: 77,
    nodes: null,
  }) as VantaQueryResult;
  assert.ok("Write" in q);
  assert.equal(q.Write.affected_nodes, 3);
  assert.equal(q.Write.message, "created k1");
  assert.equal(q.Write.node_id, "77");
});

test("vanta_query StaleContext → {StaleContext}", () => {
  const q = getHttpMapping("vanta_query").transform?.({
    success: true,
    data: "STALE_CONTEXT: Confidence Score critical. Rehydration available for summary 5",
    node_id: 5,
    nodes: null,
  }) as VantaQueryResult;
  assert.ok("StaleContext" in q);
  assert.equal(q.StaleContext.node_id, "5");
});

test("vanta_query success:false throws with the server message", () => {
  assert.throws(
    () =>
      getHttpMapping("vanta_query").transform?.({
        success: false,
        data: "Server query pool closed",
        node_id: null,
        nodes: null,
      }),
    /pool closed/,
  );
});

test("vanta_put: metadata tagged to VantaValue, ttl computed from expires_at_ms", () => {
  const body = getHttpMapping("vanta_put").body?.({
    namespace: "ns1",
    key: "k1",
    payload: "p",
    metadata: { kind: "doc", n: 3, tags: ["a", "b"] },
  }) as Record<string, unknown>;
  assert.equal(body.key, "k1");
  assert.equal(body.payload, "p");
  assert.deepEqual(body.metadata, {
    kind: { String: "doc" },
    n: { Int: 3 },
    tags: { ListString: ["a", "b"] },
  });
  assert.equal(body.ttl_ms, null);

  const future = getHttpMapping("vanta_put").body?.({
    key: "k2",
    payload: "p2",
    expires_at_ms: Date.now() + 60_000,
  }) as Record<string, unknown>;
  assert.equal(typeof future.ttl_ms, "number");
  assert.ok((future.ttl_ms as number) >= 0);
});

test("vanta_search: SearchQuery → VantaMemorySearchRequest (wire shape)", () => {
  const body = getHttpMapping("vanta_search").body?.({
    query: { query: "hi", namespace: "ns1", top_k: 5, filters: { kind: "doc" } },
  });
  assert.deepEqual(body, {
    namespace: "ns1",
    query_vector: [],
    query_sparse: null,
    filters: { kind: { String: "doc" } },
    text_query: "hi",
    top_k: 5,
    distance_metric: "Cosine",
    explain: false,
  });
});

test("vanta_ingest: IngestItem[] → inputs[] with generated key, response → ids", () => {
  const body = getHttpMapping("vanta_ingest").body?.({
    records: [
      { id: "a", text: "x", namespace: "ns1" },
      { text: "y", namespace: "ns1" },
    ],
  }) as Record<string, unknown>[];
  assert.equal(body.length, 2);
  assert.equal(body[0].key, "a");
  assert.equal(body[0].payload, "x");
  assert.equal(body[0].namespace, "ns1");
  assert.match(body[1].key as string, /^rec_\d+_\d+$/);

  const ids = getHttpMapping("vanta_ingest").transform?.([
    { key: "a", id: 1 },
    { key: "b", id: 2 },
  ]);
  assert.deepEqual(ids, ["a", "b"]);
});

test("vanta_list: VantaMemoryListPage → desktop ListPage", () => {
  const page = getHttpMapping("vanta_list").transform?.({
    records: [sdkRecord],
    next_cursor: 1,
  }) as { records: MemoryRecord[]; next_cursor: number | null };
  assert.equal(page.records[0].id, "k1");
  assert.equal(page.next_cursor, 1);
});

test("vanta_delete_by_filter: filter tagged to VantaMemoryFilter wire", () => {
  const query = getHttpMapping("vanta_delete_by_filter").query?.({
    namespace: "ns1",
    filter: [{ field: "kind", op: "Eq", value: "doc" }],
  });
  assert.equal(query.namespace, "ns1");
  assert.equal(
    query.filter,
    JSON.stringify([{ field: "kind", op: "Eq", value: { String: "doc" } }]),
  );
  const out = getHttpMapping("vanta_delete_by_filter").transform?.({ deleted: 7 });
  assert.equal(out, 7);
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
    "vanta_audit_events",
  ];
  const known = new Set([...mappedCommands(), ...unsupportedCommands()]);
  for (const cmd of allCommands) {
    assert.ok(known.has(cmd), `${cmd} has no entry in vanta-http-map.ts`);
  }
});

test("mapped endpoints resolve under /api/v2/", () => {
  for (const cmd of mappedCommands()) {
    const m = getHttpMapping(cmd);
    const path = m.path({});
    assert.ok(path.startsWith("/api/v2/"), `${cmd} path ${path} not under /api/v2/`);
  }
});

test("unsupported commands reject with a descriptive reason", () => {
  for (const cmd of unsupportedCommands()) {
    assert.throws(() => getHttpMapping(cmd), new RegExp(`vanta.*${cmd.replace("vanta_", "")}`));
  }
});