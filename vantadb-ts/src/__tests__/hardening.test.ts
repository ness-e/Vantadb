import { describe, it, expect, vi, beforeAll, afterAll } from "vitest";
import { Client, DbError } from "../vantadb.js";
import { wrapWasmError, classifyWasmError, ERROR_CODES } from "../errors.js";
import {
  isMemoryRecord,
  isSearchHit,
  isNodeRecord,
  isValidValue,
  isMetadata,
  isValidVector,
} from "../guards.js";
import type { MemoryRecord, SearchHit, NodeRecord, SearchRequest, ImportReport } from "../types.js";

describe("DbError serialization", () => {
  it("toJSON includes all fields", () => {
    const err = new DbError("TEST", "msg", { key: "val" });
    const json = err.toJSON();
    expect(json.name).toBe("VantaError");
    expect(json.code).toBe("TEST");
    expect(json.message).toBe("msg");
    expect(json.details).toEqual({ key: "val" });
    expect(typeof json.timestamp).toBe("string");
  });

  it("toJSON omits details when undefined", () => {
    const json = new DbError("NO_DETAILS", "msg").toJSON();
    expect(json.details).toBeUndefined();
  });

  it("DbError is instanceof Error", () => {
    expect(new DbError("C", "m") instanceof Error).toBe(true);
  });

  it("DbError has stack trace", () => {
    const err = new DbError("STACK", "trace");
    expect(typeof err.stack).toBe("string");
  });
});

describe("wrapWasmError", () => {
  it("passes through DbError", () => {
    const original = new DbError("EXISTING", "already wrapped");
    expect(wrapWasmError(original, "context")).toBe(original);
  });

  it("preserves the original error as cause (ERR-TS-01 §4.3)", () => {
    const original = new Error("root boom");
    const wrapped = wrapWasmError(original, "op");
    expect(wrapped.cause).toBe(original);
  });

  it("wraps Error with context prefix", () => {
    const wrapped = wrapWasmError(new Error("boom"), "myFunc");
    expect(wrapped).toBeInstanceOf(DbError);
    expect(wrapped.code).toBe(ERROR_CODES.WASM_ERROR);
    expect(wrapped.message).toBe("myFunc: boom");
  });

  it("wraps string with context prefix", () => {
    const wrapped = wrapWasmError("raw string", "test");
    expect(wrapped.message).toBe("test: raw string");
  });

  it("wraps null/undefined", () => {
    const wrapped = wrapWasmError(null, "nullCase");
    expect(wrapped.message).toBe("nullCase: null");
  });
});

describe("WASM error classification (FIND-10)", () => {
  it("uses the structured code attached by the wasm binding", () => {
    const err = new Error("Node not found: 1") as Error & { code?: string };
    err.code = "VANTADB_NOT_FOUND";
    const wrapped = wrapWasmError(err, "addEdge");
    expect(wrapped.code).toBe(ERROR_CODES.NOT_FOUND);
    expect(wrapped.message).toBe("addEdge: Node not found: 1");
  });

  it("ignores unknown codes (e.g. Node ERR_*) and falls back to message classification", () => {
    const err = new Error("completely unexpected") as Error & { code?: string };
    err.code = "ERR_MODULE_NOT_FOUND";
    const wrapped = wrapWasmError(err, "open");
    expect(wrapped.code).toBe(ERROR_CODES.WASM_ERROR);
  });

  it("classifies not-found by message prefix (older pkg fallback)", () => {
    const wrapped = wrapWasmError(new Error("Node not found: 42"), "getNode");
    expect(wrapped.code).toBe(ERROR_CODES.NOT_FOUND);
  });

  it("classifies validation errors by message prefix", () => {
    const wrapped = wrapWasmError(
      new Error("Validation error on namespace: namespace must not be empty"),
      "get",
    );
    expect(wrapped.code).toBe(ERROR_CODES.VALIDATION_ERROR);
  });

  it("classifies corrupt-format errors by message prefix", () => {
    const wrapped = wrapWasmError(
      new Error("Incompatible binary format: expected magic [1,2,3,4], version 1"),
      "open",
    );
    expect(wrapped.code).toBe(ERROR_CODES.CORRUPT);
  });

  it("classifyWasmError returns WASM_ERROR for unrecognized messages", () => {
    expect(classifyWasmError("completely unexpected")).toBe(ERROR_CODES.WASM_ERROR);
  });
});

describe("Client error codes from the real WASM engine (FIND-10)", () => {
  let db: Client;

  beforeAll(() => { db = Client.create(); });
  afterAll(() => { db.close(); });

  it("zero-norm cosine search surfaces VANTADB_VALIDATION_ERROR", () => {
    let caught: unknown;
    try {
      db.search({ namespace: "err", query_vector: [0, 0, 0], top_k: 5 });
    } catch (err) {
      caught = err;
    }
    expect(caught).toBeInstanceOf(DbError);
    expect((caught as DbError).code).toBe(ERROR_CODES.VALIDATION_ERROR);
  });

  it("addEdge with missing nodes surfaces VANTADB_NOT_FOUND", () => {
    let caught: unknown;
    try {
      db.addEdge(888, 999, "test");
    } catch (err) {
      caught = err;
    }
    expect(caught).toBeInstanceOf(DbError);
    expect((caught as DbError).code).toBe(ERROR_CODES.NOT_FOUND);
  });
});

describe("Type guards edge cases", () => {
  it("isMemoryRecord rejects objects with wrong field types", () => {
    expect(isMemoryRecord({ namespace: 1, key: "k", payload: "p" })).toBe(false);
    expect(isMemoryRecord({ namespace: "ns", key: 2, payload: "p" })).toBe(false);
    expect(isMemoryRecord({ namespace: "ns", key: "k", payload: true })).toBe(false);
  });

  it("isSearchHit rejects missing record fields", () => {
    expect(isSearchHit({ distance: 0.5 })).toBe(false);
  });

  it("isNodeRecord rejects objects with wrong tier", () => {
    expect(isNodeRecord({
      id: "1",
      fields: {},
      vector_dimensions: 3,
      edges: [],
      confidence_score: 0.9,
      importance: 0.5,
      hits: 10,
      last_accessed: "1000",
      epoch: 0,
      tier: "Invalid",
      is_alive: true,
    })).toBe(false);
  });

  it("isValidValue rejects Null with extra value", () => {
    expect(isValidValue({ Null: 1 })).toBe(false);
  });

  it("isValidValue rejects empty object", () => {
    expect(isValidValue({})).toBe(false);
  });

  it("isValidValue rejects multi-key object", () => {
    expect(isValidValue({ String: "a", Int: 1 })).toBe(false);
  });

  it("isMetadata rejects non-object values", () => {
    expect(isMetadata("string")).toBe(false);
    expect(isMetadata(null)).toBe(false);
  });

  it("isValidVector rejects arrays with non-finite values", () => {
    expect(isValidVector([NaN])).toBe(false);
    expect(isValidVector([Infinity])).toBe(false);
    expect(isValidVector([-Infinity])).toBe(false);
  });
});

describe("Client input validation", () => {
  let db: Client;

  beforeAll(() => { db = Client.create(); });
  afterAll(() => { db.close(); });

  it("put with null namespace throws", () => {
    expect(() => db.put({ namespace: null as unknown as string, key: "k", payload: "v" })).toThrow();
  });

  it("put with null key throws", () => {
    expect(() => db.put({ namespace: "ns", key: null as unknown as string, payload: "v" })).toThrow();
  });

  it("put with undefined payload throws", () => {
    expect(() => db.put({ namespace: "ns", key: "k", payload: undefined as unknown as string })).toThrow();
  });

  it("put with vector containing NaN does not crash", () => {
    expect(() => db.put({ namespace: "ns", key: "k", payload: "v", vector: [1, NaN] })).not.toThrow();
  });

  it("get with empty string namespace returns null or throws", () => {
    expect(() => db.get({ namespace: "", key: "k" })).toThrow();
  });

  it("delete with empty string key throws", () => {
    expect(() => db.delete({ namespace: "ns", key: "" })).toThrow();
  });

  it("list with empty namespace throws", () => {
    expect(() => db.list({ namespace: "" })).toThrow();
  });

  it("search with empty namespace throws", () => {
    expect(() => db.search({ namespace: "", query_vector: [0.1, 0.2] })).toThrow();
  });

  it("search with empty vector throws typed validation error (D5a)", () => {
    // D5a: the TS frontier validates query_vector (non-empty, finite) like
    // `validateVector` does — an empty vector is a caller error surfaced as
    // DbError/VALIDATION_ERROR, not an engine round-trip. Previously asserted
    // `.not.toThrow()` when the frontier passed everything through unchecked.
    let caught: unknown;
    try {
      db.search({ namespace: "ns", query_vector: [] });
    } catch (err) {
      caught = err;
    }
    expect(caught).toBeInstanceOf(DbError);
    expect((caught as DbError).code).toBe(ERROR_CODES.VALIDATION_ERROR);
  });
});

describe("Client search edge cases", () => {
  let db: Client;

  beforeAll(() => {
    db = Client.create();
    db.put({ namespace: "search_edge", key: "a", payload: "alpha", vector: [1, 0, 0] });
    db.put({ namespace: "search_edge", key: "b", payload: "beta", vector: [0, 1, 0] });
  });

  afterAll(() => { db.close(); });

  it("search with Euclidean metric", () => {
    const hits = db.search({ namespace: "search_edge", query_vector: [1, 0, 0], distance_metric: "Euclidean", top_k: 5 });
    expect(hits.length).toBeGreaterThan(0);
    expect(hits[0].distance).toBeGreaterThanOrEqual(0);
  });

  it("search with top_k = 1 returns single result", () => {
    const hits = db.search({ namespace: "search_edge", query_vector: [1, 0, 0], top_k: 1 });
    expect(hits.length).toBe(1);
  });

  it("search with top_k = 0 returns empty", () => {
    const hits = db.search({ namespace: "search_edge", query_vector: [1, 0, 0], top_k: 0 });
    expect(hits).toEqual([]);
  });

  it("searchVector returns results sorted by distance", () => {
    const hits = db.searchVector({ vector: [0.5, 0.5, 0], topK: 5 });
    expect(hits.length).toBeGreaterThan(0);
    for (let i = 1; i < hits.length; i++) {
      expect(hits[i - 1].distance).toBeLessThanOrEqual(hits[i].distance);
    }
  });

  it("searchVector with top_k = 0 returns empty", () => {
    const hits = db.searchVector({ vector: [1, 0, 0], topK: 0 });
    expect(hits).toEqual([]);
  });

  it("explainSearch with explain flag", () => {
    const exp = db.explainSearch({ namespace: "search_edge", query_vector: [1, 0, 0], text_query: "alpha" });
    expect(exp).toBeDefined();
  });

  it("hybrid search with only text_query", () => {
    const hits = db.search({ namespace: "search_edge", query_vector: [1, 0, 0], text_query: "beta", top_k: 5 });
    expect(hits.length).toBeGreaterThanOrEqual(0);
  });

  it("zero-norm cosine query throws instead of falling back to Euclidean (ERR-028)", () => {
    // The core rejects zero-norm cosine queries (src/sdk/search/mod.rs) —
    // the binding must propagate that error, not silently switch metrics.
    expect(() => db.search({ namespace: "search_edge", query_vector: [0, 0, 0], top_k: 5 })).toThrow(
      /zero-norm|undefined|InvalidInput/i,
    );
  });

  it("zero-norm euclidean query is accepted", () => {
    const hits = db.search({ namespace: "search_edge", query_vector: [0, 0, 0], distance_metric: "Euclidean", top_k: 5 });
    expect(hits.length).toBeGreaterThanOrEqual(0);
  });
});

describe("Client export/import roundtrip", () => {
  let db: Client;

  beforeAll(() => {
    db = Client.create();
    db.put({ namespace: "export", key: "k1", payload: "v1", metadata: { tag: { String: "test" } } });
    db.put({ namespace: "export", key: "k2", payload: "v2" });
  });

  afterAll(() => { db.close(); });

  it("importRecords round-trip with strict counts (FIND-79)", () => {
    const report = db.importRecords([
      { namespace: "import_test", key: "a", payload: "pa", metadata: {} },
      { namespace: "import_test", key: "b", payload: "pb", metadata: {} },
    ]);
    expect(report.inserted).toBe(2);
    expect(report.updated).toBe(0);
    expect(report.errors).toBe(0);
    expect(db.get({ namespace: "import_test", key: "a" })!.payload).toBe("pa");
    expect(db.get({ namespace: "import_test", key: "b" })!.payload).toBe("pb");
  });

  it("importRecords with empty array reports zeros (FIND-79)", () => {
    expect(db.importRecords([])).toEqual({
      inserted: 0,
      updated: 0,
      skipped: 0,
      errors: 0,
      duration_ms: expect.any(Number),
    });
  });
});

describe("Client list edge cases", () => {
  let db: Client;

  beforeAll(() => {
    db = Client.create();
    for (let i = 0; i < 25; i++) {
      db.put({ namespace: "list_edge", key: `k${i}`, payload: `v${i}` });
    }
  });

  afterAll(() => { db.close(); });

  it("list with small limit paginates", () => {
    const page1 = db.list({ namespace: "list_edge", limit: 10 });
    expect(page1.records.length).toBe(10);
    expect(page1.next_cursor).toBeDefined();
    const page2 = db.list({ namespace: "list_edge", limit: 10, cursor: page1.next_cursor });
    expect(page2.records.length).toBe(10);
  });

  it("list with limit larger than dataset returns all", () => {
    const page = db.list({ namespace: "list_edge", limit: 100 });
    expect(page.records.length).toBe(25);
  });

  it("list with filters (empty filters does not error)", () => {
    expect(() => db.list({ namespace: "list_edge", filters: {} })).not.toThrow();
  });

  it("listNamespaces returns strings", () => {
    const nss = db.listNamespaces();
    expect(Array.isArray(nss)).toBe(true);
    nss.forEach((ns) => expect(typeof ns).toBe("string"));
  });
});

describe("Client lifecycle harden", () => {
  it("double close is safe", () => {
    const db = Client.create();
    db.close();
    expect(() => db.close()).not.toThrow();
  });

  it("operations after close throw typed error", () => {
    const db = Client.create();
    db.close();
    expect(() => db.capabilities()).toThrow(DbError);
    expect(() => db.listNamespaces()).toThrow(DbError);
    expect(() => db.flush()).toThrow(DbError);
    expect(() => db.compactWal()).toThrow(DbError);
  });

  it("Client.create with storage_path warns but does not throw", () => {
    const spy = vi.spyOn(console, "warn").mockImplementation(() => {});
    const db = Client.create({ storage_path: "./ignored" });
    expect(spy).toHaveBeenCalled();
    spy.mockRestore();
    db.close();
  });

  it("Client.connect(':memory:') is equivalent to connect()", () => {
    const db = Client.connect(":memory:");
    expect(db.capabilities().vector_search).toBe(true);
    db.close();
  });

  it("open() returns a non-durable handle (persistence:false per WSM-01)", () => {
    // WSM-01: `Client.open` no longer fakes `persistence:true` — only
    // `connect_persistent` (browser OPFS) claims durable persistence. In the
    // Node/wrapper build `open` attaches no durable backend, so capabilities
    // honestly reports false. The DB still opens and is usable.
    const db = Client.open("/nonexistent/test_" + Date.now());
    expect(db.capabilities().persistence).toBe(false);
    db.close();
  });
});

describe("Client graph edge cases", () => {
  let db: Client;

  beforeAll(() => { db = Client.create(); });
  afterAll(() => { db.close(); });

  it("getNode on non-existent returns null", () => {
    expect(db.getNode(999999)).toBeNull();
  });

  it("deleteNode on non-existent does not throw", () => {
    expect(() => db.deleteNode(999999, "test")).not.toThrow();
  });

  it("addEdge with non-existent nodes throws", () => {
    expect(() => db.addEdge(888, 999, "test")).toThrow(DbError);
  });

  it("graphIsDag on empty graph returns true", () => {
    expect(db.graphIsDag([1])).toBe(true);
  });

  it("graphBfs with empty roots returns empty result", () => {
    const result = db.graphBfs([], 5);
    expect(result).toBeDefined();
  });

  it("graphDfs with empty roots returns empty result", () => {
    const result = db.graphDfs([], 5);
    expect(result).toBeDefined();
  });
});

describe("Client batch edge cases", () => {
  let db: Client;

  beforeAll(() => { db = Client.create(); });
  afterAll(() => { db.close(); });

  it("putBatch with single element", () => {
    const records = db.putBatch([{ namespace: "batch1", key: "only", payload: "solo" }]);
    expect(records.length).toBe(1);
    expect(records[0].payload).toBe("solo");
  });

  it("putBatch with vectors", () => {
    const records = db.putBatch([
      { namespace: "batch_vec", key: "a", payload: "pa", vector: [0.1, 0.2] },
      { namespace: "batch_vec", key: "b", payload: "pb", vector: [0.3, 0.4] },
    ]);
    expect(records.length).toBe(2);
  });

  it("putBatch with metadata", () => {
    const records = db.putBatch([
      { namespace: "batch_meta", key: "a", payload: "pa", metadata: { x: { Int: 1 } } },
    ]);
    expect(records.length).toBe(1);
  });
});

describe("Client maintenance edge cases", () => {
  let db: Client;

  beforeAll(() => { db = Client.create(); });
  afterAll(() => { db.close(); });

  it("rebuildIndex on empty DB throws IO error on non-persistent", () => {
    expect(() => db.rebuildIndex()).toThrow(DbError);
  });

  it("compactLayout on empty DB returns >= 0", () => {
    const reclaimed = db.compactLayout();
    expect(typeof reclaimed).toBe("bigint");
  });

  it("purgeExpired on empty DB returns >= 0", () => {
    const purged = db.purgeExpired();
    expect(typeof purged).toBe("bigint");
    expect(purged >= 0n).toBe(true);
  });
});

describe("Client TTL edge cases", () => {
  let db: Client;

  beforeAll(() => { db = Client.create(); });
  afterAll(() => { db.close(); });

  it("put with TTL = null is same as no TTL", () => {
    db.put({ namespace: "ttl_null", key: "k", payload: "v", ttl_ms: null as unknown as number });
    const got = db.get({ namespace: "ttl_null", key: "k" });
    expect(got).not.toBeNull();
  });

  it("put with negative TTL throws", () => {
    expect(() => db.put({ namespace: "ttl_neg", key: "k", payload: "v", ttl_ms: -1 })).toThrow();
  });
});

describe("Client metadata edge cases", () => {
  let db: Client;

  beforeAll(() => { db = Client.create(); });
  afterAll(() => { db.close(); });

  it("put with all metadata types round-trips", () => {
    const meta = {
      s: { String: "hello" },
      i: { Int: 42 },
      f: { Float: 3.14 },
      b: { Bool: true },
      n: { Null: null },
    };
    db.put({ namespace: "meta_all", key: "k", payload: "v", metadata: meta });
    const got = db.get({ namespace: "meta_all", key: "k" });
    expect(got).not.toBeNull();
  });

  it("put with empty metadata object", () => {
    db.put({ namespace: "meta_empty", key: "k", payload: "v", metadata: {} });
    const got = db.get({ namespace: "meta_empty", key: "k" });
    expect(got).not.toBeNull();
  });
});

describe("Client generate snippet edge cases", () => {
  let db: Client;

  beforeAll(() => { db = Client.create(); });
  afterAll(() => { db.close(); });

  it("generateSnippet with empty query", () => {
    const s = db.generateSnippet("some text", "", true);
    expect(s === undefined || s.length > 0).toBe(true);
  });

  it("generateSnippet with empty payload", () => {
    const s = db.generateSnippet("", "query", true);
    expect(s === undefined || s.length === 0).toBe(true);
  });
});
