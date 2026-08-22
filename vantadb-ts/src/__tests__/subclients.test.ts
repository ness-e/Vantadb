import { describe, it, expect, beforeAll, afterAll } from "vitest";
import { VantaDB, VantaError } from "../vantadb.js";

// ---------------------------------------------------------------------------
// Sub-clients (SDKB-02) — domain-grouped views over the flat methods.
// Contract: db.<client>.x(...) === db.x(...) (same result, same signature).
// Pure delegation only (D43); domain map: docs/api/BINDINGS_NAMESPACES.md.
//
// Deferred per D43/D42 and NOT tested here because the TS surface does not
// expose them: `supersede` (Python-only), `recover_archived_nodes` (wiki,
// Python-only). `db.wiki` exists but is intentionally empty in v1.
// ---------------------------------------------------------------------------

describe("Sub-client shape", () => {
  it("getters return frozen objects", () => {
    const db = VantaDB.create();
    expect(Object.isFrozen(db.memory)).toBe(true);
    expect(Object.isFrozen(db.graph)).toBe(true);
    expect(Object.isFrozen(db.wiki)).toBe(true);
    expect(Object.isFrozen(db.system)).toBe(true);
    db.close();
  });

  it("getters are memoized (same object across accesses)", () => {
    const db = VantaDB.create();
    expect(db.memory).toBe(db.memory);
    expect(db.graph).toBe(db.graph);
    expect(db.system).toBe(db.system);
    db.close();
  });

  it("wiki is empty in v1 (D43: wiki features are core-only)", () => {
    const db = VantaDB.create();
    expect(Object.keys(db.wiki)).toEqual([]);
    db.close();
  });
});

describe("db.memory delegates to flat memory methods", () => {
  let db: VantaDB;

  beforeAll(() => { db = VantaDB.create(); });
  afterAll(() => { db.close(); });

  it("memory.put returns the identical record as flat put", () => {
    const viaFlat = db.put({ namespace: "sc_mem", key: "flat", payload: "v" });
    const viaClient = db.memory.put({ namespace: "sc_mem", key: "client", payload: "v" });
    expect(viaClient.namespace).toBe(viaFlat.namespace);
    expect(viaClient.payload).toBe("v");
    expect(typeof viaClient.version).toBe("string");
  });

  it("memory.get retrieves what memory.put stored (round-trip)", () => {
    const stored = db.memory.put({ namespace: "sc_rt", key: "k1", payload: "hello" });
    const fetched = db.memory.get("sc_rt", "k1");
    expect(fetched).not.toBeNull();
    expect(fetched!.node_id).toBe(stored.node_id);
    expect(fetched!.payload).toBe("hello");
  });

  it("memory.get returns null for missing record (same as flat get)", () => {
    expect(db.memory.get("nope", "nope")).toBeNull();
  });

  it("memory.search returns hits ordered by distance (hybrid request)", () => {
    db.memory.put({ namespace: "sc_search", key: "a", payload: "apple", vector: [1, 0, 0, 0] });
    db.memory.put({ namespace: "sc_search", key: "b", payload: "banana", vector: [0, 1, 0, 0] });
    const hits = db.memory.search({
      namespace: "sc_search",
      query_vector: [1, 0, 0, 0],
      top_k: 2,
    });
    expect(hits.length).toBeGreaterThan(0);
    for (let i = 1; i < hits.length; i++) {
      expect(hits[i - 1].distance).toBeGreaterThanOrEqual(hits[i].distance);
    }
    // Same result as the flat method on the same data.
    const flatHits = db.search({ namespace: "sc_search", query_vector: [1, 0, 0, 0], top_k: 2 });
    expect(hits.map((h) => h.record.key)).toEqual(flatHits.map((h) => h.record.key));
  });

  it("optional arguments pass through (list without options)", () => {
    db.memory.put({ namespace: "sc_list", key: "x", payload: "y" });
    const page = db.memory.list("sc_list");
    expect(page.records.length).toBeGreaterThanOrEqual(1);
  });

  it("operations after close() throw VantaError CLOSED via sub-client too", () => {
    const tmp = VantaDB.create();
    tmp.close();
    expect(() => tmp.memory.put({ namespace: "n", key: "k", payload: "p" })).toThrow(/closed/i);
    expect(() => tmp.memory.list("n")).toThrow();
    expect(() => tmp.graph.bfs([1])).toThrow();
  });
});

describe("db.graph delegates to flat graph methods", () => {
  let db: VantaDB;

  beforeAll(() => { db = VantaDB.create(); });
  afterAll(() => { db.close(); });

  it("graph.insertNode + getNode round-trip", () => {
    db.graph.insertNode(900, "sub-client node");
    const node = db.graph.getNode(900);
    expect(node).not.toBeNull();
    expect(node!.id).toBeDefined();
  });

  it("graph.bfs matches flat graphBfs result (this-binding check)", () => {
    // this-binding risk from pre-mortem: call the arrow while `this` would
    // drift if it were a plain method extracted from the object.
    const { bfs } = db.graph;
    const result = bfs.call(db.graph, [900], 5);
    expect(result).toBeDefined();

    db.graph.insertNode(901, "child");
    db.graph.addEdge(900, 901, "link");
    expect(bfs([900], 5)).toEqual(db.graphBfs([900], 5));
  });

  it("graph.topologicalSort works on a DAG (same as flat)", () => {
    db.graph.insertNode(910, "gp");
    db.graph.insertNode(911, "p");
    db.graph.insertNode(912, "c");
    db.graph.addEdge(910, 911, "parent_of");
    db.graph.addEdge(911, 912, "parent_of");

    const viaClient = db.graph.topologicalSort([910]);
    const viaFlat = db.graphTopologicalSort([910]);
    // Delegation identity is the contract (SDKB-02); the exact result shape
    // has a known drift between types.ts and the current pkg build and is
    // owned by the flat API, not by sub-clients.
    expect(viaClient).toEqual(viaFlat);
    expect(viaClient).toBeDefined();
  });

  it("graph.isDag returns true for a tree (same as flat)", () => {
    expect(db.graph.isDag([910])).toBe(db.graphIsDag([910]));
    expect(db.graph.isDag([910])).toBe(true);
  });

  it("graph.degree and filteredTraversal delegate", () => {
    // Compare ids+degrees structurally (array order is not guaranteed across
    // separate engine calls).
    const idsOf = (entries: { id: string; in_degree: number; out_degree: number }[]) =>
      entries.map((d) => `${d.id}:${d.in_degree}:${d.out_degree}`).sort();
    expect(idsOf(db.graph.degree([910]))).toEqual(idsOf(db.graphDegree([910])));

    const traversal = db.graph.filteredTraversal([910], 3, "Forward", null);
    expect(traversal).toBeDefined();
  });
});

describe("db.system delegates to flat system methods", () => {
  let db: VantaDB;

  beforeAll(() => { db = VantaDB.create(); });
  afterAll(() => { db.close(); });

  it("system.capabilities returns the same shape as flat capabilities", () => {
    const caps = db.system.capabilities();
    expect(caps.vector_search).toBe(true);
    expect(caps).toEqual(db.capabilities());
  });

  it("system.query delegates with identical error surface as flat query", () => {
    // The local pkg build lacks the experimental-lisp extension, so LISP IQL
    // queries throw. Delegation contract: sub-client and flat method produce
    // the exact same error (message included).
    const runFlat = () => db.query("(entity :id 1)");
    expect(runFlat).toThrow(VantaError);
    const runClient = () => db.system.query("(entity :id 1)");
    expect(runClient).toThrow(VantaError);
    let flatMsg = "";
    let clientMsg = "";
    try { runFlat(); } catch (e) { flatMsg = (e as VantaError).message; }
    try { runClient(); } catch (e) { clientMsg = (e as VantaError).message; }
    expect(clientMsg).toBe(flatMsg);
  });

  it("system.operationalMetrics returns expected fields", () => {
    const m = db.system.operationalMetrics();
    expect(typeof m.hnsw_nodes_count).toBe("string");
  });
});
