import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/use-cases")({
  head: () => ({
    meta: [
      { title: "VantaDB — Persistent Memory Use Cases" },
      {
        name: "description",
        content:
          "8 production patterns for AI agent memory, local-first RAG, codebase intelligence, multi-agent orchestration, semantic search, edge IoT, healthcare RAG, and financial document processing.",
      },
      { property: "og:title", content: "VantaDB — Use Cases & Production Patterns" },
      {
        property: "og:description",
        content: "Eight production-tested patterns for embedded vector search.",
      },
      { property: "og:url", content: "https://vantadb.dev/use-cases" },
    ],
    links: [{ rel: "canonical", href: "https://vantadb.dev/use-cases" }],
  }),
});
