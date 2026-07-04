import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/storage")({
  head: () => ({
    meta: [
      { title: "VantaDB — Single-Binary Storage Architecture" },
      {
        name: "description",
        content:
          "VantaDB replaces Pinecone + Redis + S3 with a single Rust binary. LSM-tree storage, WAL durability, HNSW indexing — everything in one file.",
      },
      { property: "og:title", content: "VantaDB — Single-Binary Storage Architecture" },
      {
        property: "og:description",
        content: "VantaDB replaces Pinecone + Redis + S3 with one Rust binary.",
      },
      { property: "og:url", content: "https://vantadb.dev/storage" },
    ],
    links: [{ rel: "canonical", href: "https://vantadb.dev/storage" }],
  }),
});
