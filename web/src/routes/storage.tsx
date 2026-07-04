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
    ],
    links: [{ rel: "canonical", href: "https://vantadb.dev/storage" }],
  }),
});
