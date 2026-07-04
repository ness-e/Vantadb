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
      { property: "og:title", content: "VantaDB — AI-Powered IDE Tooling" },
      {
        property: "og:description",
        content: "Semantic code search and AST-aware retrieval for IDEs.",
      },
      { property: "og:url", content: "https://vantadb.dev/solutions/ai-ide-tooling" },
    ],
    links: [{ rel: "canonical", href: "https://vantadb.dev/solutions/ai-ide-tooling" }],
  }),
});
