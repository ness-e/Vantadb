import { describe, it, expect } from "vitest";

import { VantaDb } from "../index.js";

/**
 * FIND-412 — `exclude_superseded` (ADR-028) was accepted on the wire but
 * silently dropped by the native binding: `parse_list_options` and
 * `parse_search_request` hardcoded `exclude_superseded: false`, so a caller
 * asking to hide the replaced record still got it back — the caller believed
 * the filter had been applied.
 *
 * Reference semantics (the flag is opt-in, default `false`):
 *   - WASM: `ListOptions.exclude_superseded` / `SearchRequest.exclude_superseded`
 *     with `#[serde(default)]` ⇒ `false` (`vantadb-wasm/src/lib.rs`).
 *   - Python: `exclude_superseded: bool = False` (`vantadb-python/src/lib.rs`).
 *
 * This exercises the real native engine (napi → Rust core), not a mock.
 */
describe("FIND-412: exclude_superseded is honored on the native path (node)", () => {
  it("hides the superseded record when true, keeps it when false/absent", async () => {
    const db = await VantaDb.connect(":memory:");
    try {
      await db.put({ namespace: "ns", key: "old", payload: "v1", vector: [1, 0] });
      await db.put({ namespace: "ns", key: "new", payload: "v2", vector: [1, 0] });
      await db.supersede("ns", "old", "new");

      // Precondition: the supersede landed. Without it the filter would be a
      // no-op and every assertion below would pass vacuously.
      const old = await db.get("ns", "old");
      expect(old).not.toBeNull();
      expect(old!.superseded_by).toBe("new");

      // list: `true` hides the replaced record.
      const listExcluded = await db.list("ns", { exclude_superseded: true });
      expect(listExcluded.records.map((r) => r.key)).toEqual(["new"]);

      // list: `false` keeps it.
      const listKept = await db.list("ns", { exclude_superseded: false });
      expect(listKept.records.map((r) => r.key).sort()).toEqual(["new", "old"]);

      // list: absent ⇒ default `false` ⇒ it is kept (same as WASM/Python).
      const listDefault = await db.list("ns", {});
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
});
