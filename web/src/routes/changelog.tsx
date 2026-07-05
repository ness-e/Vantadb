import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/changelog")({
  head: () => ({
    meta: [
      { title: "VantaDB — Changelog" },
      {
        name: "description",
        content:
          "Release notes for VantaDB. Track new features, performance improvements, bug fixes, and breaking changes across versions.",
      },
      { property: "og:title", content: "VantaDB — Changelog & Releases" },
      {
        property: "og:description",
        content: "Release notes, version history, and breaking changes.",
      },
      { property: "og:url", content: "https://vantadb.dev/changelog" },
    ],
    links: [{ rel: "canonical", href: "https://vantadb.dev/changelog" }],
    scripts: [
      {
        type: "application/ld+json",
        children: JSON.stringify({
          "@context": "https://schema.org",
          "@type": "WebPage",
          name: "VantaDB — Changelog",
          description:
            "Release notes for VantaDB. Track new features, performance improvements, bug fixes, and breaking changes across versions.",
        }),
      },
    ],
  }),
});
