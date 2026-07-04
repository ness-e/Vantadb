import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/solutions/ai-agents")({
  head: () => ({
    meta: [
      { title: "VantaDB — AI Agent Memory & State" },
      {
        name: "description",
        content:
          "Persistent memory for AI agents. Store conversation history, tool call results, and agent state in an embedded vector database.",
      },      { property: "og:title", content: "VantaDB — AI Agent Memory & State" },
      {
        property: "og:description",
        content: "Persistent memory for AI agents.",
      },
      { property: "og:url", content: "https://vantadb.dev/solutions/ai-agents" },
    ],
    links: [{ rel: "canonical", href: "https://vantadb.dev/solutions/ai-agents" }],
  }),
});



