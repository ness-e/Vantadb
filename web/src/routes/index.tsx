import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/")(
  {
    head: () => ({
      meta: [
        { title: "VantaDB — Embedded Cognitive Memory for AI Agents" },
        {
          name: "description",
          content:
            "Open-source embedded vector database. HNSW + BM25 + hybrid search in one Rust binary. Sub-millisecond hybrid queries. Zero infrastructure. Apache 2.0.",
        },
        { property: "og:title", content: "VantaDB — Embedded Cognitive Memory for AI Agents" },
        {
          property: "og:description",
          content:
            "HNSW + BM25 + hybrid search in one Rust binary. Sub-millisecond hybrid queries.",
        },
      ],
      links: [{ rel: "canonical", href: "https://vantadb.dev" }],
    }),
  },
);
