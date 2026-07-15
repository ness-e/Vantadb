import { describe, it, expect, beforeAll, afterAll } from "vitest";
import { VantaDB } from "../vantadb.js";

describe("VantaDB Load Tests", () => {
  let db: VantaDB;

  beforeAll(() => {
    db = VantaDB.create();
  });

  afterAll(() => {
    db.close();
  });

  it("should handle concurrent put operations", async () => {
    const promises: Promise<any>[] = [];
    for (let i = 0; i < 1000; i++) {
      promises.push(
        db.put({
          namespace: "concurrent",
          key: `k${i}`,
          payload: `v${i}`,
          vector: [i % 10, 0, 0, 0],
        })
      );
    }
    await Promise.all(promises);

    const hits = await db.search({
      namespace: "concurrent",
      query_vector: [0, 0, 0, 0],
      top_k: 10,
    });
    expect(hits.length).toBeGreaterThan(0);
  });

  it("should handle large batch inserts (5000 vectors)", { timeout: 30000 }, async () => {
    const inputs: any[] = [];
    for (let i = 0; i < 5000; i++) {
      inputs.push({
        namespace: "large_batch",
        key: `bk${i}`,
        payload: `batch_${i}`,
        vector: [(i % 256) / 256, 0.5, 0.3, 0.1],
      });
    }
    const records = await db.putBatch(inputs);
    expect(records.length).toBe(5000);

    const hits = await db.search({
      namespace: "large_batch",
      query_vector: [0.5, 0.5, 0.3, 0.1],
      top_k: 10,
    });
    expect(hits.length).toBe(10);
  });

  it("should not error on repeated create/destroy cycles", async () => {
    for (let i = 0; i < 50; i++) {
      const tmp = VantaDB.create();
      await tmp.put({
        namespace: `cycle_${i}`,
        key: "k",
        payload: `cycle_${i}`,
        vector: [i % 10, 0, 0, 0],
      });
      const got = await tmp.get(`cycle_${i}`, "k");
      expect(got).not.toBeNull();
      expect(got!.payload).toBe(`cycle_${i}`);
      tmp.close();
    }
  });

  it("should handle high-dimensional vectors (1536 dims)", async () => {
    const vec: number[] = new Array(1536).fill(0).map((_, i) => (i % 100) / 100);
    for (let i = 0; i < 100; i++) {
      await db.put({
        namespace: "highdim",
        key: `hd${i}`,
        payload: `highdim_${i}`,
        vector: vec,
      });
    }
    const hits = await db.search({
      namespace: "highdim",
      query_vector: vec,
      top_k: 5,
    });
    expect(hits.length).toBeGreaterThan(0);
  });

  it("should handle concurrent inserts from multiple callers", { timeout: 30000 }, async () => {
    const promises: Promise<any>[] = [];
    for (let i = 0; i < 2000; i++) {
      promises.push(
        db.put({
          namespace: "multi_concurrent",
          key: `mk${i}`,
          payload: `mv${i}`,
          vector: [i % 5, 0, 0, 0],
        })
      );
    }
    await Promise.all(promises);

    const hits = await db.search({
      namespace: "multi_concurrent",
      query_vector: [0, 0, 0, 0],
      top_k: 10,
    });
    expect(hits.length).toBeGreaterThan(0);
  });

  it("should handle sustained search throughput", async () => {
    // Seed data
    for (let i = 0; i < 500; i++) {
      await db.put({
        namespace: "sustained",
        key: `sk${i}`,
        payload: `sv${i}`,
        vector: [i % 10, 0, 0, 0],
      });
    }

    const start = performance.now();
    const iterations = 200;
    for (let i = 0; i < iterations; i++) {
      await db.search({
        namespace: "sustained",
        query_vector: [0.5, 0, 0, 0],
        top_k: 10,
      });
    }
    const elapsed = performance.now() - start;
    const opsPerSec = (iterations / elapsed) * 1000;

    expect(opsPerSec).toBeGreaterThan(50);
  });
});
