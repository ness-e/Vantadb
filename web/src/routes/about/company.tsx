import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/about/company")({
  head: () => ({
    meta: [
      { title: "About VantaDB — Company" },
      {
        name: "description",
        content:
          "VantaDB is an open-source embedded vector database for AI agents, local RAG pipelines, and intelligent applications. Built in Rust, Apache 2.0.",
      },
      { property: "og:title", content: "VantaDB — Company & Team" },
      {
        property: "og:description",
        content: "The team behind VantaDB.",
      },
      { property: "og:url", content: "https://vantadb.dev/about/company" },
    ],
    links: [{ rel: "canonical", href: "https://vantadb.dev/about/company" }],
    scripts: [
      {
        type: "application/ld+json",
        children: JSON.stringify({
          "@context": "https://schema.org",
          "@type": "AboutPage",
          name: "About VantaDB — Company",
        }),
      },
    ],
  }),
});
