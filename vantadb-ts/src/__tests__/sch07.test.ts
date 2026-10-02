import { describe, it, expect } from "vitest";

import { Client } from "../vantadb.js";
import { _mapRecord, isMemoryRecord } from "../guards.js";
import { NativeVantaDB } from "../native.js";

/**
 * SCH-07 — v2 fields (bitemporal + quarantine) + temporal/quarantine params.
 *
 * ADR-046 §D3/§D5: every record carries `valid_at_ms`/`invalid_at_ms` (valid
 * time) and the 4 quarantine fields; `as_of_ms`/`valid_window`/
 * `include_quarantined` cross `search`/`list` on both backends (native via
 * napi, WASM via the `SearchRequest`/`ListOptions` structs). The wire stays
 * additive — old payloads without the new fields remain valid.
 */

// Probe the native binding once — WASM-only environments (browser/CI without
// the napi build) skip the native roundtrip cleanly (WIRE-03 pattern).
const nativeAvailable = await import("vantadb-node").then(
  () => true,
  () => false,
);

describe("SCH-07: v2 fields + temporal params via the WASM backend", () => {
  it("records expose the v2 fields; params filter through", () => {
    const db = Client.create();
    try {
      db.put({ namespace: "s7w", key: "k1", payload: "temporal probe", vector: [1, 0] });

      const got = db.get({ namespace: "s7w", key: "k1" });
      expect(got).not.toBeNull();
      // WASM emits u64s as decimal strings; v1 normalize: valid := created.
      expect(got!.valid_at_ms).toBe(got!.created_at_ms);
      expect(got!.invalid_at_ms ?? null).toBeNull();
      expect(got!.quarantined_at_ms ?? null).toBeNull();
      expect(got!.quarantine_reason ?? null).toBeNull();
      expect(got!.quarantined_by ?? null).toBeNull();
      expect(got!.quarantine_review_due_ms ?? null).toBeNull();

      // as_of before creation → no hit; far future → hit.
      const none = db.search({ namespace: "s7w", query_vector: [1, 0], top_k: 5, as_of_ms: 0 });
      expect(none).toHaveLength(0);
      const all = db.search({
        namespace: "s7w",
        query_vector: [1, 0],
        top_k: 5,
        as_of_ms: 9_000_000_000_000,
      });
      expect(all).toHaveLength(1);

      // `list` carries the same temporal/quarantine/confidence params.
      const page = db.list({
        namespace: "s7w",
        as_of_ms: 9_000_000_000_000,
        valid_window: { from_ms: 0, to_ms: 9_000_000_000_000 },
        include_quarantined: true,
        min_confidence: 0.5,
      });
      expect(page.records).toHaveLength(1);

      // Discrimination guard: `as_of_ms: 0` must return 0 records — a dropped
      // param would still return 1 and the assertion above would pass.
      const noneList = db.list({ namespace: "s7w", as_of_ms: 0 });
      expect(noneList.records).toHaveLength(0);

      // Inverted window → core boundary rejection (never silently swapped).
      expect(() =>
        db.list({ namespace: "s7w", valid_window: { from_ms: 10, to_ms: 1 } }),
      ).toThrow(/valid_window/);
    } finally {
      db.close();
    }
  });
});

describe("SCH-07: _mapRecord preserves the v2 fields verbatim", () => {
  const base = {
    namespace: "ns",
    key: "k",
    payload: "p",
    metadata: {},
    created_at_ms: "1000",
    updated_at_ms: "2000",
    version: "1",
    node_id: "42",
  };

  it("bitemporal + quarantine fields survive mapping", () => {
    const rec = _mapRecord({
      ...base,
      valid_at_ms: "1000",
      invalid_at_ms: "2000",
      quarantined_at_ms: "1500",
      quarantine_reason: "unreviewed_import",
      quarantined_by: "system:import",
      quarantine_review_due_ms: "2592000000",
    });
    expect(rec.valid_at_ms).toBe("1000");
    expect(rec.invalid_at_ms).toBe("2000");
    expect(rec.quarantined_at_ms).toBe("1500");
    expect(rec.quarantine_reason).toBe("unreviewed_import");
    expect(rec.quarantined_by).toBe("system:import");
    expect(rec.quarantine_review_due_ms).toBe("2592000000");
  });

  it("isMemoryRecord stays additive — payloads without v2 fields are valid", () => {
    expect(isMemoryRecord(base)).toBe(true);
    expect(_mapRecord(base).valid_at_ms).toBeUndefined();
  });
});

describe.skipIf(!nativeAvailable)("SCH-07: v2 fields + temporal params via the native backend", () => {
  it("records expose the fields and list forwards the params through napi", async () => {
    const db = await NativeVantaDB.connect(":memory:");
    try {
      await db.put({ namespace: "s7n", key: "k1", payload: "temporal probe", vector: [1, 0] });

      const got = await db.get({ namespace: "s7n", key: "k1" });
      expect(got).not.toBeNull();
      // Native emits u64s as numbers.
      expect(typeof got!.valid_at_ms).toBe("number");
      expect(got!.valid_at_ms).toBe(got!.created_at_ms);
      expect(got!.quarantined_at_ms ?? null).toBeNull();

      const page = await db.list({ namespace: "s7n", as_of_ms: 9_000_000_000_000 });
      expect(page.records).toHaveLength(1);
      const filtered = await db.list({ namespace: "s7n", as_of_ms: 0 });
      expect(filtered.records).toHaveLength(0);

      await expect(
        db.list({ namespace: "s7n", valid_window: { from_ms: 10, to_ms: 1 } }),
      ).rejects.toThrow(/valid_window/);
    } finally {
      await db.close();
    }
  });
});
