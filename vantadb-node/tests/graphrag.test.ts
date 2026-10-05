import { describe, it, expect } from "vitest";
import { VantaDb } from "../index.js";

// DIST-15: GraphRAG (`graphragSearch`) on the native Node binding. The wire
// shape is canonical across Py/TS/Node/WASM; ids travel as decimal strings
// (u128 > Number.MAX_SAFE_INTEGER, API-01).

const NAMESPACE = "graphrag";
const RECORDS = [
  { key: "a", payload: "vector database for agents", vector: [0.1, 0.2, 0.3] },
  { key: "b", payload: "graph expansion uses edges", vector: [0.2, 0.3, 0.4] },
  { key: "c", payload: "BM25 lexical retrieval engine", vector: [0.3, 0.4, 0.5] },
];

async function seed(db: VantaDb): Promise<string[]> {
  const ids: string[] = [];
  for (const r of RECORDS) {
    const rec = await db.put({
      namespace: NAMESPACE,
      key: r.key,
      payload: r.payload,
      vector: r.vector,
    });
    ids.push(String(rec.node_id));
  }
  await db.addEdge(ids[0], ids[1], "uses", undefined, undefined);
  await db.addEdge(ids[0], ids[2], "uses", undefined, undefined);
  return ids;
}

describe("vantadb-node graphragSearch (DIST-15)", () => {
  it("returns the canonical wire shape", async () => {
    const db = await VantaDb.connect(":memory:");
    try {
      await seed(db);

      const result = await db.graphragSearch(NAMESPACE, "vector database", undefined);

      expect(Object.keys(result).sort()).toEqual(["context_text", "edges", "nodes", "stats"]);
      expect(result.nodes.length).toBeGreaterThan(0);
      expect(typeof result.nodes[0].id).toBe("string");
      expect(result.nodes[0].id).toMatch(/^\d+$/);
      expect(typeof result.nodes[0].score).toBe("number");
      expect(result.context_text).toContain("## Relevant Nodes");
      expect(Object.keys(result.stats).sort()).toEqual([
        "expansion_hops_used",
        "nodes_expanded",
        "seeds_found",
        "total_candidates",
      ]);
      expect(result.stats.seeds_found).toBeGreaterThan(0);
      expect(result.edges.length).toBeGreaterThan(0);
      expect(result.edges[0].source).toMatch(/^\d+$/);
    } finally {
      await db.close();
    }
  });

  it("empty namespace returns an empty result", async () => {
    const db = await VantaDb.connect(":memory:");
    try {
      const result = await db.graphragSearch("missing-namespace", "anything", undefined);

      expect(result.nodes).toEqual([]);
      expect(result.edges).toEqual([]);
      expect(result.context_text).toBe("");
      expect(result.stats.seeds_found).toBe(0);
    } finally {
      await db.close();
    }
  });

  it("rejects an oversized query vector", async () => {
    const db = await VantaDb.connect(":memory:");
    try {
      await expect(
        db.graphragSearch(NAMESPACE, undefined, new Array(10_001).fill(0)),
      ).rejects.toThrow(/max vector dimension/);
    } finally {
      await db.close();
    }
  });
});
