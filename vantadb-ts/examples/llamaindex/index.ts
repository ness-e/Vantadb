import { VantaDB } from "vantadb";

async function main() {
  const db = await VantaDB.create();

  // Index documents as memory records
  await db.putBatch([
    {
      namespace: "documents",
      key: "paper_001",
      payload: "Graph neural networks excel at node classification tasks.",
      metadata: { category: { type: "String", value: "AI" } },
      vector: [0.9, 0.1, 0.7, 0.3],
    },
    {
      namespace: "documents",
      key: "paper_002",
      payload: "Vector databases enable efficient similarity search at scale.",
      metadata: { category: { type: "String", value: "databases" } },
      vector: [0.2, 0.8, 0.4, 0.6],
    },
  ]);

  // Query with vector search (simulating an embedding query)
  const results = await db.search({
    namespace: "documents",
    query_vector: [0.3, 0.7, 0.5, 0.5],
    top_k: 3,
  });

  console.log(`Found ${results.length} relevant documents`);
  for (const r of results) {
    console.log(`  [${r.score.toFixed(3)}] ${r.record.payload}`);
  }

  // Graph operations: link related nodes
  await db.insertNode(1, "Graph Neural Networks", [0.9, 0.1, 0.7, 0.3], {});
  await db.insertNode(2, "Vector Databases", [0.2, 0.8, 0.4, 0.6], {});
  await db.addEdge(1, 2, "related_to", 0.8);

  const bfs = await db.graphBfs([1], 2);
  console.log("Graph BFS from node 1:", JSON.stringify(bfs, null, 2));

  db.close();
}

main().catch(console.error);
