import { describe, it, expect, beforeAll, afterAll } from "vitest";
import { Client } from "../vantadb.js";

describe("Client Load Tests", () => {
  let db: Client;

  beforeAll(() => {
    db = Client.create();
  });

  afterAll(() => {
    db.close();
  });

  it("should handle concurrent put operations", async () => {
    // Arrange: 1000 puts concurrentes.
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
    // Act (bulk único).
    await Promise.all(promises);

    // Assert (final).
    const hits = await db.search({
      namespace: "concurrent",
      query_vector: [1, 0, 0, 0],
      top_k: 10,
    });
    expect(hits.length).toBeGreaterThan(0);
  });

  it("should handle large batch inserts (5000 vectors)", { timeout: 30000 }, async () => {
    // Arrange: 5000 inputs.
    const inputs: any[] = [];
    for (let i = 0; i < 5000; i++) {
      inputs.push({
        namespace: "large_batch",
        key: `bk${i}`,
        payload: `batch_${i}`,
        vector: [(i % 256) / 256, 0.5, 0.3, 0.1],
      });
    }
    // Act (bulk único).
    const records = await db.putBatch(inputs);
    // Assert (final).
    expect(records.length).toBe(5000);

    const hits = await db.search({
      namespace: "large_batch",
      query_vector: [0.5, 0.5, 0.3, 0.1],
      top_k: 10,
    });
    expect(hits.length).toBe(10);
  });

  it("should not error on repeated create/destroy cycles", async () => {
    // Arrange: 50 independent cycle namespaces.
    const cycles = Array.from({ length: 50 }, (_, i) => `cycle_${i}`);
    // Act (bulk único): cada ciclo create→put→get→close; se recolecta lo
    // observado sin asertar dentro del loop (integración legítima: NO partir).
    const observed: Array<{ payload: unknown; expected: string }> = [];
    for (let i = 0; i < cycles.length; i++) {
      const ns = cycles[i];
      const tmp = Client.create();
      await tmp.put({
        namespace: ns,
        key: "k",
        payload: ns,
        vector: [i % 10, 0, 0, 0],
      });
      const got = await tmp.get({ namespace: ns, key: "k" });
      observed.push({ payload: got?.payload, expected: ns });
      tmp.close();
    }
    // Assert (final): los 50 ciclos round-trippearon su payload.
    expect(observed.length).toBe(50);
    for (const { payload, expected } of observed) {
      expect(payload).toBe(expected);
    }
  });

  it("should handle high-dimensional vectors (1536 dims)", async () => {
    // Arrange: vector 1536-d + 100 inputs.
    const vec: number[] = new Array(1536).fill(0).map((_, i) => (i % 100) / 100);
    const inputs: any[] = [];
    for (let i = 0; i < 100; i++) {
      inputs.push({
        namespace: "highdim",
        key: `hd${i}`,
        payload: `highdim_${i}`,
        vector: vec,
      });
    }
    // Act (bulk único): una sola inserción batch en vez de 100 puts secuenciales.
    const records = await db.putBatch(inputs);
    expect(records.length).toBe(100);
    // Assert (final).
    const hits = await db.search({
      namespace: "highdim",
      query_vector: vec,
      top_k: 5,
    });
    expect(hits.length).toBeGreaterThan(0);
  });

  it("should handle concurrent inserts from multiple callers", { timeout: 30000 }, async () => {
    // Arrange: 2000 puts concurrentes.
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
    // Act (bulk único).
    await Promise.all(promises);

    // Assert (final).
    const hits = await db.search({
      namespace: "multi_concurrent",
      query_vector: [1, 0, 0, 0],
      top_k: 10,
    });
    expect(hits.length).toBeGreaterThan(0);
  });

  it("should handle sustained search throughput", async () => {
    // Arrange: seed 500 records (bulk).
    const seed: any[] = [];
    for (let i = 0; i < 500; i++) {
      seed.push({
        namespace: "sustained",
        key: `sk${i}`,
        payload: `sv${i}`,
        vector: [i % 10, 0, 0, 0],
      });
    }
    await db.putBatch(seed);

    // Act (bulk único): 200 búsquedas secuenciales = medición de throughput.
    // Loop de medición legítimo (integración): NO partir en N tests.
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

    // Assert (final).
    expect(opsPerSec).toBeGreaterThan(50);
  });
});
