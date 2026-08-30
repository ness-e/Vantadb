import { describe, it, expect, afterAll } from "vitest";
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

import { VantaDb } from "../index.js";

/**
 * API-surface coverage (plan BND-12): search / explain_search / put_batch /
 * capabilities / close-drain, plus FFI-boundary validation errors.
 *
 * The close-drain tests exercise the OpGate durability barrier documented in
 * src/lib.rs: once `close()` begins, new operations are rejected and every
 * in-flight operation finishes before flush — a fire-and-forget `put` can
 * never be silently lost on process exit.
 */
describe("vantadb-node api surface", () => {
  const dirs: string[] = [];
  const tmp = (tag: string): string => {
    const dir = mkdtempSync(join(tmpdir(), `vantadb-node-api-${tag}-`));
    dirs.push(dir);
    return dir;
  };

  afterAll(() => {
    for (const dir of dirs) {
      rmSync(dir, { recursive: true, force: true });
    }
  });

  // ── put_batch ──────────────────────────────────────────────────────────────

  it("putBatch stores several records and returns them", async () => {
    const db = await VantaDb.connect(tmp("batch"));
    try {
      const out = await db.putBatch([
        { namespace: "batch", key: "k1", payload: "one" },
        { namespace: "batch", key: "k2", payload: "two", vector: [1, 0] },
      ]);
      expect(Array.isArray(out)).toBe(true);
      expect(out.map((r) => r.key).sort()).toEqual(["k1", "k2"]);
      expect(out[0].version).toBe(1);
      // node_id travels as a decimal string (u128 > Number.MAX_SAFE_INTEGER).
      expect(typeof out[0].node_id).toBe("string");
    } finally {
      await db.close();
    }
  });

  it("putBatch rejects a non-array input", async () => {
    const db = await VantaDb.connect(tmp("batcherr"));
    try {
      await expect(
        // @ts-expect-error — exercising the runtime guard behind the type
        db.putBatch({ namespace: "batch", key: "k1", payload: "one" }),
      ).rejects.toThrow(/records must be an array/);
    } finally {
      await db.close();
    }
  });

  it("put rejects a record with a missing required field", async () => {
    const db = await VantaDb.connect(tmp("puterr"));
    try {
      await expect(
        // @ts-expect-error — exercising the runtime guard behind the type
        db.put({ namespace: "ns", payload: "no key here" }),
      ).rejects.toThrow(/missing required field `key`/);
    } finally {
      await db.close();
    }
  });

  it("put rejects a negative ttl_ms", async () => {
    const db = await VantaDb.connect(tmp("ttl"));
    try {
      await expect(
        db.put({
          namespace: "ns",
          key: "k",
          payload: "v",
          // @ts-expect-error — exercising the runtime guard behind the type
          ttl_ms: -5,
        }),
      ).rejects.toThrow(/ttl_ms.*non-negative integer/);
    } finally {
      await db.close();
    }
  });

  // ── list / listNamespaces ──────────────────────────────────────────────────

  it("list paginates with limit and cursor", async () => {
    const db = await VantaDb.connect(tmp("list"));
    try {
      await db.putBatch([
        { namespace: "page", key: "a", payload: "1" },
        { namespace: "page", key: "b", payload: "2" },
        { namespace: "page", key: "c", payload: "3" },
      ]);

      const first = await db.list("page", { limit: 2 });
      expect(first.records).toHaveLength(2);
      expect(first.next_cursor).toBeDefined();

      const second = await db.list("page", { limit: 2, cursor: first.next_cursor });
      expect(second.records).toHaveLength(1);
      const seen = [...first.records, ...second.records].map((r) => r.key).sort();
      expect(seen).toEqual(["a", "b", "c"]);
    } finally {
      await db.close();
    }
  });

  it("list honors metadata filters", async () => {
    const db = await VantaDb.connect(tmp("filter"));
    try {
      await db.putBatch([
        {
          namespace: "meta",
          key: "hit",
          payload: "match",
          metadata: { tag: { String: "keep" } },
        },
        {
          namespace: "meta",
          key: "miss",
          payload: "other",
          metadata: { tag: { String: "drop" } },
        },
      ]);

      const res = await db.list("meta", {
        filters: { tag: { String: "keep" } },
      });
      expect(res.records.map((r) => r.key)).toEqual(["hit"]);
    } finally {
      await db.close();
    }
  });

  it("listNamespaces reports every namespace holding records", async () => {
    const db = await VantaDb.connect(tmp("namespaces"));
    try {
      await db.putBatch([
        { namespace: "alpha", key: "k", payload: "v" },
        { namespace: "beta", key: "k", payload: "v" },
      ]);
      const ns = await db.listNamespaces();
      expect(ns).toEqual(expect.arrayContaining(["alpha", "beta"]));
    } finally {
      await db.close();
    }
  });

  // ── search ─────────────────────────────────────────────────────────────────

  it("hybrid search ranks the record matching both text and vector first", async () => {
    const db = await VantaDb.connect(tmp("hybrid"));
    try {
      await db.putBatch([
        {
          namespace: "docs",
          key: "rust-doc",
          payload: "rust systems programming language",
          vector: [0.9, 0.1],
        },
        {
          namespace: "docs",
          key: "py-doc",
          payload: "python scripting language",
          vector: [0.1, 0.9],
        },
      ]);

      const hits = await db.search({
        namespace: "docs",
        query_vector: [0.95, 0.05],
        text_query: "rust programming",
        top_k: 2,
      });

      expect(hits.length).toBeGreaterThan(0);
      expect(hits[0].record.key).toBe("rust-doc");
      expect(typeof hits[0].score).toBe("number");
    } finally {
      await db.close();
    }
  });

  it("search respects top_k", async () => {
    const db = await VantaDb.connect(tmp("topk"));
    try {
      await db.putBatch([
        { namespace: "vec", key: "a", payload: "a", vector: [1, 0] },
        { namespace: "vec", key: "b", payload: "b", vector: [0, 1] },
        { namespace: "vec", key: "c", payload: "c", vector: [1, 1] },
        { namespace: "vec", key: "d", payload: "d", vector: [-1, 0] },
      ]);
      const hits = await db.search({
        namespace: "vec",
        query_vector: [1, 0],
        top_k: 2,
      });
      expect(hits).toHaveLength(2);
    } finally {
      await db.close();
    }
  });

  it("search rejects a request missing query_vector", async () => {
    const db = await VantaDb.connect(tmp("searcherr"));
    try {
      await expect(
        // @ts-expect-error — exercising the runtime guard behind the type
        db.search({ namespace: "ns" }),
      ).rejects.toThrow(/missing required field `query_vector`/);
    } finally {
      await db.close();
    }
  });

  it("search rejects vectors above the max dimension cap", async () => {
    const db = await VantaDb.connect(tmp("dimcap"));
    try {
      await expect(
        db.search({
          namespace: "ns",
          query_vector: new Array(10_001).fill(0),
        }),
      ).rejects.toThrow(/exceeds max vector dimension/);
    } finally {
      await db.close();
    }
  });

  // ── explain_search ────────────────────────────────────────────────────────

  it("explainSearch returns route, per-hit breakdown, and optional fusion report", async () => {
    const db = await VantaDb.connect(tmp("explain"));
    try {
      await db.putBatch([
        {
          namespace: "explain",
          key: "doc-a",
          payload: "database engines store durable memory",
          vector: [1, 0],
        },
        {
          namespace: "explain",
          key: "doc-b",
          payload: "unrelated cooking recipe",
          vector: [0, 1],
        },
      ]);

      const expl = await db.explainSearch({
        namespace: "explain",
        query_vector: [1, 0],
        text_query: "database memory",
        top_k: 2,
      });

      expect(typeof expl.route).toBe("string");
      expect(expl.route.length).toBeGreaterThan(0);
      expect(Array.isArray(expl.hits)).toBe(true);

      const hit = expl.hits.find((h) => h.identity.includes("doc-a"));
      expect(hit).toBeDefined();
      expect(hit!.identity).toContain("explain"); // identity is `namespace\0key`
      expect(typeof hit!.score).toBe("number");
      expect(Array.isArray(hit!.matched_tokens)).toBe(true);
      expect(Array.isArray(hit!.bm25_terms)).toBe(true);
      for (const term of hit!.bm25_terms) {
        expect(typeof term.token).toBe("string");
        expect(typeof term.contribution).toBe("number");
      }

      if (expl.fusion_report !== null) {
        expect(typeof expl.fusion_report.rrf_k).toBe("number");
      }
    } finally {
      await db.close();
    }
  });

  // ── capabilities ───────────────────────────────────────────────────────────

  it("capabilities exposes stable runtime keys", async () => {
    const db = await VantaDb.connect(tmp("caps"));
    try {
      const caps = db.capabilities();
      expect(caps.persistence).toBe(true); // filesystem-backed dir
      expect(caps.read_only).toBe(false);
      expect(typeof caps.vector_search).toBe("boolean");
      expect(typeof caps.iql_queries).toBe("boolean");
      expect(["Enterprise", "Performance", "LowResource"]).toContain(
        caps.runtime_profile,
      );
    } finally {
      await db.close();
    }
  });

  // ── close-drain (OpGate durability barrier) ────────────────────────────────

  it("close() lets an in-flight put finish and persists it", async () => {
    const dir = tmp("drain");
    const db = await VantaDb.connect(dir);

    const inflight = db.put({ namespace: "drain", key: "k", payload: "survives" });
    // Let the operation register in the gate and run on the blocking pool.
    await new Promise((resolve) => setTimeout(resolve, 50));

    await db.close(); // drains any still-running op before flushing
    await inflight;   // resolved, never silently lost

    const reopened = await VantaDb.connect(dir);
    try {
      const got = await reopened.get("drain", "k");
      expect(got).not.toBeNull();
      expect(got!.payload).toBe("survives");
    } finally {
      await reopened.close();
    }
  });

  it("operations are rejected once close() has begun", async () => {
    const db = await VantaDb.connect(tmp("closed"));
    try {
      await db.put({ namespace: "ns", key: "k", payload: "v" });
    } finally {
      await db.close();
    }
    await expect(db.put({ namespace: "ns", key: "k2", payload: "v" })).rejects.toThrow(
      /database is closing/,
    );
    await expect(db.get("ns", "k")).rejects.toThrow(/database is closing/);
  });

  // ── graph api shapes ───────────────────────────────────────────────────────

  it("insertNode/getNode round-trips tagged fields and string ids", async () => {
    const db = await VantaDb.connect(tmp("graph"));
    try {
      await db.insertNode({
        id: "42",
        content: "ada",
        fields: { name: { String: "Ada" }, year: { Int: 1815 } },
      });

      const node = await db.getNode("42");
      expect(node).not.toBeNull();
      expect(node!.id).toBe("42"); // decimal string, not number
      expect(node!.fields.name).toEqual({ String: "Ada" });
      expect(node!.is_alive).toBe(true);
      expect(Array.isArray(node!.edges)).toBe(true);
    } finally {
      await db.close();
    }
  });

  it("graphBfs rejects an invalid direction", async () => {
    const db = await VantaDb.connect(tmp("dir"));
    try {
      await expect(
        db.graphBfs(["1"], 2, // @ts-expect-error — exercising the runtime guard
          "Sideways"),
      ).rejects.toThrow(/invalid direction/);
    } finally {
      await db.close();
    }
  });

  // ── BND-10 parity additions ──────────────────────────────────────────────

  it("versions returns empty array for a missing key", async () => {
    const db = await VantaDb.connect(tmp("versions"));
    try {
      const out = await db.versions("ns", "missing");
      expect(Array.isArray(out)).toBe(true);
      expect(out).toHaveLength(0);
    } finally {
      await db.close();
    }
  });

  it("supersede marks the old key as superseded", async () => {
    const db = await VantaDb.connect(tmp("supersede"));
    try {
      await db.put({ namespace: "ns", key: "old", payload: "first" });
      await db.put({ namespace: "ns", key: "new", payload: "second" });
      await db.supersede("ns", "old", "new");
      const versions = await db.versions("ns", "old");
      expect(versions.length).toBeGreaterThanOrEqual(1);
      const latest = versions[versions.length - 1];
      expect(latest.superseded_by).toBe("new");
      expect(typeof latest.superseded_at_ms).toBe("number");
    } finally {
      await db.close();
    }
  });

  it("supersede rejects when oldKey === newKey", async () => {
    const db = await VantaDb.connect(tmp("supersede-same"));
    try {
      await db.put({ namespace: "ns", key: "k", payload: "v" });
      await expect(db.supersede("ns", "k", "k")).rejects.toThrow();
    } finally {
      await db.close();
    }
  });

  it("compactWal succeeds and is idempotent", async () => {
    const db = await VantaDb.connect(tmp("compactWal"));
    try {
      await db.put({ namespace: "ns", key: "k", payload: "v" });
      await expect(db.compactWal()).resolves.toBeUndefined();
      await expect(db.compactWal()).resolves.toBeUndefined();
    } finally {
      await db.close();
    }
  });

  it("purgeExpired returns a number for an empty namespace", async () => {
    const db = await VantaDb.connect(tmp("purgeExpired"));
    try {
      await expect(db.purgeExpired()).resolves.toBe(0);
    } finally {
      await db.close();
    }
  });

  it("count returns 0 for an empty namespace", async () => {
    const db = await VantaDb.connect(tmp("count"));
    try {
      await expect(db.count("empty")).resolves.toBe(0);
      await db.put({ namespace: "ns", key: "k1", payload: "a" });
      await db.put({ namespace: "ns", key: "k2", payload: "b" });
      await expect(db.count("ns")).resolves.toBe(2);
      await expect(db.count("ns", null)).resolves.toBe(2);
    } finally {
      await db.close();
    }
  });

  it("deleteByFilter rejects an empty filter", async () => {
    const db = await VantaDb.connect(tmp("deleteByFilter"));
    try {
      await expect(
        db.deleteByFilter("ns", []),
      ).rejects.toThrow(/at least one item/);
    } finally {
      await db.close();
    }
  });

  it("searchWithMethod accepts 'Flat' and returns hits", async () => {
    const db = await VantaDb.connect(tmp("searchWithMethod"));
    try {
      await db.put({
        namespace: "ns",
        key: "k1",
        payload: "alpha",
        vector: [1, 0, 0],
      });
      const hits = await db.searchWithMethod(
        { namespace: "ns", query_vector: [1, 0, 0], top_k: 5 },
        "Flat",
      );
      expect(hits.length).toBeGreaterThanOrEqual(1);
    } finally {
      await db.close();
    }
  });

  it("searchWithMethod rejects an unknown method", async () => {
    const db = await VantaDb.connect(tmp("searchWithMethod-bad"));
    try {
      await expect(
        db.searchWithMethod(
          { namespace: "ns", query_vector: [1, 0], top_k: 1 },
          // @ts-expect-error — exercising the runtime guard
          "Unknown",
        ),
      ).rejects.toThrow(/invalid method/);
    } finally {
      await db.close();
    }
  });
});
