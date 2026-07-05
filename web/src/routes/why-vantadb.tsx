import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/why-vantadb")({
  head: () => ({
    meta: [
      { title: "Why VantaDB — Zero-Infrastructure Vector Search" },
      {
        name: "description",
        content:
          "Why teams choose VantaDB: embeddable Rust binary, sub-millisecond hybrid search, zero servers, Apache 2.0. Built for AI agents, local RAG, and edge intelligence.",
      },
      { property: "og:title", content: "Why VantaDB" },
      {
        property: "og:description",
        content: "Zero infrastructure, sub-millisecond hybrid search.",
      },
      { property: "og:url", content: "https://vantadb.dev/why-vantadb" },
    ],
    links: [{ rel: "canonical", href: "https://vantadb.dev/why-vantadb" }],
  }),
});
