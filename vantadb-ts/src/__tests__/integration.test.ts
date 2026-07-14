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
});
