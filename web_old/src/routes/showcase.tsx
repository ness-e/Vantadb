import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/showcase")({
  head: () => ({
    meta: [
      { title: "VantaDB — Community Showcase" },
      {
        name: "description",
        content:
          "See what the community is building with VantaDB. AI agents, RAG pipelines, edge applications, and more.",
      },
      { property: "og:title", content: "VantaDB — Community Showcase" },
      {
        property: "og:description",
        content: "See what the community is building with VantaDB.",
      },
    ],
    links: [{ rel: "canonical", href: "https://vantadb.dev/showcase" }],
  }),
});
