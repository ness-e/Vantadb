/**
 * Example: VantaDB + LlamaIndex.TS — Index and Query Documents
 *
 * Dependencies:
 *   npm install vantadb llamaindex
 *
 * Usage:
 *   export OPENAI_API_KEY=sk-...
 *   node --experimental-wasm-modules examples/llamaindex-rag.mjs
 */

import { VantaDB } from "vantadb";
import { OpenAIEmbedding } from "llamaindex";

const db = await VantaDB.create();
const NAMESPACE = "knowledge";

const embedModel = new OpenAIEmbedding({ model: "text-embedding-3-small" });

// Index documents
const documents = [
  { id: "doc1", text: "VantaDB provides HNSW-based vector search with cosine similarity." },
  { id: "doc2", text: "BM25 full-text search is built-in for hybrid retrieval fusion via RRF." },
  { id: "doc3", text: "GraphRAG support enables multi-hop reasoning across connected nodes." },
];

for (const doc of documents) {
  const [vector] = await embedModel.getTextEmbeddings([doc.text]);
  await db.put({
    namespace: NAMESPACE,
    key: doc.id,
    payload: doc.text,
    vector,
    metadata: { source: "llamaindex" },
  });
}
console.log(`Indexed ${documents.length} documents`);

// Query
const query = "What search methods does VantaDB support?";
const [queryVector] = await embedModel.getTextEmbeddings([query]);

const results = await db.search({
  namespace: NAMESPACE,
  query_vector: queryVector,
  text_query: query,
  top_k: 3,
});

console.log(`\nQuery: "${query}"`);
console.log("Results:");
for (const hit of results) {
  console.log(`  [score=${hit.score.toFixed(4)}] ${hit.record.payload}`);
}

// Example: iterate all documents in a namespace
const page = await db.list(NAMESPACE, { limit: 10 });
console.log(`\nTotal records in namespace: ${page.records.length}`);

await db.close();
