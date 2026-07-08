import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/demo")({
  head: () => ({
    meta: [
      { title: "VantaDB — Browser Demo" },
      {
        name: "description",
        content:
          "Try VantaDB in your browser. AI-powered vector memory running entirely client-side via WebAssembly.",
      },
      { property: "og:title", content: "VantaDB — Try It in Your Browser" },
      {
        property: "og:description",
        content:
          "AI-powered vector memory running entirely client-side via WASM. No server, no install.",
      },
      { property: "og:url", content: "https://vantadb.dev/demo" },
    ],
    links: [{ rel: "canonical", href: "https://vantadb.dev/demo" }],
    scripts: [
      {
        type: "application/ld+json",
        children: JSON.stringify({
          "@context": "https://schema.org",
          "@type": "WebApplication",
          name: "VantaDB Browser Demo",
          applicationCategory: "DeveloperApplication",
          browserRequirements: "Requires JavaScript and WebAssembly",
          description:
            "Interactive demo of VantaDB vector database running entirely in the browser via WebAssembly.",
        }),
      },
    ],
  }),
});
