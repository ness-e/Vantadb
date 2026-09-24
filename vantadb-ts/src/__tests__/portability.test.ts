import { describe, it, expect, beforeAll, afterAll } from "vitest";
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { Client, DbError } from "../vantadb.js";

// FIND-79: portability safety net (export/import/reindex + strict importRecords).
//
// FS-backed portability (`exportAll`, `exportNamespace`, `importFile`) and
// `reindexHnswFromText` are NOT supported on the WASM runtime: the core
// returns `IO error: operation not supported on this platform`
// (`src/sdk/serialization/impl_export.rs`, `src/sdk/api/admin.rs`). These
// tests pin that honest failure with strong asserts and tmpdir-isolated
// paths — no soft try/catch. A real file round-trip export→import is
// DEFERRED to a native binding / future WASI-OPFS wiring (FIND-79 H2).

function tmpPath(name: string): string {
  return join(mkdtempSync(join(tmpdir(), "find79-")), name);
}

describe("FIND-79: importRecords strict (no soft try/catch)", () => {
  let db: Client;

  beforeAll(() => {
    db = Client.create();
  });

  afterAll(() => {
    db.close();
  });

  it("inserts new records and reports inserted:2/errors:0", () => {
    const report = db.importRecords([
      { namespace: "f79_rt", key: "k1", payload: "v1" },
      { namespace: "f79_rt", key: "k2", payload: "v2" },
    ]);
    expect(report.inserted).toBe(2);
    expect(report.updated).toBe(0);
    expect(report.errors).toBe(0);
    expect(report.skipped).toBe(0);
    expect(db.get({ namespace: "f79_rt", key: "k1" })!.payload).toBe("v1");
    expect(db.get({ namespace: "f79_rt", key: "k2" })!.payload).toBe("v2");
  });

  it("re-import of existing keys reports updated (core parity)", () => {
    const report = db.importRecords([
      { namespace: "f79_rt", key: "k1", payload: "v1b" },
    ]);
    expect(report.inserted).toBe(0);
    expect(report.updated).toBe(1);
    expect(report.errors).toBe(0);
    expect(db.get({ namespace: "f79_rt", key: "k1" })!.payload).toBe("v1b");
  });

  it("empty array reports all-zero without throwing", () => {
    expect(db.importRecords([])).toEqual({
      inserted: 0,
      updated: 0,
      skipped: 0,
      errors: 0,
      duration_ms: expect.any(Number),
    });
  });

  it("invalid record counts errors:1 without throwing (core parity)", () => {
    const report = db.importRecords([
      { namespace: "f79_mix", key: "ok", payload: "v" },
      { namespace: "", key: "bad", payload: "v" },
    ]);
    expect(report.inserted).toBe(1);
    expect(report.errors).toBe(1);
    expect(db.get({ namespace: "f79_mix", key: "ok" })!.payload).toBe("v");
  });

  it("accepts full MemoryRecord shape (get() output round-trips)", () => {
    db.put({ namespace: "f79_full", key: "a", payload: "pa", metadata: { lang: "en" } });
    const got = db.get({ namespace: "f79_full", key: "a" })!;
    const report = db.importRecords([got]);
    expect(report.errors).toBe(0);
    expect(report.inserted + report.updated).toBe(1);
    expect(db.get({ namespace: "f79_full", key: "a" })!.payload).toBe("pa");
  });

  it("round-trips metadata and vector through import", () => {
    const report = db.importRecords([
      {
        namespace: "f79_meta",
        key: "m",
        payload: "p",
        metadata: { tier: "hot" },
        vector: [0.5, 0.5],
      },
    ]);
    expect(report.inserted).toBe(1);
    expect(report.errors).toBe(0);
    const got = db.get({ namespace: "f79_meta", key: "m" })!;
    expect(got.payload).toBe("p");
    expect(got.vector).toBeDefined();
  });

  it("system subclient delegates with identical report", () => {
    const report = db.system.importRecords([
      { namespace: "f79_sys", key: "s", payload: "v" },
    ]);
    expect(report.inserted).toBe(1);
    expect(report.errors).toBe(0);
    expect(db.get({ namespace: "f79_sys", key: "s" })!.payload).toBe("v");
  });

  it("throws DbError when closed", () => {
    const tmp = Client.create();
    tmp.close();
    expect(() =>
      tmp.importRecords([{ namespace: "f79_c", key: "k", payload: "v" }]),
    ).toThrow(DbError);
  });
});

describe("FIND-79: FS-backed portability pins platform error (DEFER e2e)", () => {
  let db: Client;

  beforeAll(() => {
    db = Client.create();
  });

  afterAll(() => {
    db.close();
  });

  it("exportAll rejects with platform IO error (WASM has no std::fs)", () => {
    db.put({ namespace: "f79_exp", key: "k", payload: "v" });
    expect(() => db.exportAll(tmpPath("backup.jsonl"))).toThrow(
      /operation not supported/i,
    );
  });

  it("exportNamespace rejects with platform IO error", () => {
    expect(() => db.exportNamespace(tmpPath("ns.jsonl"), "f79_exp")).toThrow(
      /operation not supported/i,
    );
  });

  it("exportNamespace with filter rejects with platform IO error", () => {
    expect(() =>
      db.exportNamespace(tmpPath("f.jsonl"), "f79_exp", [
        { field: "tier", op: "Eq", value: "hot" },
      ]),
    ).toThrow(/operation not supported/i);
  });

  it("importFile rejects with DbError", () => {
    expect(() => db.importFile(tmpPath("nope.jsonl"))).toThrow(DbError);
  });

  it("reindexHnswFromText rejects with platform IO error", () => {
    expect(() => db.reindexHnswFromText("f79_exp")).toThrow(
      /operation not supported/i,
    );
  });

  it("system subclient delegates export/import/reindex identically", () => {
    expect(() => db.system.exportAll(tmpPath("s.jsonl"))).toThrow(DbError);
    expect(() => db.system.importFile(tmpPath("s.jsonl"))).toThrow(DbError);
    expect(() => db.system.reindexHnswFromText("f79_exp")).toThrow(DbError);
  });
});
