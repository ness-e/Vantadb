import { describe, it, expect, afterAll } from "vitest";
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

import { VantaDb } from "../index.js";

/**
 * WIRE-03 — Node side of the bindings-parity contract:
 * - `query_sparse` roundtrips put(sparse_vector) → search(query_sparse);
 * - text-only (`query_vector: []` + `text_query`) by the main search door;
 * - advanced filters (range / DateTime) via the canonical `FilterItem[]` wire,
 *   mirroring the Python `$gte`/`$lt` + `datetime` test.
 *
 * Before this task the binding silently dropped both `query_sparse` and
 * `sparse_vector` (serde parsing never read them) — these tests pin that the
 * values now reach the engine.
 */
describe("WIRE-03: query_sparse + text-only + advanced filters (node)", () => {
  const dirs: string[] = [];
  const tmp = (tag: string): string => {
    const dir = mkdtempSync(join(tmpdir(), `vantadb-node-wire03-${tag}-`));
    dirs.push(dir);
    return dir;
  };

  afterAll(() => {
    for (const dir of dirs) {
      rmSync(dir, { recursive: true, force: true });
    }
  });

  it("sparse query roundtrips put(sparse_vector) → search(query_sparse)", async () => {
    const db = await VantaDb.connect(tmp("sparse"));
    try {
      await db.put({
        namespace: "ns",
        key: "s1",
        payload: "sparse doc",
        vector: [1, 0],
        sparse_vector: { "7": 1.5, "42": 0.75 },
      });

      const hits = await db.search({
        namespace: "ns",
        query_vector: [1, 0],
        query_sparse: { "7": 1.5 },
        top_k: 5,
      });

      expect(hits.map((h) => h.record.key)).toEqual(["s1"]);
    } finally {
      await db.close();
    }
  });

  it("sparse-only search (empty vector, no text) matches postings", async () => {
    const db = await VantaDb.connect(tmp("sparse-only"));
    try {
      await db.put({ namespace: "ns", key: "a", payload: "alpha", sparse_vector: { "1": 1.0 } });
      await db.put({ namespace: "ns", key: "b", payload: "beta", sparse_vector: { "2": 1.0 } });

      const hits = await db.search({
        namespace: "ns",
        query_vector: [],
        query_sparse: { "2": 1.0 },
        top_k: 5,
      });

      expect(hits.map((h) => h.record.key)).toEqual(["b"]);
    } finally {
      await db.close();
    }
  });

  it("text-only search with empty query_vector returns BM25 hits", async () => {
    const db = await VantaDb.connect(tmp("text-only"));
    try {
      await db.put({ namespace: "ns", key: "t1", payload: "the quick brown fox jumps", vector: [1, 0] });
      await db.put({ namespace: "ns", key: "t2", payload: "lazy dogs sleep", vector: [0, 1] });

      const hits = await db.search({
        namespace: "ns",
        query_vector: [],
        text_query: "quick",
        top_k: 5,
      });

      expect(hits.map((h) => h.record.key)).toEqual(["t1"]);
    } finally {
      await db.close();
    }
  });

  it("rejects a non-u32 sparse dimension key instead of dropping it", async () => {
    const db = await VantaDb.connect(tmp("bad-key"));
    try {
      await expect(
        db.put({
          namespace: "ns",
          key: "bad",
          payload: "p",
          // @ts-expect-error — exercising the runtime guard behind the type
          sparse_vector: { x: 1.0 },
        }),
      ).rejects.toThrow(/u32/);
    } finally {
      await db.close();
    }
  });

  it("count/deleteByFilter support range over DateTime values (py↔js mirror)", async () => {
    const db = await VantaDb.connect(tmp("filters"));
    try {
      await db.put({
        namespace: "ns",
        key: "old",
        payload: "a",
        metadata: { when: { DateTime: "2026-01-01T00:00:00Z" } },
      });
      await db.put({
        namespace: "ns",
        key: "mid",
        payload: "b",
        metadata: { when: { DateTime: "2026-06-01T00:00:00Z" } },
      });
      await db.put({
        namespace: "ns",
        key: "new",
        payload: "c",
        metadata: { when: { DateTime: "2026-12-01T00:00:00Z" } },
      });

      const range = [
        { field: "when", op: "Gte" as const, value: { DateTime: "2026-06-01T00:00:00Z" } },
        { field: "when", op: "Lt" as const, value: { DateTime: "2026-12-01T00:00:00Z" } },
      ];
      await expect(db.count("ns", range)).resolves.toBe(1n);
      await expect(db.deleteByFilter("ns", range)).resolves.toBe(1n);
      await expect(db.count("ns")).resolves.toBe(2n);
    } finally {
      await db.close();
    }
  });
});
