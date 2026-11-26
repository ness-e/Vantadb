import { describe, it, expect } from "vitest";
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { NativeVantaDB } from "../native.js";

/**
 * EGO-12: `NativeVantaDB.reopen()` revive el mismo handle sin re-vincular
 * consumidores (el problema de Ego: `auditor.setAdapter` tras cada
 * reapertura). Mismo path por defecto, o cambio de directorio con `path`.
 */

const nativeAvailable = await import("vantadb-node").then(
  () => true,
  () => false,
);

describe("reopen() (EGO-12)", () => {
  it.runIf(nativeAvailable)("revives the same handle without rebinding", async () => {
    const dir = mkdtempSync(join(tmpdir(), "vanta-ts-reopen-"));
    try {
      const db = await NativeVantaDB.connect(dir);
      await db.put({ namespace: "life", key: "k1", payload: "survives", vector: [1, 0, 0] });
      await db.close();

      await db.reopen();
      try {
        expect((await db.get({ namespace: "life", key: "k1" }))?.payload).toBe("survives");
        await db.put({ namespace: "life", key: "k2", payload: "v2", vector: [0, 1, 0] });
        expect((await db.get({ namespace: "life", key: "k2" }))?.payload).toBe("v2");
      } finally {
        await db.close();
      }
    } finally {
      rmSync(dir, { recursive: true, force: true });
    }
  });

  it.runIf(nativeAvailable)("reopen(otherPath) switches directory", async () => {
    const dirA = mkdtempSync(join(tmpdir(), "vanta-ts-reopen-a-"));
    const dirB = mkdtempSync(join(tmpdir(), "vanta-ts-reopen-b-"));
    try {
      const db = await NativeVantaDB.connect(dirA);
      await db.put({ namespace: "life", key: "k1", payload: "in-a", vector: [1, 0, 0] });
      await db.close();

      await db.reopen(dirB);
      try {
        expect(await db.get({ namespace: "life", key: "k1" })).toBeNull();
        await db.put({ namespace: "life", key: "k9", payload: "in-b", vector: [0, 1, 0] });
        expect((await db.get({ namespace: "life", key: "k9" }))?.payload).toBe("in-b");
      } finally {
        await db.close();
      }
    } finally {
      rmSync(dirA, { recursive: true, force: true });
      rmSync(dirB, { recursive: true, force: true });
    }
  });
});
