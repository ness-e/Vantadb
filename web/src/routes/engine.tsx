import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/engine")({
  head: () => ({
    meta: [
      { title: "VantaDB — Core Engine Modalities" },
      {
        name: "description",
        content:
          "Deep dive into the VantaDB engine: BM25 + HNSW Hybrid Search, local graph edges, and WAL durability.",
      },
      { property: "og:title", content: "VantaDB — Core Engine Modalities" },
      {
        property: "og:description",
        content: "Four search modalities in one atomic Rust contract.",
      },
      { property: "og:url", content: "https://vantadb.dev/engine" },
    ],
    links: [{ rel: "canonical", href: "https://vantadb.dev/engine" }],
    scripts: [
      {
        type: "application/ld+json",
        children: JSON.stringify({
          "@context": "https://schema.org",
          "@type": "WebApplication",
          name: "VantaDB Engine",
          applicationCategory: "DeveloperApplication",
          browserRequirements: "Requires JavaScript",
        }),
      },
    ],
  }),
});
