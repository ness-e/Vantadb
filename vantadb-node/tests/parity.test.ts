import { describe, it, expect } from "vitest";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";

import { VantaDb } from "../index.js";

// DIST-17 — Node (napi) producer for the cross-language conformance scenario.
//
// Runs the canonical scenario from `tests/parity/scenario.json` (shared by the
// Py/Node/WASM producers) against the native napi binding and writes the Node
// projection to `target/bindings-parity/node.json`. The cross-language
// comparison itself is `dev-tools/parity-compare.mjs` (canonical hash + diff).
//
// The Node binding exposes the full common surface including IQL (`query`,
// FIND-268). `excluded` is empty; every step runs against the native binding.
//
// Wall-clock fields (created_at_ms, updated_at_ms, valid_at_ms, last_accessed)
// are intentionally NOT part of the projection. Node ids and scores are; ids
// travel as decimal strings (u128 > 2^53, API-01).

interface Scenario {
  scenario: string;
  namespace: string;
  records: Array<{ key: string; payload: string; vector: number[] }>;
  search: { query_vector: number[]; text_query: string; top_k: number };
  graph: {
    root_key: string;
    max_depth: number;
    direction: "Forward" | "Reverse" | "Both";
    edges: Array<{ source_key: string; target_key: string; label: string }>;
  };
  iql: { write: string; read: string };
}

const SCENARIO_URL = new URL("../../tests/parity/scenario.json", import.meta.url);
const ARTIFACT_URL = new URL("../../target/bindings-parity/node.json", import.meta.url);

describe("cross-language conformance — Node producer (DIST-17)", () => {
  it("runs the canonical scenario and writes node.json", async () => {
    const scenario = JSON.parse(readFileSync(SCENARIO_URL, "utf-8")) as Scenario;
    const db = await VantaDb.connect(":memory:");
    try {
      const ids: Record<string, string> = {};
      const put: Array<{ key: string; node_id: string }> = [];
      for (const rec of scenario.records) {
        const record = await db.put({
          namespace: scenario.namespace,
          key: rec.key,
          payload: rec.payload,
          vector: rec.vector,
        });
        ids[rec.key] = String(record.node_id);
        put.push({ key: rec.key, node_id: String(record.node_id) });
      }

      const hits = await db.search({
        namespace: scenario.namespace,
        query_vector: scenario.search.query_vector,
        text_query: scenario.search.text_query,
        top_k: scenario.search.top_k,
      });
      const search = hits.map((hit) => ({ id: String(hit.record.node_id), score: hit.score }));

      for (const edge of scenario.graph.edges) {
        await db.addEdge(
          ids[edge.source_key],
          ids[edge.target_key],
          edge.label,
          undefined,
          undefined,
        );
      }
      const bfs = await db.graphBfs(
        [ids[scenario.graph.root_key]],
        scenario.graph.max_depth,
        scenario.graph.direction,
      );

      const iqlWrite = await db.query(scenario.iql.write);
      const iqlRead = await db.query(scenario.iql.read);
      if (!iqlWrite.Write?.node_id) {
        throw new Error("IQL write result is missing node_id");
      }
      const iql = {
        write_node_id: String(iqlWrite.Write.node_id),
        read_ids: (iqlRead.Read ?? []).map((node) => String(node.id)).sort(),
      };

      // Sanity: the projection must be complete before it is written — an
      // empty or partial projection would otherwise pass the comparator
      // vacuously.
      const expectedIds = new Set(put.map((row) => row.node_id));
      expect(put).toHaveLength(scenario.records.length);
      expect(expectedIds.size).toBe(put.length);
      expect(search.length).toBeGreaterThanOrEqual(2);
      expect(search.every((row) => expectedIds.has(row.id) && row.score > 0)).toBe(true);
      expect(bfs[0]).toBe(put[0].node_id);
      expect(new Set(bfs)).toEqual(expectedIds);
      expect(iql.read_ids).toEqual(["42424242"]);

      mkdirSync(fileURLToPath(new URL("../../target/bindings-parity/", import.meta.url)), {
        recursive: true,
      });
      writeFileSync(
        ARTIFACT_URL,
        JSON.stringify(
          {
            binding: "node",
            scenario: scenario.scenario,
            excluded: [],
            steps: { put, search, graph_bfs: bfs, iql },
          },
          null,
          2,
        ) + "\n",
      );
    } finally {
      await db.close();
    }
  });
});
