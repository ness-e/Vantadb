import { describe, it, expect, afterAll } from "vitest";
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

import { VantaDb } from "../index.js";

/**
 * Graph + explain surface (BND-05) — parity with the WASM/TS SDK:
 * node CRUD (insertNode/getNode/deleteNode/addEdge/removeEdge), traversals
 * (graphBfs/graphDfs/graphTopologicalSort/graphIsDag/graphFilteredTraversal),
 * graphDegree and explainSearch.
 *
 * Node ids are u128 in the core SDK, so the Node API takes/returns ids as
 * decimal strings (JS numbers lose precision above 2^53).
 */
describe("vantadb-node graph + explain (native backend)", () => {
  const tempDirs: string[] = [];

  afterAll(() => {
    for (const dir of tempDirs) {
      rmSync(dir, { recursive: true, force: true });
    }
  });

  async function connect() {
    const dir = mkdtempSync(join(tmpdir(), "vantadb-node-graph-"));
    tempDirs.push(dir);
    return VantaDb.connect(dir);
  }

  it("insertNode → getNode round-trip (fields + vector + content)", async () => {
    const db = await connect();
    try {
      await db.insertNode({
        id: "900",
        content: "root node",
        vector: [0.1, 0.2, 0.3],
        fields: { name: { String: "root" } },
      });

      const node = await db.getNode("900");
      expect(node).not.toBeNull();
      expect(node!.id).toBe("900");
      expect(node!.fields.name).toEqual({ String: "root" });
      expect(node!.vector_dimensions).toBe(3);
      expect(node!.is_alive).toBe(true);

      // absent id → null
      expect(await db.getNode("99999")).toBeNull();
    } finally {
      await db.close();
    }
  });

  it("addEdge + traversals: bfs/dfs/topologicalSort/isDag/degree/filtered", async () => {
    const db = await connect();
    try {
      await db.insertNode({ id: "900", content: "a" });
      await db.insertNode({ id: "901", content: "b" });
      await db.insertNode({ id: "902", content: "c" });
      await db.addEdge("900", "901", "knows");
      await db.addEdge("900", "902", "knows", 0.5, 1000);

      // BFS from the root visits all three nodes
      const bfs = await db.graphBfs(["900"], 5, "Forward");
      expect(bfs).toContain("900");
      expect(bfs).toContain("901");
      expect(bfs).toContain("902");

      // DFS also visits all nodes
      const dfs = await db.graphDfs(["900"], 5, "Forward");
      expect(dfs).toContain("901");
      expect(dfs).toContain("902");

      // Tree is a DAG
      expect(await db.graphIsDag(["900"])).toBe(true);

      // Topological sort order is deterministic on a DAG: parents before children
      const sorted = await db.graphTopologicalSort(["900"]);
      expect(sorted[0]).toBe("900");
      expect(sorted.slice(1).sort()).toEqual(["901", "902"]);

      // Degree centrality: root has out_degree 2
      const degrees = await db.graphDegree(["900"]);
      const root = degrees.find((d: { id: string }) => d.id === "900");
      expect(root).toBeDefined();
      expect(root!.out_degree).toBe(2);

      // Filtered traversal: a non-matching label id blocks edge traversal
      const filteredNone = await db.graphFilteredTraversal(
        ["900"],
        5,
        "Forward",
        { labels: [999999] },
      );
      expect(filteredNone).toEqual(["900"]);

      // No filter (null) behaves like graphBfs
      const filteredAll = await db.graphFilteredTraversal(["900"], 5, "Forward", null);
      expect(filteredAll).toContain("901");

      // removeEdge prunes the graph
      await db.removeEdge("900", "902", "knows");
      const afterRemove = await db.graphBfs(["900"], 5, "Forward");
      expect(afterRemove).not.toContain("902");
      expect(afterRemove).toContain("901");
    } finally {
      await db.close();
    }
  });

  it("deleteNode removes a node", async () => {
    const db = await connect();
    try {
      await db.insertNode({ id: "42", content: "doomed" });
      expect(await db.getNode("42")).not.toBeNull();
      await db.deleteNode("42", "test cleanup");
      expect(await db.getNode("42")).toBeNull();
    } finally {
      await db.close();
    }
  });

  it("explainSearch returns route + hit breakdown", async () => {
    const db = await connect();
    try {
      await db.putBatch([
        {
          namespace: "exp",
          key: "alpha",
          payload: "the quick brown fox",
          vector: [1.0, 0.0, 0.0],
        },
        {
          namespace: "exp",
          key: "beta",
          payload: "jumps over the lazy dog",
          vector: [0.0, 1.0, 0.0],
        },
      ]);

      const explanation = await db.explainSearch({
        namespace: "exp",
        query_vector: [1.0, 0.0, 0.0],
        text_query: "quick",
        top_k: 5,
      });

      expect(explanation.route).toBe("hybrid");
      expect(Array.isArray(explanation.hits)).toBe(true);
      expect(explanation.hits.length).toBeGreaterThan(0);
      // Each explained hit carries the record identity and a score.
      expect(explanation.hits[0].identity).toBeTruthy();
      expect(typeof explanation.hits[0].score).toBe("number");
    } finally {
      await db.close();
    }
  });

  it("rejects invalid direction and non-u128 ids at the FFI boundary", async () => {
    const db = await connect();
    try {
      await db.insertNode({ id: "7", content: "n" });
      await expect(db.graphBfs(["7"], 5, "Diagonal")).rejects.toThrow(
        /invalid direction/,
      );
      await expect(db.graphBfs(["abc"], 5, "Forward")).rejects.toThrow(
        /invalid node id/,
      );
      await expect(db.insertNode({ id: "-1", content: "x" })).rejects.toThrow(
        /invalid node id/,
      );
      await expect(db.insertNode({ content: "missing id" })).rejects.toThrow(
        /missing required field `id`/,
      );
    } finally {
      await db.close();
    }
  });
});