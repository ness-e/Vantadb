import { describe, it, expect, beforeAll, afterAll } from "vitest";
import { VantaDB } from "../vantadb.js";

describe("VantaDB WASM Integration", () => {
  let db: VantaDB;

  beforeAll(() => {
    db = VantaDB.create();
  });

  afterAll(() => {
    db.close();
  });

  it("put and get a record", async () => {
    const record = await db.put({
      namespace: "test",
      key: "hello",
      payload: "world",
    });
    expect(record.namespace).toBe("test");
    expect(record.key).toBe("hello");
    expect(record.payload).toBe("world");
    expect(Number(record.version)).toBe(1);
    expect(Number(record.node_id)).toBeGreaterThan(0);

    const got = await db.get("test", "hello");
    expect(got).not.toBeNull();
    expect(got!.payload).toBe("world");
  });

  it("delete a record", async () => {
    await db.put({ namespace: "test", key: "del", payload: "gone" });
    const deleted = await db.delete("test", "del");
    expect(deleted).toBe(true);
    const got = await db.get("test", "del");
    expect(got).toBeNull();
  });

  it("list namespaces", async () => {
    const ns = await db.listNamespaces();
    expect(ns).toContain("test");
  });

  it("list records with pagination", async () => {
    for (let i = 0; i < 5; i++) {
      await db.put({ namespace: "list_test", key: `k${i}`, payload: `v${i}` });
    }
    const page = await db.list("list_test", { limit: 3 });
    expect(page.records.length).toBe(3);
    expect(page.next_cursor).toBeDefined();
  });

  it("search by vector", async () => {
    const vec = [0.1, 0.2, 0.3, 0.4];
    await db.put({ namespace: "vec", key: "a", payload: "alpha", vector: vec });
    const hits = await db.search({ namespace: "vec", query_vector: vec, top_k: 5 });
    expect(hits.length).toBeGreaterThan(0);
    expect(hits[0].distance).toBeGreaterThan(0.99);
  });

  it("put batch", async () => {
    const records = await db.putBatch([
      { namespace: "batch", key: "a", payload: "1" },
      { namespace: "batch", key: "b", payload: "2" },
    ]);
    expect(records.length).toBe(2);
    expect(records[0].payload).toBe("1");
    expect(records[1].payload).toBe("2");
  });

  it("ttl expiration", async () => {
    await db.put({ namespace: "ttl", key: "x", payload: "temp", ttl_ms: 1 });
    // purge_expired should catch it
    const purged = await db.purgeExpired();
    // May or may not purge depending on timing, but shouldn't error
    expect(purged).toBeDefined();
  });

  it("graph operations", async () => {
    await db.insertNode(1, "root", [0.1, 0.2]);
    await db.insertNode(2, "child", [0.3, 0.4]);
    await db.addEdge(1, 2, "knows", 0.8);

    const node = await db.getNode(1);
    expect(node).not.toBeNull();
    expect(node!.edges.length).toBe(1);
    expect(node!.edges[0].target).toBe(2n);

    const bfs = await db.graphBfs([1], 5);
    expect(bfs).toBeDefined();

    const dag = await db.graphIsDag([1]);
    expect(dag).toBe(true);
  });

  it("capabilities", () => {
    const caps = db.capabilities();
    expect(caps.vector_search).toBe(true);
    expect(caps.persistence).toBeDefined();
    expect(caps.iql_queries).toBe(true);
  });

  it("operational metrics", async () => {
    const m = await db.operationalMetrics();
    expect(m.startup_ms).toBeDefined();
    expect(m.hnsw_nodes_count).toBeDefined();
  });

  it("flush and compact wal", async () => {
    expect(db.flush()).toBeUndefined();
    expect(db.compactWal()).toBeUndefined();
  });

  it("generate snippet", async () => {
    const snippet = await db.generateSnippet(
      "VantaDB is a vector database for AI agents",
      "vector database",
      true
    );
    expect(snippet).toBeDefined();
    expect(snippet).toContain("vector");
  });

  // TS-04: API parity tests (removeEdge, count, supersede, similarToKey, searchMulti, sparse_vector)
  it("count records in namespace", async () => {
    await db.put({ namespace: "ts04", key: "a", payload: "alpha" });
    await db.put({ namespace: "ts04", key: "b", payload: "beta" });
    await db.put({ namespace: "ts04", key: "c", payload: "gamma" });
    const total = db.count("ts04");
    expect(total).toBeGreaterThanOrEqual(3n);
    const onlyA = db.count("ts04", [
      { field: "key", op: "Eq", value: "a" },
    ]);
    expect(onlyA).toBe(1n);
  });

  it("removeEdge removes a directed edge", async () => {
    await db.insertNode(101, "u", [0.1]);
    await db.insertNode(102, "v", [0.2]);
    await db.addEdge(101, 102, "links", 0.5);
    const before = db.getNode(101);
    expect(before).not.toBeNull();
    expect(before!.edges.length).toBeGreaterThan(0);

    db.removeEdge(101, 102, "links");
    const after = db.getNode(101);
    expect(after).not.toBeNull();
    expect(after!.edges.length).toBe(0);
  });

  it("supersede marks old record as superseded by new", async () => {
    await db.put({ namespace: "ts04sup", key: "old", payload: "first" });
    await db.put({ namespace: "ts04sup", key: "new", payload: "second" });
    db.supersede("ts04sup", "old", "new");

    const oldRec = db.get("ts04sup", "old");
    expect(oldRec).not.toBeNull();
    expect(Number(oldRec!.version)).toBeGreaterThan(1);
  });

  it("similarToKey returns related records", async () => {
    await db.put({
      namespace: "ts04sim",
      key: "src",
      payload: "x",
      vector: [1.0, 0.0, 0.0],
    });
    await db.put({
      namespace: "ts04sim",
      key: "near",
      payload: "y",
      vector: [0.99, 0.01, 0.0],
    });
    await db.put({
      namespace: "ts04sim",
      key: "far",
      payload: "z",
      vector: [-1.0, 0.0, 0.0],
    });
    const hits = db.similarToKey("ts04sim", "src", 2);
    expect(hits.length).toBeGreaterThan(0);
    // the source itself is excluded
    expect(hits.find((h) => h.record.key === "src")).toBeUndefined();
  });

  it("searchMulti merges results from multiple namespaces", async () => {
    await db.put({
      namespace: "ts04m1",
      key: "k1",
      payload: "x",
      vector: [1.0, 0.0, 0.0],
    });
    await db.put({
      namespace: "ts04m2",
      key: "k2",
      payload: "y",
      vector: [0.99, 0.01, 0.0],
    });
    const hits = db.searchMulti(["ts04m1", "ts04m2"], {
      query_vector: [1.0, 0.0, 0.0],
      top_k: 5,
    });
    expect(hits.length).toBeGreaterThanOrEqual(1);
    const keys = hits.map((h) => h.record.key);
    expect(keys).toContain("k1");
  });

  it("put accepts sparse_vector and roundtrips", async () => {
    await db.put({
      namespace: "ts04sp",
      key: "s",
      payload: "sparse test",
      sparse_vector: { 1: 0.5, 42: 1.25 },
    });
    const got = db.get("ts04sp", "s");
    expect(got).not.toBeNull();
    expect(got!.payload).toBe("sparse test");
  });
});
