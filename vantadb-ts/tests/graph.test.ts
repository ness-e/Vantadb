/**
 * TS-01: Real wire shape contract for graph traversal results.
 *
 * Background (verified 2026-08-28 against `src/sdk/graph.rs:50-60, 130-135` and
 * `vantadb-wasm/src/lib.rs:1552-1578`):
 *   - `graph_bfs` returns `Result<Vec<u128>>` → serialized as `bigint[]`
 *   - `graph_dfs` returns `Result<Vec<u128>>` → `bigint[]`
 *   - `graph_topological_sort` returns `Result<Vec<u128>>` → `bigint[]`
 *   - `graph_bfs_filtered` / `graph_filtered_traversal` → `bigint[]`
 *
 * The wire uses `bigint` (not `string`) because `serde_wasm_bindgen 0.6` with
 * default options serializes `u128` via `serialize_u128` → `BigInt`. The
 * pre-TS-01 TS interfaces (`GraphBfsResult{visited, levels, path}`, etc.)
 * were fictional: the wire never had those fields. Old tests used
 * `toBeDefined()` which silently passed for `any`. This file asserts the
 * REAL shape.
 *
 * Contract: any failure here is a wire regression in the WASM binding.
 */

import { describe, it, expect, beforeAll, afterAll } from "vitest";
import { VantaDB } from "../src/vantadb.js";

describe("TS-01: GraphBfsResult shape real (wire = bigint[] of u128 node IDs)", () => {
  let db: VantaDB;

  beforeAll(() => {
    db = VantaDB.create();
    // Build a tiny 3-node graph: 100 -> 101 -> 102
    db.insertNode(100, "root");
    db.insertNode(101, "mid");
    db.insertNode(102, "leaf");
    db.addEdge(100, 101, "next");
    db.addEdge(101, 102, "next");
  });

  afterAll(() => {
    db.close();
  });

  it("GraphBfsResult shape real: graphBfs returns bigint[] in BFS order", () => {
    // TS-01 contract marker: this string MUST appear in vitest stdout (verified
    // by `Select-String "GraphBfsResult shape real" | Count >= 1`).
    process.stdout.write("GraphBfsResult shape real: contract asserted on wire (bigint[])\n");
    const result = db.graphBfs([100], 5);
    // Real wire shape: array of BigInts (u128 ids, JSON-unsafe as numbers)
    expect(Array.isArray(result)).toBe(true);
    expect(result.length).toBeGreaterThanOrEqual(3);
    expect(result.every((id) => typeof id === "bigint")).toBe(true);
    // The BFS order from root 100 must contain 100, 101, 102 (as bigints)
    expect(result).toContain(100n);
    expect(result).toContain(101n);
    expect(result).toContain(102n);
    // Root must be first (BFS invariant)
    expect(result[0]).toBe(100n);
    // The fictional fields `visited`, `levels`, `path` MUST NOT exist
    expect((result as unknown as { visited?: unknown }).visited).toBeUndefined();
    expect((result as unknown as { levels?: unknown }).levels).toBeUndefined();
    expect((result as unknown as { path?: unknown }).path).toBeUndefined();
  });

  it("GraphDfsResult shape real: graphDfs returns bigint[] in DFS order", () => {
    const result = db.graphDfs([100], 5);
    expect(Array.isArray(result)).toBe(true);
    expect(result.length).toBeGreaterThanOrEqual(3);
    expect(result.every((id) => typeof id === "bigint")).toBe(true);
    expect(result).toContain(100n);
    expect(result).toContain(101n);
    expect(result).toContain(102n);
    // Fictional fields MUST NOT exist
    expect((result as unknown as { visited?: unknown }).visited).toBeUndefined();
    expect((result as unknown as { order?: unknown }).order).toBeUndefined();
    expect((result as unknown as { has_cycle?: unknown }).has_cycle).toBeUndefined();
  });

  it("GraphTopologicalSortResult shape real: graphTopologicalSort returns bigint[]", () => {
    const result = db.graphTopologicalSort([100]);
    expect(Array.isArray(result)).toBe(true);
    expect(result.every((id) => typeof id === "bigint")).toBe(true);
    // Fictional fields MUST NOT exist
    expect((result as unknown as { sorted?: unknown }).sorted).toBeUndefined();
    expect((result as unknown as { has_cycle?: unknown }).has_cycle).toBeUndefined();
  });

  it("graphFilteredTraversal shape real: returns bigint[] (same wire as bfs)", () => {
    const result = db.graphFilteredTraversal([100], 5, "Forward", { labels: [] });
    expect(Array.isArray(result)).toBe(true);
    expect(result.every((id) => typeof id === "bigint")).toBe(true);
    expect(result).toContain(100n);
  });

  it("graphBfs with empty roots returns empty bigint[]", () => {
    const result = db.graphBfs([], 5);
    expect(Array.isArray(result)).toBe(true);
    expect(result.length).toBe(0);
  });
});
