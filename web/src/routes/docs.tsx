import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/docs")({
  head: () => ({
    meta: [
      { title: "VantaDB — Documentation" },
      {
        name: "description",
        content:
          "Get started with VantaDB: embedded database for AI agents. Installation, quickstart, SDK reference, and guides.",
      },
      { property: "og:title", content: "VantaDB — Documentation" },
      {
        property: "og:description",
        content: "Getting started guides, SDK references, and migration guides.",
      },
      { property: "og:url", content: "https://vantadb.dev/docs" },
    ],
    links: [{ rel: "canonical", href: "https://vantadb.dev/docs" }],
    scripts: [
      {
        type: "application/ld+json",
        children: JSON.stringify({
          "@context": "https://schema.org",
          "@type": "WebPage",
          name: "VantaDB — Documentation",
          description:
            "Get started with VantaDB: embedded database for AI agents. Installation, quickstart, SDK reference, and guides.",
        }),
      },
    ],
  }),
});
