import { describe, it, expect } from "vitest";

import { Client } from "../vantadb.js";
import { NativeVantaDB } from "../native.js";

/**
 * FIND-412 — `exclude_superseded` (ADR-028) was honored on the WASM backend
 * but silently dropped on the native one: the TS native wrapper never put the
 * flag on the wire and the napi binding hardcoded `exclude_superseded: false`.
 * The caller believed the filter had been applied while the superseded record
 * was still returned.
 *
 * Reference semantics (opt-in, default `false`):
 *   - WASM: `ListOptions.exclude_superseded` / `SearchRequest.exclude_superseded`
 *     with `#[serde(default)]` ⇒ `false` (`vantadb-wasm/src/lib.rs`).
 *   - Python: `exclude_superseded: bool = False` (`vantadb-python/src/lib.rs`).
 *
 * The native block below runs against the real napi engine — not a mock.
 */

// Probe the native binding once — WASM-only environments (browser/CI without
// the napi build) skip the native roundtrip cleanly (WIRE-03 pattern).
const nativeAvailable = await import("vantadb-node").then(
  () => true,
  () => false,
);

describe("FIND-412: exclude_superseded — WASM backend (reference semantics)", () => {
  it("hides the superseded record when true, keeps it when false/absent", () => {
    const db = Client.create();
    try {
      db.put({ namespace: "ns", key: "old", payload: "v1", vector: [1, 0] });
      db.put({ namespace: "ns", key: "new", payload: "v2", vector: [1, 0] });
      db.supersede({ namespace: "ns", oldKey: "old", newKey: "new" });

      const old = db.get({ namespace: "ns", key: "old" });
      // The WASM wire does not expose `superseded_by` (the native one does),
      // so the discriminator here is the flag flipping the result set below:
      // if the supersede had not landed, `exclude_superseded: true` would
      // still return "old" and the assertion would fail.
      expect(old).not.toBeNull();

      const searchExcluded = db.search({
        namespace: "ns",
        query_vector: [1, 0],
        top_k: 5,
        exclude_superseded: true,
      });
      expect(searchExcluded.map((h) => h.record.key)).toEqual(["new"]);

      const searchKept = db.search({
        namespace: "ns",
        query_vector: [1, 0],
        top_k: 5,
        exclude_superseded: false,
      });
      expect(searchKept.map((h) => h.record.key).sort()).toEqual(["new", "old"]);

      const searchDefault = db.search({ namespace: "ns", query_vector: [1, 0], top_k: 5 });
      expect(searchDefault.map((h) => h.record.key).sort()).toEqual(["new", "old"]);

      const listExcluded = db.list({ namespace: "ns", exclude_superseded: true });
      expect(listExcluded.records.map((r) => r.key)).toEqual(["new"]);

      const listDefault = db.list({ namespace: "ns" });
      expect(listDefault.records.map((r) => r.key).sort()).toEqual(["new", "old"]);
    } finally {
      db.close();
    }
  });
});

describe.skipIf(!nativeAvailable)(
  "FIND-412: exclude_superseded — native backend (TS wrapper over napi)",
  () => {
    it("hides the superseded record when true, keeps it when false/absent", async () => {
      const db = await NativeVantaDB.connect(":memory:");
      try {
        await db.put({ namespace: "ns", key: "old", payload: "v1", vector: [1, 0] });
        await db.put({ namespace: "ns", key: "new", payload: "v2", vector: [1, 0] });
        await db.supersede("ns", "old", "new");

        // Precondition: the supersede landed (else the assertions below pass
        // vacuously).
        const old = await db.get({ namespace: "ns", key: "old" });
        expect(old?.superseded_by).toBe("new");

        // list: `true` hides the replaced record.
        const listExcluded = await db.list({ namespace: "ns", exclude_superseded: true });
        expect(listExcluded.records.map((r) => r.key)).toEqual(["new"]);

        // list: `false` keeps it.
        const listKept = await db.list({ namespace: "ns", exclude_superseded: false });
        expect(listKept.records.map((r) => r.key).sort()).toEqual(["new", "old"]);

        // list: absent ⇒ default `false` ⇒ it is kept (same as WASM/Python).
        const listDefault = await db.list({ namespace: "ns" });
        expect(listDefault.records.map((r) => r.key).sort()).toEqual(["new", "old"]);

        // search: `true` hides the replaced record.
        const searchExcluded = await db.search({
          namespace: "ns",
          query_vector: [1, 0],
          top_k: 5,
          exclude_superseded: true,
        });
        expect(searchExcluded.map((h) => h.record.key)).toEqual(["new"]);

        // search: `false` keeps it.
        const searchKept = await db.search({
          namespace: "ns",
          query_vector: [1, 0],
          top_k: 5,
          exclude_superseded: false,
        });
        expect(searchKept.map((h) => h.record.key).sort()).toEqual(["new", "old"]);

        // search: absent ⇒ default `false` ⇒ it is kept (same as WASM/Python).
        const searchDefault = await db.search({
          namespace: "ns",
          query_vector: [1, 0],
          top_k: 5,
        });
        expect(searchDefault.map((h) => h.record.key).sort()).toEqual(["new", "old"]);
      } finally {
        await db.close();
      }
    });
  },
);
