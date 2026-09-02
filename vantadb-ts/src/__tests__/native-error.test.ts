import { describe, it, expect } from "vitest";
import { NativeVantaDB } from "../native.js";
import { VantaError, ERROR_CODES } from "../errors.js";

/**
 * TS-02 regression tests: every failure raised by the native binding —
 * synchronous throws AND async promise rejections — must surface to callers
 * as a wrapped `VantaError`, never as a raw binding error.
 *
 * We bypass `NativeVantaDB.connect()` (needs a built native binary) and
 * inject a fake inner whose methods reject asynchronously.
 */
function makeDbWith(inner: Record<string, unknown>): NativeVantaDB {
  // The constructor is private but accessible at runtime; this is deliberate
  // so the wrapping contract can be tested without a compiled .node binary.
  return new (NativeVantaDB as unknown as new (i: unknown) => NativeVantaDB)(
    inner,
  );
}

describe("NativeVantaDB error wrapping (TS-02)", () => {
  it("recovers the VANTADB_* code from the napi '{code}: {message}' prefix (ERR-TS-01)", async () => {
    // vantadb-node's map_err emits e.g. "VANTADB_NOT_FOUND: Node not found: 7".
    const db = makeDbWith({
      get: () => Promise.reject(new Error("VANTADB_NOT_FOUND: Node not found: 7")),
    });
    await expect(db.get("ns", "k")).rejects.toMatchObject({
      code: "VANTADB_NOT_FOUND",
      message: expect.stringContaining("Node not found: 7"),
    });
    // The prefix must not leak into the human message twice.
    await expect(db.get("ns", "k")).rejects.toMatchObject({
      message: expect.not.stringMatching(/^get: VANTADB_/),
    });
  });

  it("classifies an unprefixed native error via message heuristics, never NATIVE_ERROR (ERR-TS-01)", async () => {
    const db = makeDbWith({
      get: () => Promise.reject(new Error("engine panicked on background thread")),
    });
    await expect(db.get("ns", "k")).rejects.toBeInstanceOf(VantaError);
    await expect(db.get("ns", "k")).rejects.toMatchObject({
      code: "VANTADB_WASM_ERROR",
      message: expect.stringContaining(
        "get: engine panicked on background thread",
      ),
    });
  });

  it("wraps an ASYNC rejection from the inner binding in a VantaError with code", async () => {
    const db = makeDbWith({
      get: () => Promise.reject(new Error("engine panicked on background thread")),
    });
    await expect(db.get("ns", "k")).rejects.toBeInstanceOf(VantaError);
    await expect(db.get("ns", "k")).rejects.toMatchObject({
      code: "VANTADB_WASM_ERROR",
      message: expect.stringContaining(
        "get: engine panicked on background thread",
      ),
    });
  });

  it("wraps a SYNCHRONOUS throw from the inner binding in a VantaError", async () => {
    const db = makeDbWith({
      flush: () => {
        throw new Error("sync boom");
      },
    });
    await expect(db.flush()).rejects.toBeInstanceOf(VantaError);
    await expect(db.flush()).rejects.toMatchObject({ code: "VANTADB_WASM_ERROR" });
  });

  it("passes an existing VantaError through untouched", async () => {
    const original = new VantaError(ERROR_CODES.BUSY, "database busy");
    const db = makeDbWith({ delete: () => Promise.reject(original) });
    let caught: unknown;
    try {
      await db.delete("ns", "k");
    } catch (e) {
      caught = e;
    }
    expect(caught).toBe(original);
  });
});
