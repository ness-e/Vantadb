// Simple integration test runner for VantaDB TS SDK
// Usage: node --experimental-wasm-modules test-runner.mjs

import { VantaDB } from "./dist/vantadb.js";

let passed = 0;
let failed = 0;

function assert(condition, msg) {
  if (condition) {
    passed++;
    console.log(`  \u2705 ${msg}`);
  } else {
    failed++;
    console.error(`  \u274c ${msg}`);
  }
}

async function test(name, fn) {
  console.log(`\n\uD83E\uDDEA ${name}`);
  try {
    await fn();
  } catch (e) {
    failed++;
    console.error(`  \u274c Error: ${e.message}`);
  }
}

async function main() {
  const db = VantaDB.create();

  await test("put and get", async () => {
    const r = await db.put({ namespace: "t", key: "k1", payload: "v1" });
    assert(r.namespace === "t", "namespace matches");
    assert(r.key === "k1", "key matches");
    assert(r.payload === "v1", "payload matches");
    assert(typeof r.version === "string", "version is string");

    const g = await db.get("t", "k1");
    assert(g !== null, "get returns record");
    assert(g.payload === "v1", "get returns correct payload");
  });

  await test("delete", async () => {
    await db.put({ namespace: "t", key: "del", payload: "gone" });
    const d = await db.delete("t", "del");
    assert(d === true, "delete returns true");
    const g = await db.get("t", "del");
    assert(g === null, "get returns null after delete");
  });

  await test("list namespaces", async () => {
    const ns = await db.listNamespaces();
    assert(ns.includes("t"), "list includes namespace");
  });

  await test("list with pagination", async () => {
    for (let i = 0; i < 5; i++) {
      await db.put({ namespace: "list", key: `k${i}`, payload: `v${i}` });
    }
    const page = await db.list("list", { limit: 2 });
    assert(page.records.length === 2, "list returns limited results");
    assert(page.next_cursor !== undefined, "list has cursor");
  });

  await test("vector search", async () => {
    const vec = [0.1, 0.2, 0.3, 0.4];
    await db.put({ namespace: "vec", key: "a", payload: "alpha", vector: vec });
    const hits = await db.search({ namespace: "vec", query_vector: vec, top_k: 5 });
    assert(hits.length > 0, "search returns results");
    assert(hits[0].score > 0.99, "search score is high");
  });

  await test("put batch", async () => {
    const rs = await db.putBatch([
      { namespace: "b", key: "a", payload: "1" },
      { namespace: "b", key: "b", payload: "2" },
    ]);
    assert(rs.length === 2, "batch returns 2 records");
    assert(rs[0].payload === "1", "batch record 1 correct");
    assert(rs[1].payload === "2", "batch record 2 correct");
  });

  await test("graph operations", async () => {
    await db.insertNode(1, "root");
    await db.insertNode(2, "child");
    await db.addEdge(1, 2, "knows", 0.8);

    const node = await db.getNode(1);
    assert(node !== null, "get node returns node");
    assert(node.edges.length === 1, "node has 1 edge");

    const dag = await db.graphIsDag([1]);
    assert(dag === true, "graph is DAG");
  });

  await test("capabilities", async () => {
    const caps = db.capabilities();
    assert(caps.vector_search === true, "vector_search enabled");
    assert(caps.iql_queries === true, "iql enabled");
  });

  await test("operational metrics", async () => {
    const m = await db.operationalMetrics();
    assert(typeof m.startup_ms === "string", "startup_ms is string");
    assert(typeof m.hnsw_nodes_count === "string", "hnsw_nodes_count is string");
  });

  await test("flush and compact", async () => {
    await db.flush();
    await db.compactWal();
    assert(true, "flush and compact succeed");
  });

  await test("generate snippet", async () => {
    const s = await db.generateSnippet("VantaDB is a vector DB", "vector", true);
    assert(s !== undefined, "snippet is defined");
    assert(s.includes("vector"), "snippet contains query word");
  });

  db.close();

  console.log(`\n${"=".repeat(40)}`);
  console.log(`Results: ${passed} passed, ${failed} failed`);
  process.exit(failed > 0 ? 1 : 0);
}

main();
