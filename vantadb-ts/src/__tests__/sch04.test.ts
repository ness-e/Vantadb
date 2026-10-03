import { describe, it, expect } from "vitest";

import { Client } from "../vantadb.js";
import { _mapRecord, isMemoryRecord } from "../guards.js";
import { NativeVantaDB } from "../native.js";

/**
 * SCH-04 — confidence fields (asserted/derived) on `MemoryRecord`.
 *
 * ADR-046 §D2: every record carries `confidence_class`, `confidence`
 * (D_a = 1.0 for asserted), `last_validated_at_ms` (null = never) and
 * `derived_from` ([] for asserted). The fields travel on both backends
 * (native via serde_json, WASM via `memory_record_to_js`) — `_mapRecord`
 * passes them through untouched, and old payloads without them stay valid
 * (additive wire).
 */

// Probe the native binding once — the WASM-only environments (browser/CI
// without the napi build) skip the roundtrip cleanly (WIRE-03 pattern).
const nativeAvailable = await import("vantadb-node").then(
  () => true,
  () => false,
);

describe("SCH-04: confidence fields via the WASM backend", () => {
  it("records and search hits expose the confidence fields; filter works", () => {
    const db = Client.create();
    try {
      db.put({ namespace: "s4w", key: "k1", payload: "p", vector: [1, 0] });

      const got = db.get({ namespace: "s4w", key: "k1" });
      expect(got).not.toBeNull();
      expect(got!.confidence_class).toBe("Asserted");
      expect(got!.confidence).toBe(1);
      expect(got!.derived_from).toEqual([]);
      // WASM omits `last_validated_at_ms` when never validated.
      expect(got!.last_validated_at_ms ?? null).toBeNull();

      const hits = db.search({
        namespace: "s4w",
        query_vector: [1, 0],
        top_k: 1,
        min_confidence: 0.95,
      });
      expect(hits).toHaveLength(1);
      expect(hits[0].record.confidence_class).toBe("Asserted");

      // Out-of-range threshold is rejected by the core boundary.
      expect(() =>
        db.search({ namespace: "s4w", query_vector: [1, 0], min_confidence: 2 }),
      ).toThrow(/min_confidence/);
    } finally {
      db.close();
    }
  });
});

describe("SCH-04: confidence fields on MemoryRecord (shared mapping)", () => {
  it("_mapRecord preserves derived confidence fields verbatim", () => {
    const raw = {
      namespace: "ns",
      key: "k",
      payload: "p",
      metadata: {},
      created_at_ms: "1000",
      updated_at_ms: "2000",
      version: "1",
      node_id: "42",
      confidence_class: "Derived",
      confidence: 0.45,
      last_validated_at_ms: "1700000000000",
      derived_from: ["parent-1", "parent-2"],
    };

    const rec = _mapRecord(raw);
    expect(rec.confidence_class).toBe("Derived");
    expect(rec.confidence).toBe(0.45);
    expect(rec.last_validated_at_ms).toBe("1700000000000");
    expect(rec.derived_from).toEqual(["parent-1", "parent-2"]);
  });

  it("isMemoryRecord stays additive — records without confidence fields are still valid", () => {
    // Old backend payloads (pre-0.8.0 addon/wasm pkg) omit the new fields;
    // the guard must not start rejecting them (no breaking).
    const legacy = {
      namespace: "ns",
      key: "k",
      payload: "p",
      metadata: {},
      created_at_ms: "1000",
      updated_at_ms: "2000",
      version: "1",
      node_id: "42",
    };
    expect(isMemoryRecord(legacy)).toBe(true);
    expect(_mapRecord(legacy).confidence).toBeUndefined();
  });
});

describe.skipIf(!nativeAvailable)("SCH-04: confidence via the native backend", () => {
  it("get after put carries the D_a defaults (1.0 / Asserted / null / [])", async () => {
    const db = await NativeVantaDB.connect(":memory:");
    try {
      await db.put({ namespace: "s4n", key: "k1", payload: "p", vector: [1, 0] });

      const got = await db.get({ namespace: "s4n", key: "k1" });
      expect(got).not.toBeNull();
      expect(got!.confidence_class).toBe("Asserted");
      expect(got!.confidence).toBe(1);
      expect(got!.last_validated_at_ms).toBeNull();
      expect(got!.derived_from).toEqual([]);
    } finally {
      await db.close();
    }
  });

  it("min_confidence is forwarded to the engine and validated there", async () => {
    const db = await NativeVantaDB.connect(":memory:");
    try {
      await db.put({ namespace: "s4n", key: "k1", payload: "p", vector: [1, 0] });

      const hits = await db.search({
        namespace: "s4n",
        query_vector: [1, 0],
        top_k: 5,
        min_confidence: 0.95,
      });
      expect(hits).toHaveLength(1);
      expect(hits[0].record.key).toBe("k1");

      await expect(
        db.search({ namespace: "s4n", query_vector: [1, 0], min_confidence: 2 }),
      ).rejects.toThrow(/min_confidence/);
    } finally {
      await db.close();
    }
  });
});
