import { describe, it, expect, beforeAll, afterAll } from "vitest";
import { Client } from "../vantadb.js";
import { NativeVantaDB } from "../native.js";
import { DbError, ERROR_CODES } from "../errors.js";

/**
 * EGO-11 (decisión owner): barridos explícitos vía `scan()` — `searchMulti`
 * ordena por relevancia y exige señal de query; para caminar todos los
 * records se usa `scan()`/`list()`, nunca `text_query: ""` (el guard lo
 * rechaza y el mensaje apunta acá).
 */

const NS = "scan-ego11";

const nativeAvailable = await import("vantadb-node").then(
  () => true,
  () => false,
);

function seedWasm(db: Client): void {
  for (let i = 0; i < 3; i++) {
    db.put({ namespace: NS, key: `k${i}`, payload: `payload ${i}` });
  }
}

async function seedNative(db: NativeVantaDB): Promise<void> {
  for (let i = 0; i < 3; i++) {
    await db.put({ namespace: NS, key: `k${i}`, payload: `payload ${i}` });
  }
}

describe("scan() (EGO-11)", () => {
  describe("WASM Client", () => {
    let db: Client;

    beforeAll(() => {
      db = Client.create();
      seedWasm(db);
    });

    afterAll(() => {
      db.close();
    });

    it("sweeps the whole namespace with cursor pagination", () => {
      const p1 = db.scan({ namespace: NS, limit: 2 });
      expect(p1.records).toHaveLength(2);
      expect(p1.next_cursor).toBeDefined();
      const p2 = db.scan({ namespace: NS, limit: 2, cursor: p1.next_cursor });
      const keys = [...p1.records, ...p2.records].map((r) => r.key).sort();
      expect(keys).toEqual(["k0", "k1", "k2"]);
    });

    it("matches list() for the same page", () => {
      const viaScan = db
        .scan({ namespace: NS, limit: 10 })
        .records.map((r) => r.key)
        .sort();
      const viaList = db
        .list({ namespace: NS, limit: 10 })
        .records.map((r) => r.key)
        .sort();
      expect(viaScan).toEqual(viaList);
    });

    it("is exposed on db.memory", () => {
      expect(db.memory.scan({ namespace: NS, limit: 10 }).records).toHaveLength(3);
    });

    it("rejects filters — sweeps are unfiltered, use list()", () => {
      expect(() =>
        (db.scan as (i: unknown) => unknown)({ namespace: NS, filters: {} }),
      ).toThrowError(DbError);
      try {
        (db.scan as (i: unknown) => unknown)({ namespace: NS, filters: {} });
      } catch (e) {
        expect((e as DbError).code).toBe(ERROR_CODES.VALIDATION_ERROR);
      }
    });

    it("search without query signal points at scan()", () => {
      expect(() => db.search({ namespace: NS, query_vector: [] })).toThrow(/scan/);
    });
  });

  describe("NativeVantaDB wrapper", () => {
    it.runIf(nativeAvailable)("sweeps via scan() like list()", async () => {
      const db = await NativeVantaDB.connect(":memory:");
      try {
        await seedNative(db);
        const page = await db.scan({ namespace: NS, limit: 10 });
        expect(page.records.map((r) => r.key).sort()).toEqual(["k0", "k1", "k2"]);
        await expect(
          db.scan({ namespace: NS, filters: {} } as never),
        ).rejects.toMatchObject({ code: ERROR_CODES.VALIDATION_ERROR });
      } finally {
        await db.close();
      }
    });
  });
});
