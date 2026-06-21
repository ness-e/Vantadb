/**
 * Example: VantaDB + Vercel AI SDK — Conversational Memory
 *
 * Dependencies:
 *   npm install vantadb @ai-sdk/openai ai
 *
 * Usage:
 *   export OPENAI_API_KEY=sk-...
 *   node --experimental-wasm-modules examples/vercel-ai-memory.mjs
 */

import { VantaDB } from "vantadb";
import { openai } from "@ai-sdk/openai";
import { streamText, tool } from "ai";
import { z } from "zod";

const db = await VantaDB.create();
const NAMESPACE = "chat_history";
const SESSION_ID = "session_001";

// Store every user message with its embedding
async function rememberMessage(userId, message) {
  // In production, use an embedding model (e.g. @ai-sdk/openai embed)
  const dummyVector = new Array(128).fill(0).map(() => Math.random() - 0.5);
  await db.put({
    namespace: NAMESPACE,
    key: `${userId}:${Date.now()}`,
    payload: message,
    vector: dummyVector,
    metadata: { userId, sessionId: SESSION_ID, timestamp: String(Date.now()) },
  });
}

// Retrieve semantically similar past messages
async function recallRelevant(userId, query, topK = 5) {
  const dummyVector = new Array(128).fill(0).map(() => Math.random() - 0.5);
  const results = await db.search({
    namespace: NAMESPACE,
    query_vector: dummyVector,
    top_k: topK,
  });
  return results.map((h) => h.record);
}

// Example: LLM tool that stores + retrieves from VantaDB
const memoryTool = tool({
  description: "Store or retrieve conversation memories",
  parameters: z.object({
    action: z.enum(["store", "recall"]),
    content: z.string().optional(),
  }),
  execute: async ({ action, content }) => {
    if (action === "store" && content) {
      await rememberMessage(SESSION_ID, content);
      return "Stored in long-term memory.";
    }
    if (action === "recall") {
      const memories = await recallRelevant(SESSION_ID, content || "");
      return JSON.stringify(memories.map((m) => m.payload));
    }
    return "No action taken.";
  },
});

// Stream a response with memory-aware context
const result = streamText({
  model: openai("gpt-4o-mini"),
  messages: [
    { role: "system", content: "You have memory tools. Use them wisely." },
    { role: "user", content: "Remember that I like Rust programming." },
  ],
  tools: { memory: memoryTool },
  maxSteps: 5,
});

for await (const chunk of result.textStream) {
  process.stdout.write(chunk);
}
console.log("\n--- Vercel AI SDK + VantaDB example complete ---");

await db.close();
