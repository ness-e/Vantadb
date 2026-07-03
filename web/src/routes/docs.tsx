import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/docs")({
  head: () => ({
    meta: [
      { title: "VantaDB — Documentation" },
      {
        name: "description",
        content:
          "Get started with VantaDB: embedded database for AI agents. Installation, quickstart, SDK reference, and guides.",
      },
    ],
    links: [{ rel: "canonical", href: "https://vantadb.dev/docs" }],
  }),
});
