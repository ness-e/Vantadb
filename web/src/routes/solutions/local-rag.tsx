import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/solutions/local-rag")({
  head: () => ({
    meta: [
      { title: "VantaDB — Local-First RAG Pipeline" },
      {
        name: "description",
        content:
          "Run RAG entirely on-device. No vectors in the cloud. Embed documents locally, query with your local LLM, keep your data private.",
      },      { property: "og:title", content: "VantaDB — Local-First RAG Pipeline" },
      {
        property: "og:description",
        content: "Run RAG entirely on-device with zero cloud dependencies.",
      },
      { property: "og:url", content: "https://vantadb.dev/solutions/local-rag" },
    ],
    links: [{ rel: "canonical", href: "https://vantadb.dev/solutions/local-rag" }],
  }),
});



