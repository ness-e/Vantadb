import { describe, it, expect, beforeAll, afterAll } from "vitest";
import { Client } from "../vantadb.js";
import { NativeVantaDB } from "../native.js";

// DIST-15: GraphRAG (`graphragSearch`) on the TS surfaces. The WASM Client and
// the raw `vantadb-node` binding run the same dataset and must produce the
// same `context_text` / node ids (cross-binding parity, same core pipeline).
// Node ids are deterministic (`memory_node_id(namespace, key)`), so the
// comparison is exact, not fuzzy.

const NAMESPACE = "graphrag-parity";
const QUERY = "vector database";
const RECORDS = [
  { key: "a", payload: "vector database for agents", vector: [0.1, 0.2, 0.3] },
  { key: "b", payload: "graph expansion uses edges", vector: [0.2, 0.3, 0.4] },
  { key: "c", payload: "BM25 lexical retrieval engine", vector: [0.3, 0.4, 0.5] },
];

const nativeAvailable = await import("vantadb-node").then(
  () => true,
  () => false,
);

function seedWasm(db: Client): void {
  const ids = RECORDS.map(
    (r) => db.put({ namespace: NAMESPACE, key: r.key, payload: r.payload, vector: r.vector }).node_id,
  );
  db.addEdge(BigInt(String(ids[0])), BigInt(String(ids[1])), "uses");
  db.addEdge(BigInt(String(ids[0])), BigInt(String(ids[2])), "uses");
}

describe("graphragSearch (DIST-15)", () => {
  describe("WASM Client", () => {
    let db: Client;

    beforeAll(() => {
      db = Client.create();
      seedWasm(db);
    });

    afterAll(() => {
      db.close();
    });

    it("returns the canonical wire shape", () => {
      const result = db.graphragSearch(NAMESPACE, QUERY);

      expect(Object.keys(result).sort()).toEqual(["context_text", "edges", "nodes", "stats"]);
      expect(result.nodes.length).toBeGreaterThan(0);

      const node = result.nodes[0];
      expect(typeof node.id).toBe("string");
      expect(node.id).toMatch(/^\d+$/);
      expect(typeof node.content).toBe("string");
      expect(typeof node.score).toBe("number");
      expect(typeof node.hop_distance).toBe("number");

      expect(result.context_text).toContain("## Relevant Nodes");
      expect(Object.keys(result.stats).sort()).toEqual([
        "expansion_hops_used",
        "nodes_expanded",
        "seeds_found",
        "total_candidates",
      ]);
      expect(result.stats.seeds_found).toBeGreaterThan(0);

      expect(result.edges.length).toBeGreaterThan(0);
      expect(typeof result.edges[0].source).toBe("string");
      expect(result.edges[0].source).toMatch(/^\d+$/);
      expect(typeof result.edges[0].label).toBe("string");
    });

    it("empty namespace returns an empty result", () => {
      const result = db.graphragSearch("missing-namespace", "anything");

      expect(result.nodes).toEqual([]);
      expect(result.edges).toEqual([]);
      expect(result.context_text).toBe("");
      expect(result.stats.seeds_found).toBe(0);
      expect(result.stats.nodes_expanded).toBe(0);
    });

    it.runIf(nativeAvailable)(
      "parity: same dataset in vantadb-node yields the same context_text and ids",
      async () => {
        const native = await import("vantadb-node");
        const nativeDb = await native.VantaDb.connect(":memory:");
        try {
          const ids: string[] = [];
          for (const r of RECORDS) {
            const rec = await nativeDb.put({
              namespace: NAMESPACE,
              key: r.key,
              payload: r.payload,
              vector: r.vector,
            });
            ids.push(String(rec.node_id));
          }
          await nativeDb.addEdge(ids[0], ids[1], "uses", undefined, undefined);
          await nativeDb.addEdge(ids[0], ids[2], "uses", undefined, undefined);

          const nativeResult = await nativeDb.graphragSearch(NAMESPACE, QUERY, undefined);
          const wasmResult = db.graphragSearch(NAMESPACE, QUERY);

          expect(nativeResult.context_text).toBe(wasmResult.context_text);
          expect(nativeResult.nodes.map((n) => n.id)).toEqual(wasmResult.nodes.map((n) => n.id));
        } finally {
          await nativeDb.close();
        }
      },
    );
  });

  describe("NativeVantaDB wrapper", () => {
    it.runIf(nativeAvailable)("exposes graphragSearch with the same shape", async () => {
      const db = await NativeVantaDB.connect(":memory:");
      try {
        for (const r of RECORDS) {
          await db.put({ namespace: NAMESPACE, key: r.key, payload: r.payload, vector: r.vector });
        }
        const result = await db.graphragSearch(NAMESPACE, QUERY);
        expect(Object.keys(result).sort()).toEqual(["context_text", "edges", "nodes", "stats"]);
        expect(result.nodes.length).toBeGreaterThan(0);
        expect(result.context_text).toContain("## Relevant Nodes");
      } finally {
        await db.close();
      }
    });
  });
});
