import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/integrations")({
  head: () => ({
    meta: [
      { title: "VantaDB — Ecosystem & Integrations" },
      {
        name: "description",
        content:
          "Integrate VantaDB vector store and persistent memory tools natively inside LangChain, LlamaIndex and MCP runtimes.",
      },
    ],
    links: [{ rel: "canonical", href: "https://vantadb.dev/integrations" }],
  }),
});
