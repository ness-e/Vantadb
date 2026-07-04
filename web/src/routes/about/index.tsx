import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/about/")({
  head: () => ({
    meta: [
      { title: "About VantaDB — The Database That Thinks With You" },
      {
        name: "description",
        content:
          "Learn about VantaDB: an embeddable, open-source vector database built for AI agents, local RAG, and intelligent applications.",
      },      { property: "og:title", content: "About VantaDB" },
      {
        property: "og:description",
        content: "Open-source embedded database for AI agents.",
      },
      { property: "og:url", content: "https://vantadb.dev/about" },
    ],
    links: [{ rel: "canonical", href: "https://vantadb.dev/about" }],
  }),
});



