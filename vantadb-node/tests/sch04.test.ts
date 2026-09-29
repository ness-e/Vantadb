import { describe, it, expect, afterAll } from "vitest";
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

import { VantaDb } from "../index.js";

/**
 * SCH-04 — confidence fields + opt-in `min_confidence` filter (node binding).
 *
 * Records carry the ADR-046 §D2 fields on every read surface (serde_json wire);
 * `min_confidence` is an opt-in search filter validated at the core boundary.
 * Filter *discrimination* (derived 0.9 vs asserted 1.0) is pinned by the core
 * and Python/HTTP tests — node has no writer path for `derived` records.
 */
describe("SCH-04: confidence fields + min_confidence (node)", () => {
  const dirs: string[] = [];
  const tmp = (tag: string): string => {
    const dir = mkdtempSync(join(tmpdir(), `vantadb-node-sch04-${tag}-`));
    dirs.push(dir);
    return dir;
  };

  afterAll(() => {
    for (const dir of dirs) {
      rmSync(dir, { recursive: true, force: true });
    }
  });

  it("get exposes the D_a confidence defaults", async () => {
    const db = await VantaDb.connect(tmp("fields"));
    try {
      await db.put({ namespace: "s4", key: "k1", payload: "p", vector: [1, 0] });

      const got = await db.get("s4", "k1");
      expect(got).not.toBeNull();
      expect(got!.confidence_class).toBe("Asserted");
      expect(got!.confidence).toBe(1);
      expect(got!.last_validated_at_ms).toBeNull();
      expect(got!.derived_from).toEqual([]);
    } finally {
      await db.close();
    }
  });

  it("search hits expose the record confidence fields", async () => {
    const db = await VantaDb.connect(tmp("hits"));
    try {
      await db.put({ namespace: "s4", key: "k1", payload: "p", vector: [1, 0] });

      const hits = await db.search({ namespace: "s4", query_vector: [1, 0], top_k: 1 });
      expect(hits).toHaveLength(1);
      expect(hits[0].record.confidence_class).toBe("Asserted");
      expect(hits[0].record.confidence).toBe(1);
    } finally {
      await db.close();
    }
  });

  it("min_confidence accepts a valid threshold (asserted D_a = 1.0 passes)", async () => {
    const db = await VantaDb.connect(tmp("filter-ok"));
    try {
      await db.put({ namespace: "s4", key: "k1", payload: "p", vector: [1, 0] });

      const hits = await db.search({
        namespace: "s4",
        query_vector: [1, 0],
        top_k: 5,
        min_confidence: 0.95,
      });
      expect(hits).toHaveLength(1);
      expect(hits[0].record.key).toBe("k1");
    } finally {
      await db.close();
    }
  });

  it("min_confidence rejects out-of-range thresholds at the boundary", async () => {
    const db = await VantaDb.connect(tmp("filter-bad"));
    try {
      await db.put({ namespace: "s4", key: "k1", payload: "p", vector: [1, 0] });

      await expect(
        db.search({ namespace: "s4", query_vector: [1, 0], min_confidence: 1.5 }),
      ).rejects.toThrow(/min_confidence/);
    } finally {
      await db.close();
    }
  });
});
