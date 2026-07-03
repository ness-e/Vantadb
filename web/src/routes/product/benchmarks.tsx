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
    ],
    links: [{ rel: "canonical", href: "https://vantadb.dev/product/benchmarks" }],
  }),
});
