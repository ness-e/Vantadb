import { describe, it, expect, beforeAll, afterAll } from "vitest";
import { VantaDB } from "../vantadb.js";

/**
 * FIND-01 — Flat metadata input.
 *
 * Callers pass plain JS values (`{ lang: "en" }`); the SDK normalizes to the
 * tagged wire form (`{ lang: { String: "en" } }`) expected by the Rust engine.
 * The tagged form remains accepted for backward compat. Records READ from the
 * engine come back with metadata as a Map of tagged values
 * (serde_wasm_bindgen deserializes the HashMap that way).
 */

/** Runtime metadata is a Map<string, VantaValue> — convert for assertions. */
function metaOf(m: unknown): Record<string, unknown> {
  return Object.fromEntries(m as Map<string, unknown>);
}

describe("FIND-01: flat metadata input", () => {
  let db: VantaDB;

  beforeAll(() => {
    db = VantaDB.create();
  });

  afterAll(() => {
    db.close();
  });

  it("put with plain string metadata round-trips via get", async () => {
    await db.put({
      namespace: "flat",
      key: "k1",
      payload: "hello",
      metadata: { lang: "en" },
    });
    const got = await db.get("flat", "k1");
    expect(got).not.toBeNull();
    expect(got!.payload).toBe("hello");
    // Engine stores/returns the tagged wire form.
    expect(metaOf(got!.metadata)).toEqual({ lang: { String: "en" } });
  });

  it("plain values map by JS type: bool/int/float/null", async () => {
    await db.put({
      namespace: "flat",
      key: "types",
      payload: "typed",
      metadata: { ok: true, count: 3, ratio: 0.5, nothing: null },
    });
    const got = await db.get("flat", "types");
    expect(metaOf(got!.metadata)).toEqual({
      ok: { Bool: true },
      count: { Int: 3 },
      ratio: { Float: 0.5 },
      // Engine serializes the Null variant as a bare "Null" string (unit variant).
      nothing: "Null",
    });
  });

  it("tagged wire form still works (backward compat)", async () => {
    await db.put({
      namespace: "flat",
      key: "tagged",
      payload: "legacy",
      metadata: { source: { String: "manual" }, priority: { Int: 1 } },
    });
    const got = await db.get("flat", "tagged");
    expect(got!.payload).toBe("legacy");
    expect(metaOf(got!.metadata)).toEqual({
      source: { String: "manual" },
      priority: { Int: 1 },
    });
  });

  it("putBatch accepts plain metadata", async () => {
    const records = await db.putBatch([
      { namespace: "flat_batch", key: "a", payload: "a", metadata: { tier: "hot" } },
      { namespace: "flat_batch", key: "b", payload: "b" },
    ]);
    expect(records.length).toBe(2);
    expect(metaOf(records[0].metadata)).toEqual({ tier: { String: "hot" } });
    const got = await db.get("flat_batch", "a");
    expect(got!.payload).toBe("a");
  });

  it("list() filters accept plain values", async () => {
    await db.put({
      namespace: "flat_filter",
      key: "x",
      payload: "match me",
      metadata: { env: "prod" },
    });
    await db.put({
      namespace: "flat_filter",
      key: "y",
      payload: "not prod",
      metadata: { env: "dev" },
    });
    const page = db.list("flat_filter", { filters: { env: "prod" } });
    expect(page.records.length).toBe(1);
    expect(page.records[0].key).toBe("x");
  });

  it("search() filters accept plain values", async () => {
    await db.put({
      namespace: "flat_vec",
      key: "x",
      payload: "match me",
      metadata: { env: "dev" },
      vector: [1.0, 0.0],
    });
    await db.put({
      namespace: "flat_vec",
      key: "y",
      payload: "other env",
      metadata: { env: "prod" },
      vector: [1.0, 0.0],
    });
    const hits = db.search({
      namespace: "flat_vec",
      query_vector: [1.0, 0.0],
      filters: { env: "dev" },
      top_k: 10,
    });
    expect(hits.length).toBe(1);
    expect(hits[0].record.key).toBe("x");
  });

  it("deleteByFilter accepts plain values", async () => {
    const deleted = db.deleteByFilter("flat_filter", [
      { field: "env", op: "Eq", value: "prod" },
    ]);
    expect(deleted).toBeGreaterThanOrEqual(1n);
    expect(await db.get("flat_filter", "x")).toBeNull();
  });
});
