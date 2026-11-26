import { describe, it, expect } from "vitest";
import { NativeVantaDB } from "../native.js";
import { DbError, ERROR_CODES, mapCodeToCanonical } from "../errors.js";

/**
 * TS-02 regression tests: every failure raised by the native binding —
 * synchronous throws AND async promise rejections — must surface to callers
 * as a wrapped `DbError`, never as a raw binding error.
 *
 * We bypass `NativeVantaDB.connect()` (needs a built native binary) and
 * inject a fake inner whose methods reject asynchronously.
 */
function makeDbWith(inner: Record<string, unknown>): NativeVantaDB {
  // The constructor is private but accessible at runtime; this is deliberate
  // so the wrapping contract can be tested without a compiled .node binary.
  return new (NativeVantaDB as unknown as new (i: unknown) => NativeVantaDB)(inner);
}

describe("NativeVantaDB error wrapping (TS-02)", () => {
  it("recovers the VANTADB_* code from the napi '{code}: {message}' prefix (ERR-TS-01)", async () => {
    // vantadb-node's map_err emits e.g. "VANTADB_NOT_FOUND: Node not found: 7".
    const db = makeDbWith({
      get: () => Promise.reject(new Error("VANTADB_NOT_FOUND: Node not found: 7")),
    });
    await expect(db.get({ namespace: "ns", key: "k" })).rejects.toMatchObject({
      code: "VANTADB_NOT_FOUND",
      message: expect.stringContaining("Node not found: 7"),
    });
    // The prefix must not leak into the human message twice.
    await expect(db.get({ namespace: "ns", key: "k" })).rejects.toMatchObject({
      message: expect.not.stringMatching(/^get: VANTADB_/),
    });
  });

  it("classifies an unprefixed native error via message heuristics, never NATIVE_ERROR (ERR-TS-01)", async () => {
    const db = makeDbWith({
      get: () => Promise.reject(new Error("engine panicked on background thread")),
    });
    await expect(db.get({ namespace: "ns", key: "k" })).rejects.toBeInstanceOf(DbError);
    await expect(db.get({ namespace: "ns", key: "k" })).rejects.toMatchObject({
      code: "VANTADB_WASM_ERROR",
      message: expect.stringContaining("get: engine panicked on background thread"),
    });
  });

  it("wraps an ASYNC rejection from the inner binding in a DbError with code", async () => {
    const db = makeDbWith({
      get: () => Promise.reject(new Error("engine panicked on background thread")),
    });
    await expect(db.get({ namespace: "ns", key: "k" })).rejects.toBeInstanceOf(DbError);
    await expect(db.get({ namespace: "ns", key: "k" })).rejects.toMatchObject({
      code: "VANTADB_WASM_ERROR",
      message: expect.stringContaining("get: engine panicked on background thread"),
    });
  });

  it("wraps a SYNCHRONOUS throw from the inner binding in a DbError", async () => {
    const db = makeDbWith({
      flush: () => {
        throw new Error("sync boom");
      },
    });
    await expect(db.flush()).rejects.toBeInstanceOf(DbError);
    await expect(db.flush()).rejects.toMatchObject({ code: "VANTADB_WASM_ERROR" });
  });

  it("passes an existing DbError through untouched", async () => {
    const original = new DbError(ERROR_CODES.BUSY, "database busy");
    const db = makeDbWith({ delete: () => Promise.reject(original) });
    let caught: unknown;
    try {
      await db.delete({ namespace: "ns", key: "k" });
    } catch (e) {
      caught = e;
    }
    expect(caught).toBe(original);
  });

  it("classifies serialization error as VANTADB_SERIALIZATION and NEVER as VANTADB_CORRUPT (FFI-02)", async () => {
    // 1. Structured prefix
    const db1 = makeDbWith({
      get: () =>
        Promise.reject(
          new Error(
            "[ERR_CODE:ERR_SERIALIZATION] VANTADB_SERIALIZATION: serialization error: failed",
          ),
        ),
    });
    await expect(db1.get({ namespace: "ns", key: "k" })).rejects.toMatchObject({
      code: "VANTADB_SERIALIZATION",
      message: expect.stringContaining("serialization error: failed"),
    });

    // 2. Unprefixed message heuristic fallback must NEVER map to CORRUPT
    const db2 = makeDbWith({
      get: () => Promise.reject(new Error("serialization error: cannot decode bincode")),
    });
    await expect(db2.get({ namespace: "ns", key: "k" })).rejects.toMatchObject({
      code: "VANTADB_SERIALIZATION",
      message: expect.stringContaining("serialization error: cannot decode bincode"),
    });
  });

  it("classifies quota exceeded as VANTADB_QUOTA_EXCEEDED (FFI-02)", async () => {
    const db = makeDbWith({
      get: () =>
        Promise.reject(
          new Error("[ERR_CODE:ERR_QUOTA_EXCEEDED] VANTADB_QUOTA_EXCEEDED: tenant quota exceeded"),
        ),
    });
    await expect(db.get({ namespace: "ns", key: "k" })).rejects.toMatchObject({
      code: "VANTADB_QUOTA_EXCEEDED",
      message: expect.stringContaining("tenant quota exceeded"),
    });
  });
});

describe("NativeVantaDB honest surfaces (FIND-536/537/539)", () => {
  it("metricsPrometheus rejects with VANTADB_NOT_SUPPORTED — no metrics primitive in vantadb-node", async () => {
    const db = makeDbWith({});
    await expect(db.metricsPrometheus()).rejects.toBeInstanceOf(DbError);
    await expect(db.metricsPrometheus()).rejects.toMatchObject({
      code: "VANTADB_NOT_SUPPORTED",
    });
  });

  it("findNodesByLabel rejects with VANTADB_NOT_SUPPORTED instead of a fabricated []", async () => {
    const db = makeDbWith({});
    await expect(db.findNodesByLabel("knows")).rejects.toMatchObject({
      code: "VANTADB_NOT_SUPPORTED",
    });
  });

  it("graphShortestPath rejects with VANTADB_NOT_SUPPORTED instead of a fabricated 2-hop path", async () => {
    const db = makeDbWith({});
    await expect(db.graphShortestPath(1, 2)).rejects.toMatchObject({
      code: "VANTADB_NOT_SUPPORTED",
    });
  });

  it("getEdges returns the real edges from getNode ([] when the node is absent)", async () => {
    const edges = [
      { target: "2", label: "knows", weight: 0.8, reverse: false, created_at_ms: 1000 },
    ];
    const db = makeDbWith({ getNode: () => Promise.resolve({ id: "1", edges }) });
    expect(await db.getEdges(1)).toEqual(edges);
    const empty = makeDbWith({ getNode: () => Promise.resolve(null) });
    expect(await empty.getEdges(1)).toEqual([]);
  });

  it("query delegates to the native IQL primitive (FIND-268)", async () => {
    const result = { Read: [{ id: "1", edges: [] }] };
    let seenIql: unknown;
    const db = makeDbWith({
      query: (iql: string) => {
        seenIql = iql;
        return Promise.resolve(result);
      },
    });
    expect(await db.query("SELECT * FROM Person")).toBe(result);
    expect(seenIql).toBe("SELECT * FROM Person");
  });

  it("query rejects non-string iql with VANTADB_INVALID_ARGUMENT before hitting the engine", async () => {
    const db = makeDbWith({ query: () => Promise.resolve({}) });
    await expect(db.query(5 as unknown as string)).rejects.toMatchObject({
      code: "VANTADB_INVALID_ARGUMENT",
    });
  });

  it("query wraps engine rejections in a DbError", async () => {
    const db = makeDbWith({
      query: () => Promise.reject(new Error("VANTADB_VALIDATION_ERROR: iql parse error")),
    });
    await expect(db.query("(bad")).rejects.toMatchObject({
      code: "VANTADB_VALIDATION_ERROR",
    });
  });

  it("graphragSearch forwards abort handle to the native binding", async () => {
    const fakeAbort = { abort: () => {} } as unknown as import("vantadb-node").AbortHandle;
    const expected = {
      nodes: [],
      edges: [],
      context_text: "",
      stats: { seeds_found: 0, nodes_expanded: 0, total_candidates: 0, expansion_hops_used: 0 },
    };
    let seenArgs: unknown[] = [];
    const db = makeDbWith({
      graphragSearch: (...args: unknown[]) => {
        seenArgs = args;
        return Promise.resolve(expected);
      },
    });
    const out = await db.graphragSearch("ns", "hello", [0.1, 0.2], fakeAbort);
    expect(out).toBe(expected);
    expect(seenArgs).toEqual(["ns", "hello", [0.1, 0.2], fakeAbort]);
  });
});

describe("mapCodeToCanonical keeps the core wire codes (ERROR_HANDLING.md §1.1)", () => {
  // Every code `Error::code()` emits in the Rust core must reach TS callers
  // unchanged; FFI-02's coarse numeric aliases must never rewrite them.
  it.each([
    "VANTADB_VALIDATION_ERROR",
    "VANTADB_NOT_FOUND",
    "VANTADB_TIMEOUT",
    "VANTADB_BUSY",
    "VANTADB_RESOURCE_LIMIT",
    "VANTADB_CORRUPT",
    "VANTADB_INVALID_ARGUMENT",
    "VANTADB_IO_ERROR",
    "VANTADB_WASM_ERROR",
    "VANTADB_CLOSED",
  ])("%s maps to itself", (code) => {
    expect(mapCodeToCanonical(code)).toBe(code);
  });
});
