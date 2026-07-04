import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/about/")({
  head: () => ({
    meta: [
      { title: "About VantaDB — The Database That Thinks With You" },
      {
        name: "description",
        content:
          "Learn about VantaDB: an embeddable, open-source vector database built for AI agents, local RAG, and intelligent applications.",
      },
    ],
    links: [{ rel: "canonical", href: "https://vantadb.dev/about" }],
  }),
});
