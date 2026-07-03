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
    ],
    links: [{ rel: "canonical", href: "https://vantadb.dev/engine" }],
  }),
});
