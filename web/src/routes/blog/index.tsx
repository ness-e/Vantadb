import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/blog/")({
  head: () => ({
    meta: [
      { title: "VantaDB — Engineering Blog" },
      {
        name: "description",
        content:
          "Engineering blog, release notes, and deep dives into embedded vector databases, AI agents, and local RAG.",
      },
    ],
    links: [{ rel: "canonical", href: "https://vantadb.dev/blog" }],
  }),
});
