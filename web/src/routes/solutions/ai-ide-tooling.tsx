import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/solutions/ai-ide-tooling")({
  head: () => ({
    meta: [
      { title: "VantaDB — AI-Powered IDE Tooling" },
      {
        name: "description",
        content:
          "Augment your coding workflow with semantic code search, AST-aware retrieval, and context-aware completions powered by an embedded vector database.",
      },
    ],
    links: [{ rel: "canonical", href: "https://vantadb.dev/solutions/ai-ide-tooling" }],
  }),
});
