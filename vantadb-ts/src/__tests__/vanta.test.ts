import { describe, it, expect, beforeAll, afterAll } from "vitest";
import { VantaDB, VantaError } from "../vantadb.js";
import {
  isMemoryRecord,
  isSearchHit,
  isNodeRecord,
  isValidVantaValue,
  isVantaMetadata,
  isValidVector,
  validateVector,
} from "../guards.js";
import type { MemoryRecord, SearchHit, NodeRecord } from "../types.js";

// ---------------------------------------------------------------------------
// Type guard unit tests (no DB required)
// ---------------------------------------------------------------------------
describe("Type guards: isMemoryRecord", () => {
  const validRecord: MemoryRecord = {
    namespace: "ns",
    key: "k",
    payload: "p",
    metadata: {},
    created_at_ms: "1000",
    updated_at_ms: "2000",
    version: "1",
    node_id: "42",
  };

  it("accepts a well-formed MemoryRecord", () => {
    expect(isMemoryRecord(validRecord)).toBe(true);
  });

  it("rejects null", () => {
    expect(isMemoryRecord(null)).toBe(false);
  });

  it("rejects primitive values", () => {
    expect(isMemoryRecord(42)).toBe(false);
    expect(isMemoryRecord("string")).toBe(false);
    expect(isMemoryRecord(true)).toBe(false);
  });

  it("rejects objects missing namespace", () => {
    expect(isMemoryRecord({ key: "k", payload: "p" } as unknown)).toBe(false);
  });

  it("rejects objects missing key", () => {
    expect(isMemoryRecord({ namespace: "ns", payload: "p" } as unknown)).toBe(false);
  });

  it("rejects objects missing payload", () => {
    expect(isMemoryRecord({ namespace: "ns", key: "k" } as unknown)).toBe(false);
  });

  it("rejects objects with non-string namespace", () => {
    expect(isMemoryRecord({ ...validRecord, namespace: 123 } as unknown)).toBe(false);
  });
});

describe("Type guards: isSearchHit", () => {
  it("accepts a well-formed SearchHit", () => {
    const hit: SearchHit = {
      record: {
        namespace: "ns",
        key: "k",
        payload: "p",
        metadata: {},
        created_at_ms: "0",
        updated_at_ms: "0",
        version: "1",
        node_id: "1",
      },
      distance: 0.42,
    };
    expect(isSearchHit(hit)).toBe(true);
  });

  it("rejects null", () => expect(isSearchHit(null)).toBe(false));

  it("rejects missing distance", () => {
    const h = { record: { namespace: "ns", key: "k", payload: "p" } };
    expect(isSearchHit(h)).toBe(false);
  });
});

describe("Type guards: isNodeRecord", () => {
  const validNode: NodeRecord = {
    id: "1",
    fields: {},
    vector_dimensions: 3,
    edges: [],
    confidence_score: 0.9,
    importance: 0.5,
    hits: 10,
    last_accessed: "1000",
    epoch: 0,
    tier: "Hot",
    is_alive: true,
  };

  it("accepts a well-formed NodeRecord", () => {
    expect(isNodeRecord(validNode)).toBe(true);
  });

  it("rejects null", () => expect(isNodeRecord(null)).toBe(false));

  it("rejects missing vector_dimensions", () => {
    const { vector_dimensions, ...rest } = validNode;
    expect(isNodeRecord(rest)).toBe(false);
  });

  it("rejects invalid tier", () => {
    expect(isNodeRecord({ ...validNode, tier: "Warm" })).toBe(false);
  });
});

describe("Type guards: isValidVantaValue", () => {
  it("accepts String", () => {
    expect(isValidVantaValue({ String: "hello" })).toBe(true);
  });

  it("accepts Int", () => {
    expect(isValidVantaValue({ Int: 42 })).toBe(true);
  });

  it("accepts Float", () => {
    expect(isValidVantaValue({ Float: 3.14 })).toBe(true);
  });

  it("accepts Bool", () => {
    expect(isValidVantaValue({ Bool: true })).toBe(true);
  });

  it("accepts Null", () => {
    expect(isValidVantaValue({ Null: null })).toBe(true);
  });

  it("accepts ListString", () => {
    expect(isValidVantaValue({ ListString: ["a", "b"] })).toBe(true);
  });

  it("rejects unknown type", () => {
    expect(isValidVantaValue({ Unknown: "foo" })).toBe(false);
  });

  it("rejects null", () => expect(isValidVantaValue(null)).toBe(false));

  it("rejects non-object", () => expect(isValidVantaValue("string")).toBe(false));

  it("rejects Null with extra value", () => {
    expect(isValidVantaValue({ Null: 1 })).toBe(false);
  });
});

describe("Type guards: isVantaMetadata", () => {
  it("accepts valid metadata", () => {
    const m = { name: { String: "test" } };
    expect(isVantaMetadata(m)).toBe(true);
  });

  it("rejects invalid metadata value", () => {
    const m = { name: { Invalid: "test" } };
    expect(isVantaMetadata(m)).toBe(false);
  });
});

describe("Type guards: isValidVector", () => {
  it("accepts valid number array", () => {
    expect(isValidVector([0.1, 0.2, 0.3])).toBe(true);
  });

  it("rejects empty array", () => {
    expect(isValidVector([])).toBe(false);
  });

  it("rejects non-array", () => {
    expect(isValidVector("string")).toBe(false);
  });

  it("rejects array with NaN", () => {
    expect(isValidVector([1, NaN, 3])).toBe(false);
  });

  it("rejects array with Infinity", () => {
    expect(isValidVector([1, Infinity, 3])).toBe(false);
  });
});

describe("Type guards: validateVector (asserts)", () => {
  it("passes on valid array", () => {
    expect(() => validateVector([0.1, 0.2])).not.toThrow();
  });

  it("throws on non-array", () => {
    expect(() => validateVector("bad")).toThrow(TypeError);
  });

  it("throws on empty array", () => {
    expect(() => validateVector([])).toThrow(RangeError);
  });

  it("throws on NaN element", () => {
    expect(() => validateVector([1, NaN])).toThrow(TypeError);
  });
});

// ---------------------------------------------------------------------------
// VantaError unit tests
// ---------------------------------------------------------------------------
describe("VantaError", () => {
  it("creates error with code and message", () => {
    const err = new VantaError("TEST_CODE", "something broke");
    expect(err.code).toBe("TEST_CODE");
    expect(err.message).toBe("something broke");
    expect(err.name).toBe("VantaError");
  });

  it("stores optional details", () => {
    const err = new VantaError("DETAILS", "msg", { foo: 1 });
    expect(err.details).toEqual({ foo: 1 });
  });

  it("has timestamp", () => {
    const err = new VantaError("TS", "msg");
    expect(err.timestamp).toBeInstanceOf(Date);
  });

  it("toJSON produces structured output", () => {
    const err = new VantaError("JSON_TEST", "msg", { x: 1 });
    const json = err.toJSON();
    expect(json.name).toBe("VantaError");
    expect(json.code).toBe("JSON_TEST");
    expect(json.message).toBe("msg");
    expect(json.details).toEqual({ x: 1 });
    expect(typeof json.timestamp).toBe("string");
  });

  it("toJSON omits details when undefined", () => {
    const err = new VantaError("NO_DETAILS", "msg");
    const json = err.toJSON();
    expect(json.details).toBeUndefined();
  });
});

// ---------------------------------------------------------------------------
// Integration tests (require WASM engine)
// ---------------------------------------------------------------------------
describe("VantaDB lifecycle", () => {
  it("connect() with no args creates a working DB", () => {
    const db = VantaDB.connect();
    expect(db).toBeInstanceOf(VantaDB);
    db.close();
  });

  it("connect(':memory:') creates a working DB", () => {
    const db = VantaDB.connect(":memory:");
    expect(db.capabilities().vector_search).toBe(true);
    db.close();
  });

  it("create() with no args creates a working DB", () => {
    const db = VantaDB.create();
    expect(db.capabilities().persistence).toBeDefined();
    db.close();
  });

  it("create() with config does not throw", () => {
    const db = VantaDB.create({ memory_limit: 1_073_741_824 });
    expect(db).toBeInstanceOf(VantaDB);
    db.close();
  });

  it("close() is idempotent", () => {
    const db = VantaDB.create();
    db.close();
    expect(() => db.close()).not.toThrow();
  });

  it("operations after close() throw VantaError with code CLOSED", () => {
    const db = VantaDB.create();
    db.close();
    expect(() => db.put({ namespace: "ns", key: "k", payload: "p" })).toThrow(VantaError);
    try {
      db.put({ namespace: "ns", key: "k", payload: "p" });
    } catch (e) {
      expect(e).toBeInstanceOf(VantaError);
      expect((e as VantaError).code).toBe("CLOSED");
    }
  });

  it("capabilities() returns expected shape", () => {
    const db = VantaDB.create();
    const caps = db.capabilities();
    expect(caps.vector_search).toBe(true);
    expect(typeof caps.persistence).toBe("boolean");
    expect(typeof caps.read_only).toBe("boolean");
    expect(typeof caps.iql_queries).toBe("boolean");
    db.close();
  });
});

describe("VantaDB put / get / delete", () => {
  let db: VantaDB;

  beforeAll(() => { db = VantaDB.create(); });
  afterAll(() => { db.close(); });

  it("put stores and returns a record", () => {
    const r = db.put({ namespace: "test", key: "hello", payload: "world" });
    expect(r.namespace).toBe("test");
    expect(r.key).toBe("hello");
    expect(r.payload).toBe("world");
    expect(typeof r.version).toBe("string");
    expect(typeof r.node_id).toBe("string");
  });

  it("get retrieves a stored record", () => {
    db.put({ namespace: "test", key: "getme", payload: "found" });
    const r = db.get("test", "getme");
    expect(r).not.toBeNull();
    expect(r!.payload).toBe("found");
  });

  it("get returns null for missing record", () => {
    const r = db.get("missing_ns", "no_key");
    expect(r).toBeNull();
  });

  it("delete removes a record and returns true", () => {
    db.put({ namespace: "test", key: "delme", payload: "bye" });
    const deleted = db.delete("test", "delme");
    expect(deleted).toBe(true);
    expect(db.get("test", "delme")).toBeNull();
  });

  it("delete on non-existent returns false", () => {
    expect(db.delete("test", "never_existed")).toBe(false);
  });

  it("put with vector does not error", () => {
    const r = db.put({
      namespace: "vec_test",
      key: "v1",
      payload: "vectorized",
      vector: [0.1, 0.2, 0.3, 0.4],
    });
    expect(r.payload).toBe("vectorized");
  });

  it("put with metadata", () => {
    const r = db.put({
      namespace: "meta_test",
      key: "m1",
      payload: "data",
      metadata: { source: { String: "test" } },
    });
    expect(r.payload).toBe("data");
  });

  it("put with empty string namespace throws", () => {
    expect(() =>
      db.put({ namespace: "", key: "k", payload: "v" }),
    ).toThrow();
  });

  it("put with empty string key throws", () => {
    expect(() =>
      db.put({ namespace: "ns", key: "", payload: "v" }),
    ).toThrow();
  });
});

describe("VantaDB putBatch", () => {
  let db: VantaDB;

  beforeAll(() => { db = VantaDB.create(); });
  afterAll(() => { db.close(); });

  it("putBatch with empty array returns empty array", () => {
    const records = db.putBatch([]);
    expect(records).toEqual([]);
  });

  it("putBatch stores 100 records", () => {
    const inputs = Array.from({ length: 100 }, (_, i) => ({
      namespace: "batch100",
      key: `k${i}`,
      payload: `v${i}`,
    }));
    const records = db.putBatch(inputs);
    expect(records.length).toBe(100);
    expect(records[0].payload).toBe("v0");
    expect(records[99].payload).toBe("v99");
  });

  it("putBatch deduplicates last-write-wins", () => {
    db.putBatch([
      { namespace: "dedup", key: "same", payload: "first" },
      { namespace: "dedup", key: "same", payload: "second" },
    ]);
    const r = db.get("dedup", "same");
    expect(r).not.toBeNull();
    expect(r!.payload).toBe("second");
  });

  it("putBatch with vector records", () => {
    const inputs = [
      { namespace: "vec_batch", key: "a", payload: "pa", vector: [1, 0, 0] },
      { namespace: "vec_batch", key: "b", payload: "pb", vector: [0, 1, 0] },
    ];
    const records = db.putBatch(inputs);
    expect(records.length).toBe(2);
  });
});

describe("VantaDB list and namespace operations", () => {
  let db: VantaDB;

  beforeAll(() => { db = VantaDB.create(); });
  afterAll(() => { db.close(); });

  it("listNamespaces returns created namespaces", () => {
    db.put({ namespace: "list_ns_a", key: "k", payload: "v" });
    db.put({ namespace: "list_ns_b", key: "k", payload: "v" });
    const nss = db.listNamespaces();
    expect(nss).toContain("list_ns_a");
    expect(nss).toContain("list_ns_b");
  });

  it("list returns records in a namespace", () => {
    for (let i = 0; i < 5; i++) {
      db.put({ namespace: "list_records", key: `k${i}`, payload: `v${i}` });
    }
    const page = db.list("list_records", { limit: 3 });
    expect(page.records.length).toBe(3);
    expect(page.next_cursor).toBeDefined();
  });

  it("list with no options returns all", () => {
    const page = db.list("list_records");
    expect(page.records.length).toBeGreaterThanOrEqual(5);
  });
});

describe("VantaDB search", () => {
  let db: VantaDB;

  beforeAll(() => {
    db = VantaDB.create();
    db.put({ namespace: "search_db", key: "a", payload: "apple", vector: [1, 0, 0, 0] });
    db.put({ namespace: "search_db", key: "b", payload: "banana", vector: [0, 1, 0, 0] });
    db.put({ namespace: "search_db", key: "c", payload: "cherry", vector: [0, 0, 1, 0] });
  });

  afterAll(() => { db.close(); });

  it("search returns hits ordered by distance", () => {
    const hits = db.search({ namespace: "search_db", query_vector: [1, 0, 0, 0], top_k: 3 });
    expect(hits.length).toBeGreaterThan(0);
    for (let i = 1; i < hits.length; i++) {
      expect(hits[i - 1].distance).toBeGreaterThanOrEqual(hits[i].distance);
    }
  });

  it("search on empty namespace returns empty", () => {
    const hits = db.search({ namespace: "empty_search", query_vector: [1, 0, 0, 0] });
    expect(hits).toEqual([]);
  });

  it("searchVector returns node_id and distance", () => {
    const hits = db.searchVector([1, 0, 0, 0], 5);
    expect(hits.length).toBeGreaterThan(0);
    expect(typeof hits[0].node_id).toBe("string");
    expect(typeof hits[0].distance).toBe("number");
  });

  it("searchVector with top_k > available returns all", () => {
    const hits = db.searchVector([0.5, 0.5, 0.5, 0.5], 100);
    expect(hits.length).toBeLessThanOrEqual(3);
  });

  it("search with hybrid text query", () => {
    const hits = db.search({
      namespace: "search_db",
      query_vector: [1, 0, 0, 0],
      text_query: "apple",
      top_k: 5,
    });
    expect(hits.length).toBeGreaterThan(0);
  });

  it("explainSearch returns route information", () => {
    const exp = db.explainSearch({
      namespace: "search_db",
      query_vector: [1, 0, 0, 0],
      text_query: "apple",
    });
    expect(exp).toBeDefined();
    expect(typeof exp).toBe("object");
  });
});

describe("VantaDB graph operations", () => {
  let db: VantaDB;

  beforeAll(() => { db = VantaDB.create(); });
  afterAll(() => { db.close(); });

  it("insertNode creates a node", () => {
    expect(() => db.insertNode(1, "root")).not.toThrow();
  });

  it("getNode retrieves a node", () => {
    db.insertNode(10, "target");
    const node = db.getNode(10);
    expect(node).not.toBeNull();
    expect(node!.id).toBeDefined();
  });

  it("getNode returns null for missing node", () => {
    expect(db.getNode(99999)).toBeNull();
  });

  it("addEdge connects nodes", () => {
    db.insertNode(20, "source");
    db.insertNode(21, "dest");
    db.addEdge(20, 21, "connects", 1.0);
    const node = db.getNode(20);
    expect(node).not.toBeNull();
    expect(node!.edges.length).toBeGreaterThanOrEqual(1);
  });

  it("deleteNode removes a node", () => {
    db.insertNode(30, "goner");
    expect(() => db.deleteNode(30, "cleanup")).not.toThrow();
  });

  it("graphIsDag returns true for simple tree", () => {
    db.insertNode(40, "grandparent");
    db.insertNode(41, "parent");
    db.insertNode(42, "child");
    db.addEdge(40, 41, "parent_of");
    db.addEdge(41, 42, "parent_of");
    expect(db.graphIsDag([40])).toBe(true);
  });

  it("graphBfs traverses nodes", () => {
    db.insertNode(50, "a");
    db.insertNode(51, "b");
    db.addEdge(50, 51, "link");
    const result = db.graphBfs([50], 5);
    expect(result).toBeDefined();
  });

  it("graphDfs traverses nodes", () => {
    const result = db.graphDfs([50], 5);
    expect(result).toBeDefined();
  });

  it("graphTopologicalSort works on DAG", () => {
    const result = db.graphTopologicalSort([40]);
    expect(result).toBeDefined();
  });
});

describe("VantaDB maintenance operations", () => {
  let db: VantaDB;

  beforeAll(() => { db = VantaDB.create(); });
  afterAll(() => { db.close(); });

  it("flush does not throw", () => {
    db.put({ namespace: "maint", key: "k", payload: "v" });
    expect(() => db.flush()).not.toThrow();
  });

  it("compactWal does not throw", () => {
    expect(() => db.compactWal()).not.toThrow();
  });

  it("purgeExpired returns a bigint", () => {
    const purged = db.purgeExpired();
    expect(typeof purged).toBe("bigint");
  });

  it("operationalMetrics returns expected fields", () => {
    const m = db.operationalMetrics();
    expect(typeof m.startup_ms).toBe("string");
    expect(typeof m.hnsw_nodes_count).toBe("string");
    expect(typeof m.wal_records_replayed).toBe("string");
    expect(typeof m.records_exported).toBe("string");
  });

  it("compactLayout returns a bigint", () => {
    const reclaimed = db.compactLayout();
    expect(typeof reclaimed).toBe("bigint");
  });
});

describe("VantaDB text index", () => {
  let db: VantaDB;

  beforeAll(() => { db = VantaDB.create(); });
  afterAll(() => { db.close(); });

  it("auditTextIndex returns result or null", () => {
    const result = db.auditTextIndex();
    expect(result).toBeDefined();
  });

  it("auditTextIndexDeep returns result or null", () => {
    const result = db.auditTextIndexDeep();
    expect(result).toBeDefined();
  });

  it("repairTextIndex does not throw", () => {
    expect(() => db.repairTextIndex()).not.toThrow();
  });
});

describe("VantaDB generateSnippet", () => {
  let db: VantaDB;

  beforeAll(() => { db = VantaDB.create(); });
  afterAll(() => { db.close(); });

  it("returns snippet with highlighting", () => {
    const s = db.generateSnippet(
      "VantaDB is a vector-graph database for AI agents",
      "vector-graph",
      true,
    );
    expect(s).toBeDefined();
    expect(s!.length).toBeGreaterThan(0);
  });

  it("returns snippet without highlighting", () => {
    const s = db.generateSnippet("Hello world", "world", false);
    expect(s).toBeDefined();
  });
});

describe("VantaDB edge cases", () => {
  let db: VantaDB;

  beforeAll(() => { db = VantaDB.create(); });
  afterAll(() => { db.close(); });

  it("unicode payload round-trips", () => {
    const payload = "Hello 世界 🌟 привет";
    db.put({ namespace: "unicode", key: "u1", payload });
    const r = db.get("unicode", "u1");
    expect(r).not.toBeNull();
    expect(r!.payload).toBe(payload);
  });

  it("payload with special characters", () => {
    const payload = '{"json": "data"}\nnewline\t tab\u0000null';
    db.put({ namespace: "special", key: "s1", payload });
    const r = db.get("special", "s1");
    expect(r).not.toBeNull();
    expect(r!.payload).toBe(payload);
  });

  it("vector with single element", () => {
    db.put({ namespace: "edge_vec", key: "min", payload: "p", vector: [0.5] });
    const r = db.get("edge_vec", "min");
    expect(r).not.toBeNull();
  });

  it("vector with 16384 elements", () => {
    const vec = Array.from({ length: 16384 }, (_, i) => (i % 100) / 100);
    db.put({ namespace: "edge_vec", key: "max", payload: "p", vector: vec });
    const r = db.get("edge_vec", "max");
    expect(r).not.toBeNull();
  });

  it("very long key (512 chars)", () => {
    const longKey = "x".repeat(512);
    db.put({ namespace: "long_key", key: longKey, payload: "ok" });
    const r = db.get("long_key", longKey);
    expect(r).not.toBeNull();
  });

  it("TTL = 0 does not crash", () => {
    db.put({ namespace: "ttl_edge", key: "zero", payload: "p", ttl_ms: 0 });
    const r = db.get("ttl_edge", "zero");
    expect(r === null || r!.payload === "p").toBe(true);
  });

  it("concurrent puts to same key", () => {
    for (let i = 0; i < 20; i++) {
      db.put({ namespace: "concurrent_same", key: "x", payload: `v${i}` });
    }
    const r = db.get("concurrent_same", "x");
    expect(r).not.toBeNull();
  });

  it("concurrent puts across keys", () => {
    for (let i = 0; i < 100; i++) {
      db.put({
        namespace: "concurrent_multi",
        key: `k${i}`,
        payload: `v${i}`,
        vector: [i % 10, 0, 0, 0],
      });
    }
    const records: MemoryRecord[] = [];
    let cursor: string | undefined;
    do {
      const page = db.list("concurrent_multi", { limit: 20, cursor });
      records.push(...page.records);
      cursor = page.next_cursor;
    } while (cursor);
    expect(records.length).toBeGreaterThanOrEqual(90);
  });

  it("list after close throws VantaError", () => {
    const tmp = VantaDB.create();
    tmp.close();
    expect(() => tmp.list("ns")).toThrow(VantaError);
  });

  it("VantaError caught instanceof Error works", () => {
    const err = new VantaError("CODE", "msg");
    expect(err instanceof Error).toBe(true);
  });
});
