import { VantaDB } from "vantadb";

async function main() {
  const db = await VantaDB.create();

  // Store memories with embeddings
  await db.put({
    namespace: "langchain",
    key: "doc1",
    payload: "VantaDB is a vector-graph memory database for AI agents.",
    metadata: { source: { type: "String", value: "docs" } },
    vector: [0.1, 0.2, 0.3, 0.4], // would come from an embedding model
  });

  // Search with semantic similarity
  const hits = await db.search({
    namespace: "langchain",
    query_vector: [0.15, 0.25, 0.35, 0.45],
    top_k: 5,
    explain: true,
  });

  for (const hit of hits) {
    console.log(`Score: ${hit.score.toFixed(4)}`);
    console.log(`Payload: ${hit.record.payload}`);
    console.log(`Explanation:`, hit.explanation);
  }

  // Use as a retriever: store context, query with vector
  await db.put({
    namespace: "conversation",
    key: "msg_001",
    payload: "User asked about vector databases",
    metadata: { role: { type: "String", value: "system" } },
    vector: [0.5, 0.1, 0.8, 0.2],
  });

  const memory = await db.get("conversation", "msg_001");
  console.log("Retrieved memory:", memory?.payload);

  db.close();
}

main().catch(console.error);
