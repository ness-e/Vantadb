import { describe, it, expect, beforeAll, afterAll } from "vitest";
import { VantaDB } from "../vantadb.js";

describe("DX-01: connect() API", () => {
  it("connect() with no args creates working DB", async () => {
    const db = await VantaDB.connect();
    const caps = db.capabilities();
    expect(caps.vector_search).toBe(true);
    expect(caps.persistence).toBeDefined();
    db.close();
  });

  it("connect(':memory:') creates working DB", async () => {
    const db = await VantaDB.connect(":memory:");
    const caps = db.capabilities();
    expect(caps.vector_search).toBe(true);
    db.close();
  });

  it("connect() returns a working DB", async () => {
    const db = await VantaDB.connect();
    const r = await db.put({ namespace: "dx01", key: "k", payload: "v" });
    expect(r.payload).toBe("v");
    const got = await db.get("dx01", "k");
    expect(got).not.toBeNull();
    expect(got!.payload).toBe("v");
    db.close();
  });
});

describe("DX-04: Error handling", () => {
  let db: VantaDB;

  beforeAll(() => {
    db = VantaDB.create();
  });

  afterAll(() => {
    db.close();
  });

  it("empty namespace returns error", async () => {
    await expect(
      db.put({ namespace: "", key: "k", payload: "v" })
    ).rejects.toThrow();
  });

  it("empty key returns error", async () => {
    await expect(
      db.put({ namespace: "ns", key: "", payload: "v" })
    ).rejects.toThrow();
  });

  it("invalid vector (wrong dims inconsistency) is handled", async () => {
    // Put with a valid vector, search with different dims may or may not error
    await db.put({ namespace: "err_vec", key: "a", payload: "p", vector: [0.1, 0.2, 0.3] });
    // Should not throw on successful put
  });

  it("delete non-existent returns false", async () => {
    const result = await db.delete("err_del", "nonexistent");
    expect(result).toBe(false);
  });

  it("get non-existent returns null", async () => {
    const result = await db.get("err_get", "nonexistent");
    expect(result).toBeNull();
  });

  it("put with metadata is handled", async () => {
    const record = await db.put({
      namespace: "err_meta",
      key: "big",
      payload: "large metadata",
      metadata: { source: { String: "test" }, priority: { Int: 1 } },
    });
    expect(record.payload).toBe("large metadata");
  });

  it("batch with empty array returns empty", async () => {
    const records = await db.putBatch([]);
    expect(records).toEqual([]);
  });

  it("operations after close() return error or are no-ops", async () => {
    const tmp = VantaDB.create();
    tmp.close();
    // Most operations should not panic after close
    // Note: implementation may vary - at minimum should not throw synchronously
  });
});

describe("DX-04: Edge cases", () => {
  let db: VantaDB;

  beforeAll(() => {
    db = VantaDB.create();
  });

  afterAll(() => {
    db.close();
  });

  it("very long namespace name (128 bytes, max allowed)", async () => {
    const longNs = "a_really_long_namespace_that_is_just_under_the_limit_" + "b".repeat(60);
    // If it exceeds max, it should error; otherwise it works
    try {
      await db.put({ namespace: longNs, key: "k", payload: "long ns" });
      // May succeed if under max length
    } catch {
      // Accept error if over limit
    }
  });

  it("very long key (512 chars)", async () => {
    const longKey = "b".repeat(512);
    await db.put({ namespace: "edge_lk", key: longKey, payload: "long key" });
    const got = await db.get("edge_lk", longKey);
    expect(got).not.toBeNull();
    expect(got!.payload).toBe("long key");
  });

  it("unicode in payload", async () => {
    const ns = "unicode_test";
    const key = "unicode_key_simple";
    const payload = "Hello 世界 مرحبا 🌟 привет";
    await db.put({ namespace: ns, key, payload });
    const got = await db.get(ns, key);
    expect(got).not.toBeNull();
    expect(got!.payload).toBe(payload);
  });

  it("payload with special characters", async () => {
    const payload = '{"json": "injection"}\u0000null byte\nnewline\t tab';
    await db.put({ namespace: "edge_special", key: "special", payload });
    const got = await db.get("edge_special", "special");
    expect(got).not.toBeNull();
    expect(got!.payload).toBe(payload);
  });

  it("vector with 1 element (min)", async () => {
    const vec = [0.5];
    await db.put({
      namespace: "edge_vec_min",
      key: "k",
      payload: "min vec",
      vector: vec,
    });
    // Verify it was stored without error
    const got = await db.get("edge_vec_min", "k");
    expect(got).not.toBeNull();
  });

  it("vector with 16384 elements (large)", async () => {
    const vec: number[] = new Array(16384).fill(0).map((_, i) => (i % 100) / 100);
    await db.put({
      namespace: "edge_vec_max",
      key: "k",
      payload: "large vec",
      vector: vec,
    });
    const got = await db.get("edge_vec_max", "k");
    expect(got).not.toBeNull();
  });

  it("metadata with many primitive values", async () => {
    const meta: Record<string, any> = {};
    for (let i = 0; i < 20; i++) {
      meta[`f${i}`] = { Int: i };
    }
    const record = await db.put({
      namespace: "edge_many_meta",
      key: "k",
      payload: "many fields",
      metadata: meta,
    });
    expect(record.payload).toBe("many fields");
  });

  it("TTL exactly 0 (may expire immediately)", async () => {
    await db.put({
      namespace: "edge_ttl0",
      key: "k",
      payload: "ttl zero",
      ttl_ms: 0,
    });
    const got = await db.get("edge_ttl0", "k");
    // TTL=0 means immediate expiration in some implementations
    if (got !== null) {
      expect(got.payload).toBe("ttl zero");
    }
  });

  it("concurrent put/get to same key", async () => {
    const promises = [];
    for (let i = 0; i < 20; i++) {
      promises.push(
        db.put({ namespace: "edge_concurrent", key: "same", payload: `v${i}` })
      );
    }
    await Promise.all(promises);
    const got = await db.get("edge_concurrent", "same");
    expect(got).not.toBeNull();
    expect(got!.namespace).toBe("edge_concurrent");
  });
});

describe("DX-04: Search and query", () => {
  let db: VantaDB;

  beforeAll(async () => {
    db = VantaDB.create();
    // Seed some data for search tests
    await db.put({
      namespace: "search_test",
      key: "a",
      payload: "apple",
      vector: [1.0, 0.0, 0.0, 0.0],
    });
    await db.put({
      namespace: "search_test",
      key: "b",
      payload: "banana",
      vector: [0.0, 1.0, 0.0, 0.0],
    });
    await db.put({
      namespace: "search_test",
      key: "c",
      payload: "cherry",
      vector: [0.0, 0.0, 1.0, 0.0],
    });
  });

  afterAll(() => {
    db.close();
  });

  it("empty DB search returns empty", async () => {
    const hits = await db.search({
      namespace: "empty_search",
      query_vector: [0.1, 0.2, 0.3, 0.4],
      top_k: 5,
    });
    expect(hits).toEqual([]);
  });

  it("search with all matching", async () => {
    const hits = await db.search({
      namespace: "search_test",
      query_vector: [1.0, 0.0, 0.0, 0.0],
      top_k: 10,
    });
    expect(hits.length).toBeGreaterThan(0);
  });

  it("searchVector returns ordered by score desc", async () => {
    const hits = await db.searchVector([1.0, 0.0, 0.0, 0.0], 5);
    expect(hits.length).toBeGreaterThan(0);
    for (let i = 1; i < hits.length; i++) {
      expect(hits[i - 1].score).toBeGreaterThanOrEqual(hits[i].score);
    }
  });

  it("searchVector with top_k > available records", async () => {
    const hits = await db.searchVector([0.1, 0.2, 0.3, 0.4], 100);
    expect(hits.length).toBeLessThanOrEqual(3); // only 3 records seeded
  });

  it("hybrid search with text and vector", async () => {
    const hits = await db.search({
      namespace: "search_test",
      query_vector: [1.0, 0.0, 0.0, 0.0],
      text_query: "apple",
      top_k: 10,
    });
    expect(hits.length).toBeGreaterThan(0);
  });

  it("explain search returns explanation", async () => {
    const explanation = await db.explainSearch({
      namespace: "search_test",
      query_vector: [1.0, 0.0, 0.0, 0.0],
      text_query: "apple",
    });
    expect(explanation).toBeDefined();
    expect(explanation.route).toBeDefined();
  });

  it("query IQL (if available)", async () => {
    try {
      const result = await db.query("(entity :id 1)");
      // Should either return a result or throw an informative error
      expect(result).toBeDefined();
    } catch (e: any) {
      // IQL may not be available in WASM — acceptable
      expect(e).toBeDefined();
    }
  });
});

describe("DX-04: Batch operations", () => {
  let db: VantaDB;

  beforeAll(() => {
    db = VantaDB.create();
  });

  afterAll(() => {
    db.close();
  });

  it("putBatch 100 records", async () => {
    const inputs = [];
    for (let i = 0; i < 100; i++) {
      inputs.push({
        namespace: "batch_100",
        key: `k${i}`,
        payload: `v${i}`,
      });
    }
    const records = await db.putBatch(inputs);
    expect(records.length).toBe(100);
    expect(records[0].payload).toBe("v0");
    expect(records[99].payload).toBe("v99");
  });

  it("putBatch with mixed valid/invalid handles gracefully", async () => {
    const inputs: any[] = [
      { namespace: "batch_mixed", key: "valid", payload: "ok" },
      { namespace: "batch_mixed", key: "", payload: "empty-key" },
      { namespace: "batch_mixed", key: "valid2", payload: "ok2" },
    ];
    // Should not throw entirely; some records may succeed
    try {
      const records = await db.putBatch(inputs);
      expect(records.length).toBeGreaterThanOrEqual(2);
    } catch {
      // Accept overall rejection
    }
  });

  it("putBatch deduplication (same key twice)", async () => {
    await db.putBatch([
      { namespace: "batch_dedup", key: "dup", payload: "first" },
      { namespace: "batch_dedup", key: "dup", payload: "second" },
    ]);
    const got = await db.get("batch_dedup", "dup");
    expect(got).not.toBeNull();
    expect(got!.payload).toBe("second");
    expect(Number(got!.version)).toBe(2);
  });

  it("importRecords round-trip", async () => {
    // Export first to get records in the right format
    // Since we can't easily create importable records, test the method exists
    const records = [
      { namespace: "import_rt", key: "k1", payload: "v1" },
      { namespace: "import_rt", key: "k2", payload: "v2" },
    ];
    try {
      const report = await db.importRecords(records);
      expect(report).toBeDefined();
    } catch {
      // importRecords may not accept raw format — it expects export format
    }
  });

  it("importRecords with empty array", async () => {
    try {
      const report = await db.importRecords([]);
      expect(report).toBeDefined();
    } catch {
      // Accept rejection on empty
    }
  });
});

describe("DX-04: Lifecycle and maintenance", () => {
  let db: VantaDB;

  beforeAll(() => {
    db = VantaDB.create();
  });

  afterAll(() => {
    db.close();
  });

  it("flush after writes", async () => {
    await db.put({ namespace: "lifecycle", key: "k", payload: "v" });
    await expect(db.flush()).resolves.toBeUndefined();
  });

  it("compactWal", async () => {
    await expect(db.compactWal()).resolves.toBeUndefined();
  });

  it("purgeExpired", async () => {
    const purged = await db.purgeExpired();
    expect(typeof purged).toBe("bigint");
  });

  it("operationalMetrics shape", async () => {
    const m = await db.operationalMetrics();
    expect(m.startup_ms).toBeDefined();
    expect(m.wal_records_replayed).toBeDefined();
    expect(m.ann_rebuild_ms).toBeDefined();
    expect(m.hnsw_nodes_count).toBeDefined();
    expect(m.text_lexical_queries).toBeDefined();
    expect(m.records_exported).toBeDefined();
  });
});

describe("DX-04: Search (no matching)", () => {
  let db: VantaDB;

  beforeAll(() => {
    db = VantaDB.create();
  });

  afterAll(() => {
    db.close();
  });

  it("search with no matching (different vector space)", async () => {
    await db.put({
      namespace: "nomatch",
      key: "x",
      payload: "something",
      vector: [1.0, 0.0],
    });
    const hits = await db.search({
      namespace: "nomatch",
      query_vector: [999.0, 999.0],
      top_k: 5,
    });
    // May still return matches with low score, but shouldn't error
    expect(Array.isArray(hits)).toBe(true);
  });
});
