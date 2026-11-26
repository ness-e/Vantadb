import { describe, it, expect, beforeAll, afterAll } from "vitest";
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { Client, DbError } from "../vantadb.js";

// FIND-79: portability safety net (export/import/reindex + strict importRecords).
//
// FS-backed portability (`exportAll`, `exportNamespace`, `importFile`) is NOT
// supported on the WASM runtime: the core reaches `std::fs`, which returns
// `IO error: operation not supported on this platform` on
// `wasm32-unknown-unknown` (`src/sdk/serialization/impl_export.rs`). These
// tests pin that honest failure with strong asserts and tmpdir-isolated
// paths — no soft try/catch. A real file round-trip export→import is
// DEFERRED to a WASI-OPFS wiring (FIND-79 H2).
//
// `reindexHnswFromText` is NOT filesystem-bound and is NOT in that group
// (FIND-728): the core implements it for every backend
// (`src/sdk/api/admin.rs:45`) and the WASM binding forwards it
// (`vantadb-wasm/src/lib.rs:1756`), so the old "rejects with platform IO error"
// pin asserted a limitation that no longer exists. What is pinned below is the
// real contract: it returns a rebuild report and leaves the records intact.

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
    const report = db.importRecords([{ namespace: "f79_rt", key: "k1", payload: "v1b" }]);
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
    const report = db.system.importRecords([{ namespace: "f79_sys", key: "s", payload: "v" }]);
    expect(report.inserted).toBe(1);
    expect(report.errors).toBe(0);
    expect(db.get({ namespace: "f79_sys", key: "s" })!.payload).toBe("v");
  });

  it("throws DbError when closed", () => {
    const tmp = Client.create();
    tmp.close();
    expect(() => tmp.importRecords([{ namespace: "f79_c", key: "k", payload: "v" }])).toThrow(
      DbError,
    );
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
    expect(() => db.exportAll(tmpPath("backup.jsonl"))).toThrow(/operation not supported/i);
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

  it("reindexHnswFromText rebuilds the index instead of failing (FIND-728)", () => {
    // Own client: `scanned_nodes` is engine-wide, so a shared DB would make the
    // count depend on the tests above.
    const local = Client.create();
    try {
      local.put({ namespace: "f79_re", key: "a", payload: "pa", vector: [0.1, 0.2] });
      local.put({ namespace: "f79_re", key: "b", payload: "pb", vector: [0.3, 0.4] });

      const report = local.reindexHnswFromText("f79_re") as {
        scanned_nodes: number;
        indexed_vectors: number;
        success: boolean;
      };
      expect(report.success).toBe(true);
      expect(report.scanned_nodes).toBe(2);
      expect(report.indexed_vectors).toBe(2);

      // A rebuild is not a wipe: the records are still there.
      expect(local.get({ namespace: "f79_re", key: "a" })!.payload).toBe("pa");
      expect(local.list({ namespace: "f79_re" }).records.length).toBe(2);
    } finally {
      local.close();
    }
  });

  it("system subclient delegates export/import errors and reindex returns a report", () => {
    expect(() => db.system.exportAll(tmpPath("s.jsonl"))).toThrow(DbError);
    expect(() => db.system.importFile(tmpPath("s.jsonl"))).toThrow(DbError);
    expect(() => db.system.reindexHnswFromText("")).toThrow(DbError);

    db.put({ namespace: "f79_sysre", key: "k", payload: "v", vector: [0.5, 0.5] });
    const report = db.system.reindexHnswFromText("f79_sysre") as {
      scanned_nodes: number;
      success: boolean;
    };
    expect(report.success).toBe(true);
    expect(report.scanned_nodes).toBeGreaterThanOrEqual(1);
  });
});
