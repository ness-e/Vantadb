/**
 * Example: VantaDB + LangChain.js — Document RAG Pipeline
 *
 * Dependencies:
 *   npm install vantadb @langchain/core @langchain/community @langchain/openai
 *
 * Usage:
 *   export OPENAI_API_KEY=sk-...
 *   node --experimental-wasm-modules examples/langchain-rag.mjs
 */

import { VantaDB } from "vantadb";
import { Document } from "@langchain/core/documents";
import { OpenAIEmbeddings } from "@langchain/openai";
import { CharacterTextSplitter } from "@langchain/core/text_splitter";

const db = await VantaDB.create();
const NAMESPACE = "docs";

// Load documents
const rawDocs = [
  "VantaDB is an embedded vector database for AI agents.",
  "It supports HNSW indexing, BM25 full-text search, and GraphRAG.",
  "The TypeScript SDK runs in Node.js, Bun, Deno, and browsers via WASM.",
];

// Split into chunks
const splitter = new CharacterTextSplitter({ chunkSize: 100, chunkOverlap: 20 });
const splitDocs = await splitter.splitDocuments(
  rawDocs.map((text) => new Document({ pageContent: text }))
);

// Generate embeddings and store in VantaDB
const embeddings = new OpenAIEmbeddings({ model: "text-embedding-3-small" });

for (const doc of splitDocs) {
  const vector = await embeddings.embedQuery(doc.pageContent);
  await db.put({
    namespace: NAMESPACE,
    key: `doc:${crypto.randomUUID()}`,
    payload: doc.pageContent,
    vector,
    metadata: { source: "example" },
  });
}
console.log(`Indexed ${splitDocs.length} documents in VantaDB`);

// Search via VantaDB
const query = "How does VantaDB store vectors?";
const queryVector = await embeddings.embedQuery(query);

// Vector search via VantaDB
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

await db.close();
