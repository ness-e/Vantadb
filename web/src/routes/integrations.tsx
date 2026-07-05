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
      { property: "og:title", content: "VantaDB — Ecosystem & Integrations" },
      {
        property: "og:description",
        content: "Connect VantaDB to OpenAI, Ollama, MCP and more.",
      },
      { property: "og:url", content: "https://vantadb.dev/integrations" },
    ],
    links: [{ rel: "canonical", href: "https://vantadb.dev/integrations" }],
    scripts: [
      {
        type: "application/ld+json",
        children: JSON.stringify({
          "@context": "https://schema.org",
          "@type": "WebPage",
          name: "VantaDB — Ecosystem & Integrations",
          description:
            "Integrate VantaDB vector store and persistent memory with native bindings for Python (PyO3), OpenAI, Ollama, CrewAI, Haystack, DSPy, LiteLLM, Mem0, Letta, and MCP (experimental).",
        }),
      },
    ],
  }),
});
