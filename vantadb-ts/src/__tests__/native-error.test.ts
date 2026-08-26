import { describe, it, expect } from "vitest";
import { NativeVantaDB } from "../native.js";
import { VantaError } from "../errors.js";

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
  it("wraps an ASYNC rejection from the inner binding in a VantaError with code", async () => {
    const db = makeDbWith({
      get: () => Promise.reject(new Error("engine panicked on background thread")),
    });
    await expect(db.get("ns", "k")).rejects.toBeInstanceOf(VantaError);
    await expect(db.get("ns", "k")).rejects.toMatchObject({
      code: "NATIVE_ERROR",
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
    await expect(db.flush()).rejects.toMatchObject({ code: "NATIVE_ERROR" });
  });

  it("passes an existing VantaError through untouched", async () => {
    const original = new VantaError("BUSY", "database busy");
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
