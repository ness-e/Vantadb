import { describe, it, expect, beforeAll, afterAll } from "vitest";

import { Client, DbError } from "../vantadb.js";
import { NativeVantaDB } from "../native.js";
import { normalizeFilterInput, normalizeValue } from "../metadata.js";

import type { FilterInput } from "../types.js";

/**
 * WIRE-03 — TS side of the bindings-parity contract.
 *
 * The WASM backend covers text-only (`query_vector: []` + `text_query`) and
 * the canonical `$op` filter DSL (`{field: {$gte: v}}`, interchangeable with
 * Python/MCP/CLI). Sparse queries are native-only: the WASM crate is out of
 * scope and would drop `query_sparse` silently — the SDK fails loudly instead.
 * Mirrors the Python datetime-range tests and the Node sparse roundtrip.
 */

// WIRE-03: probe the native binding once for the sparse roundtrip test — the
// WASM-only environments (browser/CI without the napi build) skip it cleanly.
const nativeAvailable = await import("vantadb-node").then(
  () => true,
  () => false,
);

describe("WIRE-03: text-only + $op filters via the WASM door", () => {
  let db: Client;

  beforeAll(() => {
    db = Client.create();
  });

  afterAll(() => {
    db.close();
  });

  it("text-only search with empty query_vector returns BM25 hits", () => {
    db.put({ namespace: "w3t", key: "t1", payload: "the quick brown fox jumps", vector: [1, 0] });
    db.put({ namespace: "w3t", key: "t2", payload: "lazy dogs sleep", vector: [0, 1] });

    const hits = db.search({ namespace: "w3t", query_vector: [], text_query: "quick" });

    expect(hits.map((h) => h.record.key)).toEqual(["t1"]);
  });

  it("empty query_vector without text_query or query_sparse fails with intent", () => {
    expect(() => db.search({ namespace: "w3t", query_vector: [] })).toThrow(
      /text_query or query_sparse/,
    );
  });

  it("query_sparse on the WASM backend fails loudly (native-only)", () => {
    expect(() =>
      db.search({ namespace: "w3t", query_vector: [1, 0], query_sparse: { 1: 0.5 } }),
    ).toThrow(/not supported by the WASM backend/);
  });

  it("empty query_sparse {} is skip (core invariant), not an error", () => {
    // O1 (review WIRE-03): `{}` mirrors `hasSparse` in buildSearchRequestBase —
    // "no sparse work", so the WASM guard must not fire.
    const hits = db.search({ namespace: "w3t", query_vector: [1, 0], query_sparse: {} });
    expect(Array.isArray(hits)).toBe(true);
  });

  it("$op DSL filter (Date range) in count matches the FilterItem[] form", () => {
    db.put({ namespace: "w3f", key: "old", payload: "a", metadata: { when: new Date("2026-01-01T00:00:00Z") } });
    db.put({ namespace: "w3f", key: "mid", payload: "b", metadata: { when: new Date("2026-06-01T00:00:00Z") } });
    db.put({ namespace: "w3f", key: "new", payload: "c", metadata: { when: new Date("2026-12-01T00:00:00Z") } });

    const dsl = {
      when: { $gte: new Date("2026-06-01T00:00:00Z"), $lt: new Date("2026-12-01T00:00:00Z") },
    };
    const items = [
      { field: "when", op: "Gte" as const, value: new Date("2026-06-01T00:00:00Z") },
      { field: "when", op: "Lt" as const, value: new Date("2026-12-01T00:00:00Z") },
    ];

    // Same range, two interchangeable filter forms (py↔js parity).
    expect(db.count({ namespace: "w3f", filters: dsl })).toBe(1n);
    expect(db.count({ namespace: "w3f", filters: items })).toBe(1n);
  });

  it("flat DSL object is AND-combined and deleteByFilter accepts it", () => {
    db.put({ namespace: "w3d", key: "x", payload: "p", metadata: { tier: "hot", lang: "en" } });
    db.put({ namespace: "w3d", key: "y", payload: "p", metadata: { tier: "cold", lang: "en" } });

    expect(db.deleteByFilter({ namespace: "w3d", filter: { tier: "hot", lang: "en" } })).toBe(1n);
    expect(db.count({ namespace: "w3d" })).toBe(1n);
  });

  it("unknown $op throws VALIDATION_ERROR (never ignored)", () => {
    let caught: unknown;
    try {
      normalizeFilterInput({ tier: { $regex: "hot" } } as unknown as FilterInput);
    } catch (e) {
      caught = e;
    }
    expect(caught).toBeInstanceOf(DbError);
    expect((caught as DbError).message).toMatch(/unknown filter operator '\$regex'/);
  });

  it("normalizeValue accepts Date and tagged DateTime/ListDateTime", () => {
    expect(normalizeValue(new Date("2026-06-01T00:00:00Z"))).toEqual({
      DateTime: "2026-06-01T00:00:00.000Z",
    });
    expect(normalizeValue({ DateTime: "2026-06-01T00:00:00Z" })).toEqual({
      DateTime: "2026-06-01T00:00:00Z",
    });
    expect(normalizeValue({ ListDateTime: ["2026-01-01T00:00:00Z"] })).toEqual({
      ListDateTime: ["2026-01-01T00:00:00Z"],
    });
    expect(() => normalizeValue({ DateTime: 5 })).toThrow(DbError);
  });
});

describe.skipIf(!nativeAvailable)("WIRE-03: query_sparse via the native backend", () => {
  it("sparse-only query roundtrips put(sparse_vector) → search(query_sparse)", async () => {
    const db = await NativeVantaDB.connect(":memory:");
    try {
      await db.put({
        namespace: "w3s",
        key: "s1",
        payload: "sparse doc",
        vector: [1, 0],
        sparse_vector: { "7": 1.5, "42": 0.75 },
        // R1: `Date` metadata must normalize to the tagged DateTime wire form
        // on the native path too (normalizeMetadataForNative).
        metadata: { when: new Date("2026-06-01T00:00:00Z") },
      });

      const got = await db.get({ namespace: "w3s", key: "s1" });
      // chrono serializes DateTime without trailing zero millis.
      expect(got?.metadata).toEqual({
        when: { DateTime: "2026-06-01T00:00:00Z" },
      });

      // Sparse-only (empty dense vector, no text): with the `query_sparse`
      // forwarding reverted this returns no hits — the test is NOT vacuous
      // (mirror of the Node sparse-only test).
      const hits = await db.search({
        namespace: "w3s",
        query_vector: [],
        query_sparse: { 7: 1.5 },
      });

      expect(hits.map((h) => h.record.key)).toEqual(["s1"]);
    } finally {
      await db.close();
    }
  });
});
