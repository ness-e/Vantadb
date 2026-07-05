import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/playground")({
  head: () => ({
    meta: [
      { title: "VantaDB Playground — WASM Vector Search in Your Browser" },
      {
        name: "description",
        content:
          "Interactive WASM playground. Run VantaDB vector search entirely in your browser via WebAssembly. No server, no install.",
      },
      { property: "og:title", content: "VantaDB Playground" },
      {
        property: "og:description",
        content: "Run VantaDB vector search in your browser via WebAssembly.",
      },
      { property: "og:url", content: "https://vantadb.dev/playground" },
    ],
    links: [{ rel: "canonical", href: "https://vantadb.dev/playground" }],
  }),
});
