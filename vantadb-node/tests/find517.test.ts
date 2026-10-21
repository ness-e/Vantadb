import { describe, it, expect, afterAll } from "vitest";
import { mkdtempSync, rmSync, statSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

import { VantaDb } from "../index.js";

/**
 * FIND-517 — the pre-reserve of the LSM segments.
 *
 * Opening a store used to reserve 64 MiB per LSM level (4 levels = 256 MiB
 * of address space) BEFORE a single record was written; the Node binding
 * added fjall's own 64 MiB journal on top (320 MiB per open store). For the
 * Ego product, where every open memory (project, sessions, sub-egos) pays
 * that cost, it was the dominant memory cost of the product — and it did
 * not show up in any `memory_limit`, because it was *reserved* space, not
 * live memory.
 *
 * These tests exercise the real native engine (napi -> Rust core), not a
 * mock: they measure the size of the segment files the engine actually
 * creates on disk (each one is mmap'd whole, so its length IS the reserved
 * address space of that level).
 */
describe("FIND-517: an empty store does not pre-reserve 64 MB per level", () => {
  const dirs: string[] = [];
  const tmp = (tag: string): string => {
    const dir = mkdtempSync(join(tmpdir(), `vantadb-node-f517-${tag}-`));
    dirs.push(dir);
    return dir;
  };

  afterAll(() => {
    for (const dir of dirs) {
      rmSync(dir, { recursive: true, force: true, maxRetries: 10, retryDelay: 100 });
    }
  });

  /** Size of every LSM segment file of a store. */
  const segmentSizes = (dir: string): number[] =>
    [0, 1, 2, 3].map((level) =>
      statSync(join(dir, "data", `vstore_L${level}.vanta`)).size
    );

  it("reserves a low-MB amount per level on an empty store", async () => {
    const dir = tmp("default");
    const db = await VantaDb.connect(dir);
    try {
      const sizes = segmentSizes(dir);
      for (const size of sizes) {
        expect(size).toBeLessThanOrEqual(4 * 1024 * 1024);
      }
      // 4 levels x <= 4 MiB => the store reserves at most 16 MiB, not 256 MiB.
      const total = sizes.reduce((a, b) => a + b, 0);
      expect(total).toBeLessThanOrEqual(16 * 1024 * 1024);
    } finally {
      await db.close();
    }
  });

  it("honours an explicit reserve_bytes per level", async () => {
    const dir = tmp("explicit");
    const db = await VantaDb.connect(dir, { reserve_bytes: 2 * 1024 * 1024 });
    try {
      for (const size of segmentSizes(dir)) {
        expect(size).toBe(2 * 1024 * 1024);
      }
    } finally {
      await db.close();
    }
  });

  it("keeps the reserve inside the caller's memory_limit", async () => {
    const dir = tmp("limited");
    // 8 MiB budget over 4 levels => at most 2 MiB per level.
    const db = await VantaDb.connect(dir, { memory_limit: 8 * 1024 * 1024 });
    try {
      for (const size of segmentSizes(dir)) {
        expect(size).toBeLessThanOrEqual(2 * 1024 * 1024);
      }
    } finally {
      await db.close();
    }
  });

  it("rejects a non-positive reserve_bytes", async () => {
    await expect(VantaDb.connect(tmp("bad"), { reserve_bytes: 0 })).rejects.toThrow(
      /reserve_bytes/
    );
  });

  it("still stores and reads records after the reserve change", async () => {
    const dir = tmp("roundtrip");
    const db = await VantaDb.connect(dir, { reserve_bytes: 64 * 1024 });
    try {
      await db.put({ namespace: "ns", key: "k", payload: "v", vector: [1, 0] });
      const got = await db.get("ns", "k");
      expect(got).not.toBeNull();
      expect(got!.payload).toBe("v");
    } finally {
      await db.close();
    }
  });
});
