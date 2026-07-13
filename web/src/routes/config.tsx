import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/config")({
  head: () => ({
    meta: [
      { title: "VantaDB — Zero Configuration: Schema-Free" },
      {
        name: "description",
        content:
          "No YAML files, no .env secrets, no migration scripts. VantaDB requires zero configuration — just pip install and connect.",
      },
      { property: "og:title", content: "VantaDB — Zero Configuration: Schema-Free" },
      {
        property: "og:description",
        content: "VantaDB requires no configuration and no schema definitions.",
      },
      { property: "og:url", content: "https://vantadb.dev/config" },
    ],
    links: [{ rel: "canonical", href: "https://vantadb.dev/config" }],
    scripts: [
      {
        type: "application/ld+json",
        children: JSON.stringify({
          "@context": "https://schema.org",
          "@type": "WebPage",
          name: "VantaDB — Zero Configuration: Schema-Free",
          description:
            "No YAML files, no .env secrets, no migration scripts. VantaDB requires zero configuration — just pip install and connect.",
          url: "https://vantadb.dev/config",
          image: "https://vantadb.dev/og/default.svg",
        }),
      },
    ],
  }),
});
