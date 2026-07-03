import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/solutions/local-rag")({
  head: () => ({
    meta: [
      { title: "VantaDB — Local-First RAG Pipeline" },
      {
        name: "description",
        content:
          "Run RAG entirely on-device. No vectors in the cloud. Embed documents locally, query with your local LLM, keep your data private.",
      },
    ],
    links: [{ rel: "canonical", href: "https://vantadb.dev/solutions/local-rag" }],
  }),
});
