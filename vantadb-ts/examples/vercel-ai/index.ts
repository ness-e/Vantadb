import { VantaDB } from "vantadb";

// Simulated Vercel AI SDK tool integration
// In production, embeddings would come from an actual model provider

async function main() {
  const db = await VantaDB.create();

  // Seed contextual memories (as an AI tool would)
  await db.putBatch([
    {
      namespace: "context",
      key: "kb_weather",
      payload: "The weather API returns forecasts up to 14 days.",
      vector: [0.4, 0.2, 0.9, 0.1],
    },
    {
      namespace: "context",
      key: "kb_calendar",
      payload: "Calendar events can be created, updated, or deleted.",
      vector: [0.1, 0.9, 0.3, 0.5],
    },
    {
      namespace: "context",
      key: "kb_email",
      payload: "Email service supports send, read, and search operations.",
      vector: [0.7, 0.1, 0.2, 0.8],
    },
  ]);

  // Simulate a tool call: find relevant context for user query
  const userQuery = "Can you check my schedule for tomorrow?";
  const embedding = [0.15, 0.85, 0.25, 0.45]; // simulated embedding

  const results = await db.search({
    namespace: "context",
    query_vector: embedding,
    text_query: "schedule calendar",
    top_k: 3,
    explain: true,
  });

  console.log(`Query: "${userQuery}"`);
  console.log("Relevant context:");
  for (const r of results) {
    console.log(`  [${r.score.toFixed(3)}] ${r.record.payload}`);
  }

  // Retrieve a specific memory for tool output
  const memory = await db.get("context", "kb_calendar");
  if (memory) {
    console.log(`\nUsing tool knowledge: ${memory.payload}`);
  }

  db.close();
}

main().catch(console.error);
