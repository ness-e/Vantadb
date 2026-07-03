import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/integrations")({
  head: () => ({
    meta: [
      { title: "VantaDB — Ecosystem & Integrations" },
      {
        name: "description",
        content:
          "Integrate VantaDB vector store and persistent memory with native bindings for Python (PyO3), OpenAI, Ollama, CrewAI, Haystack, DSPy, LiteLLM, Mem0, Letta, and MCP (experimental).",
      },
    ],
    links: [{ rel: "canonical", href: "https://vantadb.dev/integrations" }],
  }),
});
