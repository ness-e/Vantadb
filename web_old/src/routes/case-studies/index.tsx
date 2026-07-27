import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/case-studies/")({
  head: () => ({
    meta: [
      { title: "VantaDB — Case Studies" },
      {
        name: "description",
        content:
          "Real-world case studies of teams using VantaDB for AI agents, local RAG, edge AI, and more.",
      },
      { property: "og:title", content: "VantaDB — Case Studies" },
      {
        property: "og:description",
        content: "Real-world case studies of teams using VantaDB.",
      },
      { property: "og:url", content: "https://vantadb.dev/case-studies" },
    ],
    links: [{ rel: "canonical", href: "https://vantadb.dev/case-studies" }],
  }),
});
