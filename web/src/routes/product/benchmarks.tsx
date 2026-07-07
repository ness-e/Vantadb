import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/product/benchmarks")({
  head: () => ({
    meta: [
      { title: "VantaDB — Benchmarks & Performance Metrics" },
      {
        name: "description",
        content:
          "Performance benchmarks for VantaDB: HNSW vector search, BM25 full-text, hybrid RRF queries, WAL durability, and competitive analysis.",
      },
      { property: "og:title", content: "VantaDB — Benchmarks & Performance" },
      {
        property: "og:description",
        content: "Sub-millisecond query latency and HNSW recall benchmarks.",
      },
      { property: "og:url", content: "https://vantadb.dev/product/benchmarks" },
    ],
    links: [{ rel: "canonical", href: "https://vantadb.dev/product/benchmarks" }],
    scripts: [
      {
        type: "application/ld+json",
        children: JSON.stringify({
          "@context": "https://schema.org",
          "@type": "WebPage",
          name: "VantaDB — Benchmarks & Performance Metrics",
          description:
            "Performance benchmarks for VantaDB: HNSW vector search, BM25 full-text, hybrid RRF queries, WAL durability, and competitive analysis.",
        }),
      },
    ],
  }),
});
