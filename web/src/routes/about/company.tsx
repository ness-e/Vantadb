import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/about/company")({
  head: () => ({
    meta: [
      { title: "About VantaDB — Company" },
      {
        name: "description",
        content:
          "VantaDB is an open-source embedded vector database for AI agents, local RAG pipelines, and intelligent applications. Built in Rust, Apache 2.0.",
      },
    ],
    links: [{ rel: "canonical", href: "https://vantadb.dev/about/company" }],
  }),
});
