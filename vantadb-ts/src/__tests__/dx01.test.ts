import { describe, expect, it } from "vitest";
import { Client, DbError, ERROR_CODES } from "../vantadb.js";

/**
 * DX-01 — H-009: non-string wire arguments must not trap the wasm glue.
 *
 * Before the fix, a non-string `key`/`namespace` was marshalled into
 * `passStringToWasm0` (`vantadb_wasm_bg.js:1551`) → `__wbindgen_realloc` with
 * an invalid pointer → `RuntimeError: memory access out of bounds` (reproduced
 * on Node 22.23.3 / 26.0.0 / 26.8.1 / 26.10.0 with the published `vantadb@0.8.0`).
 *
 * Contract after the fix:
 *  - `string | number | bigint` → coerced to its string form (same convention
 *    as `_rootsToWire` / `insertNode(String(id))`; numbers must be safe
 *    integers — use bigint above 2^53);
 *  - anything else (object, boolean, null, …) → clean `DbError`
 *    `VANTADB_INVALID_ARGUMENT`, never a wasm trap.
 */
describe("DX-01 — non-string wire arguments (H-009)", () => {
  it("coerces a numeric key to its string form instead of trapping (H-009 repro)", () => {
    const db = Client.create();
    try {
      db.put({ namespace: "demo", key: "5", payload: "hello" });
      const got = db.get({ namespace: "demo", key: 5 as unknown as string });
      expect(got?.payload).toBe("hello");
    } finally {
      db.close();
    }
  });

  it("coerces a bigint key", () => {
    const db = Client.create();
    try {
      db.put({ namespace: "demo", key: "42", payload: "answer" });
      const got = db.get({ namespace: "demo", key: 42n as unknown as string });
      expect(got?.payload).toBe("answer");
    } finally {
      db.close();
    }
  });

  it("throws a clean VANTADB_INVALID_ARGUMENT for an object key (no wasm trap)", () => {
    const db = Client.create();
    try {
      db.put({ namespace: "demo", key: "k", payload: "hello" });
      let caught: unknown;
      try {
        db.get({ namespace: "demo", key: {} as unknown as string });
      } catch (e) {
        caught = e;
      }
      expect(caught).toBeInstanceOf(DbError);
      expect((caught as DbError).code).toBe(ERROR_CODES.INVALID_ARGUMENT);
      expect((caught as DbError).message).toMatch(/must be a string/);
      // The instance must survive the rejected call (no poisoned wasm memory).
      expect(db.get({ namespace: "demo", key: "k" })?.payload).toBe("hello");
    } finally {
      db.close();
    }
  });

  it("guards delete / list / count / query against non-string args", () => {
    const db = Client.create();
    try {
      db.put({ namespace: "1", key: "k", payload: "hello" });
      expect(db.list({ namespace: 1 as unknown as string }).records.length).toBe(1);
      expect(db.count({ namespace: 1 as unknown as string })).toBe(1n);

      db.put({ namespace: "demo", key: "1", payload: "one" });
      expect(db.delete({ namespace: "demo", key: 1 as unknown as string })).toBe(true);
      expect(db.get({ namespace: "demo", key: "1" })).toBeNull();

      let queryErr: unknown;
      try {
        db.query(5 as unknown as string);
      } catch (e) {
        queryErr = e;
      }
      expect(queryErr).toBeInstanceOf(DbError);
      expect((queryErr as DbError).message).not.toMatch(/memory access out of bounds/);
    } finally {
      db.close();
    }
  });
});
