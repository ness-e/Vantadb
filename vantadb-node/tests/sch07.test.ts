import { describe, it, expect, afterAll } from "vitest";
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

import { VantaDb } from "../index.js";

/**
 * SCH-07 — v2 fields (bitemporal + quarantine) + temporal/quarantine params.
 *
 * Records carry the ADR-046 §D3/§D5 fields on every read surface (serde_json
 * wire); `as_of_ms`/`valid_window`/`include_quarantined`/`min_confidence`
 * cross `search`/`list` with the SDK wire names. Semantic boundaries
 * (`from_ms < to_ms`) stay in the core — the binding only parses shape.
 */
describe("SCH-07: v2 fields + temporal/quarantine params (node)", () => {
  const dirs: string[] = [];
  const tmp = (tag: string): string => {
    const dir = mkdtempSync(join(tmpdir(), `vantadb-node-sch07-${tag}-`));
    dirs.push(dir);
    return dir;
  };

  afterAll(() => {
    for (const dir of dirs) {
      rmSync(dir, { recursive: true, force: true });
    }
  });

  it("records expose the v2 bitemporal/quarantine fields", async () => {
    const db = await VantaDb.connect(tmp("fields"));
    try {
      await db.put({ namespace: "s7", key: "k1", payload: "p", vector: [1, 0] });

      const got = await db.get("s7", "k1");
      expect(got).not.toBeNull();
      // v1 normalization: valid_at := created_at (ADR-046 §D7).
      expect(got!.valid_at_ms).toBe(got!.created_at_ms);
      expect(got!.invalid_at_ms ?? null).toBeNull();
      expect(got!.quarantined_at_ms ?? null).toBeNull();
      expect(got!.quarantine_reason ?? null).toBeNull();
      expect(got!.quarantined_by ?? null).toBeNull();
      expect(got!.quarantine_review_due_ms ?? null).toBeNull();

      const hits = await db.search({ namespace: "s7", query_vector: [1, 0], top_k: 1 });
      expect(hits[0].record.valid_at_ms).toBe(got!.valid_at_ms);
      expect(hits[0].record.quarantined_at_ms ?? null).toBeNull();
    } finally {
      await db.close();
    }
  });

  it("search/list accept the temporal + quarantine params (shape at the boundary)", async () => {
    const db = await VantaDb.connect(tmp("params"));
    try {
      await db.put({ namespace: "s7", key: "k1", payload: "temporal probe", vector: [1, 0] });

      // as_of before creation → no hit; far future → hit.
      const none = await db.search({
        namespace: "s7",
        query_vector: [1, 0],
        top_k: 5,
        as_of_ms: 0,
      });
      expect(none).toHaveLength(0);

      const all = await db.search({
        namespace: "s7",
        query_vector: [1, 0],
        top_k: 5,
        as_of_ms: 9_000_000_000_000,
      });
      expect(all).toHaveLength(1);

      // list carries the same params (temporal + quarantine + confidence).
      const page = await db.list("s7", {
        as_of_ms: 9_000_000_000_000,
        valid_window: { from_ms: 0, to_ms: 9_000_000_000_000 },
        include_quarantined: true,
        min_confidence: 0.5,
      });
      expect(page.records).toHaveLength(1);

      // Inverted window → core boundary rejection (never silently swapped).
      await expect(
        db.list("s7", { valid_window: { from_ms: 10, to_ms: 1 } }),
      ).rejects.toThrow(/valid_window/);

      // Bad shape → napi trust-boundary rejection.
      await expect(
        db.search({
          namespace: "s7",
          query_vector: [1, 0],
          as_of_ms: "soon" as unknown as number,
        }),
      ).rejects.toThrow(/as_of_ms/);
    } finally {
      await db.close();
    }
  });
});
