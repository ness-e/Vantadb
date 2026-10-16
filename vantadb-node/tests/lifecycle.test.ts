import { describe, it, expect, afterAll } from "vitest";
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

import { VantaDb } from "../index.js";

/**
 * EGO-06: lifecycle hardening del binding Node.
 *
 * El lock Fjall es un flock-OS: muere con el proceso, así que un `Locked`
 * en reapertura siempre es un handle vivo EN ESTE proceso (close olvidado,
 * clon filtrado o segundo open del mismo path). El registro por path
 * canónico convierte ese fallo críptico en un `VANTADB_BUSY` fail-fast que
 * nombra el path. Matriz = flock del core: un writer excluye todo;
 * lectores comparten con lectores (EGO-09 intacto).
 */
describe("lifecycle (EGO-06)", () => {
  const tempDirs: string[] = [];

  afterAll(() => {
    for (const dir of tempDirs) {
      rmSync(dir, { recursive: true, force: true });
    }
  });

  function freshDir(prefix: string): string {
    const dir = mkdtempSync(join(tmpdir(), prefix));
    tempDirs.push(dir);
    return dir;
  }

  it("second writer open on the same path fails fast naming the path", async () => {
    const dir = freshDir("vantadb-node-lifecycle-");
    const db1 = await VantaDb.connect(dir);
    try {
      await expect(VantaDb.connect(dir)).rejects.toThrow(/already open.*writer/);
    } finally {
      await db1.close();
    }
  });

  it("reopen after close works and data persists", async () => {
    const dir = freshDir("vantadb-node-reopen-");
    const db1 = await VantaDb.connect(dir);
    await db1.put({ namespace: "life", key: "k1", payload: "survives", vector: [1, 0, 0] });
    await db1.close();

    const db2 = await VantaDb.connect(dir);
    try {
      const got = await db2.get("life", "k1");
      expect(got).not.toBeNull();
      expect(got!.payload).toBe("survives");
    } finally {
      await db2.close();
    }
  });

  it("second open is rejected even read-only (Fjall ignores read_only — Engine debt)", async () => {
    // EGO-01 debt: `FjallBackend::open` has no read-only mode, so even a
    // second read-only open dies in the core with `Locked`. The registry
    // fails fast with VANTADB_BUSY instead. Shared reads arrive with
    // Engine lock-free reads; until then the pattern is daemon/socket.
    const dir = freshDir("vantadb-node-readers-");
    const seed = await VantaDb.connect(dir);
    await seed.put({ namespace: "life", key: "k1", payload: "shared", vector: [1, 0, 0] });
    await seed.close();

    const r1 = await VantaDb.connect(dir, { read_only: true });
    try {
      await expect(VantaDb.connect(dir, { read_only: true })).rejects.toThrow(
        /already open.*reader/,
      );
      expect((await r1.get("life", "k1"))?.payload).toBe("shared");
    } finally {
      await r1.close();
    }
  });

  it("writer blocked while a reader holds the path", async () => {
    const dir = freshDir("vantadb-node-writerblocked-");
    const seed = await VantaDb.connect(dir);
    await seed.put({ namespace: "life", key: "k1", payload: "v", vector: [1, 0, 0] });
    await seed.close();

    const reader = await VantaDb.connect(dir, { read_only: true });
    try {
      await expect(VantaDb.connect(dir)).rejects.toThrow(/already open.*reader/);
    } finally {
      await reader.close();
    }
  });

  it("double close is safe and frees nobody else's reservation", async () => {
    const dir = freshDir("vantadb-node-doubleclose-");
    const db1 = await VantaDb.connect(dir);
    await db1.close();
    await db1.close();
    // Slot consumido: un writer nuevo debe poder abrir.
    const db2 = await VantaDb.connect(dir);
    try {
      await db2.put({ namespace: "life", key: "k", payload: "v", vector: [1, 0, 0] });
    } finally {
      await db2.close();
    }
  });

  it("reopen() revives the same handle without rebinding (EGO-12)", async () => {
    const dir = freshDir("vantadb-node-reopen-");
    const db = await VantaDb.connect(dir);
    await db.put({ namespace: "life", key: "k1", payload: "survives", vector: [1, 0, 0] });
    await db.close();

    await db.reopen();
    try {
      const got = await db.get("life", "k1");
      expect(got).not.toBeNull();
      expect(got!.payload).toBe("survives");
      // El handle sigue operativo tras reopen.
      await db.put({ namespace: "life", key: "k2", payload: "v2", vector: [0, 1, 0] });
      expect((await db.get("life", "k2"))?.payload).toBe("v2");
    } finally {
      await db.close();
    }
  });

  it("reopen(otherPath) switches directory (EGO-12)", async () => {
    const dirA = freshDir("vantadb-node-reopen-a-");
    const dirB = freshDir("vantadb-node-reopen-b-");
    const db = await VantaDb.connect(dirA);
    await db.put({ namespace: "life", key: "k1", payload: "in-a", vector: [1, 0, 0] });
    await db.close();

    await db.reopen(dirB);
    try {
      expect(await db.get("life", "k1")).toBeNull();
      await db.put({ namespace: "life", key: "k9", payload: "in-b", vector: [0, 1, 0] });
      expect((await db.get("life", "k9"))?.payload).toBe("in-b");
    } finally {
      await db.close();
    }

    // A liberó su reserva al cambiar: reabrir A funciona y conserva k1.
    const dbA = await VantaDb.connect(dirA);
    try {
      expect((await dbA.get("life", "k1"))?.payload).toBe("in-a");
    } finally {
      await dbA.close();
    }
  });
});
