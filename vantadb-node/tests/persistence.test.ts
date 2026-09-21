import { describe, it, expect, afterAll } from "vitest";
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

import { VantaDb } from "../index.js";

/**
 * Persistence differential test — the core advantage of the native napi-rs
 * backend over WASM: real filesystem persistence (fjall/WAL/fsync).
 *
 * (1) connect to a temp dir → put → get returns the value;
 * (2) close → reconnect to the SAME path → data persists;
 * (3) search returns hits ordered by score (closest first).
 */
describe("vantadb-node persistence (native backend)", () => {
  const tempDirs: string[] = [];

  afterAll(() => {
    for (const dir of tempDirs) {
      rmSync(dir, { recursive: true, force: true });
    }
  });

  it("put → get returns the value, search ordered by score", async () => {
    const dir = mkdtempSync(join(tmpdir(), "vantadb-node-"));
    tempDirs.push(dir);

    const db = await VantaDb.connect(dir);
    try {
      const stored = await db.put({
        namespace: "docs",
        key: "hello",
        payload: "hello world",
        vector: [0.1, 0.2, 0.3],
        metadata: { source: { String: "test" } },
      });
      expect(stored.key).toBe("hello");
      expect(stored.payload).toBe("hello world");

      const got = await db.get("docs", "hello");
      expect(got).not.toBeNull();
      expect(got!.payload).toBe("hello world");
    } finally {
      await db.close();
    }
  });

  it("data persists across close + reconnect (WAL/fsync — not possible in WASM)", async () => {
    const dir = mkdtempSync(join(tmpdir(), "vantadb-node-persist-"));
    tempDirs.push(dir);

    const db1 = await VantaDb.connect(dir);
    await db1.put({
      namespace: "persist",
      key: "k1",
      payload: "survives",
      vector: [1.0, 0.0, 0.0],
    });
    await db1.flush();
    await db1.close();

    const db2 = await VantaDb.connect(dir);
    try {
      const got = await db2.get("persist", "k1");
      expect(got).not.toBeNull();
      expect(got!.payload).toBe("survives");
    } finally {
      await db2.close();
    }
  });

  it("search returns hits ordered by score (closest first)", async () => {
    const dir = mkdtempSync(join(tmpdir(), "vantadb-node-search-"));
    tempDirs.push(dir);

    const db = await VantaDb.connect(dir);
    try {
      await db.putBatch([
        { namespace: "vec", key: "far", payload: "far", vector: [1.0, 0.0, 0.0] },
        { namespace: "vec", key: "near", payload: "near", vector: [0.5, 0.5, 0.0] },
        { namespace: "vec", key: "exact", payload: "exact", vector: [0.0, 1.0, 0.0] },
      ]);

      const hits = await db.search({
        namespace: "vec",
        query_vector: [0.0, 1.0, 0.0],
        top_k: 3,
      });

      expect(hits.length).toBe(3);
      expect(hits[0].record.key).toBe("exact");
      expect(hits[1].record.key).toBe("near");
      expect(hits[2].record.key).toBe("far");

      // Scores are cosine similarity; the closest hit has the highest score.
      expect(hits[0].score).toBeGreaterThan(hits[1].score);
      expect(hits[1].score).toBeGreaterThan(hits[2].score);
    } finally {
      await db.close();
    }
  });
});
